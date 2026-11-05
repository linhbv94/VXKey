//! Presentation metadata only. Policy IDs, PID, generation and target tokens are
//! never replaced by a label; OS lookups do not run in the keyboard callback.
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Mutex, OnceLock},
};
fn cache() -> &'static Mutex<BTreeMap<String, String>> {
    static CACHE: OnceLock<Mutex<BTreeMap<String, String>>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}
fn label(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty() && value.chars().count() <= 256 && !value.chars().any(char::is_control))
        .then(|| value.to_owned())
}
pub fn fallback(id: &str) -> String {
    let component = if id.contains(['\\', '/']) {
        id.rsplit(['\\', '/']).next().unwrap_or(id)
    } else {
        id.rsplit('.').next().unwrap_or(id)
    };
    let component = if component.to_ascii_lowercase().ends_with(".exe") {
        &component[..component.len() - 4]
    } else {
        component
    };
    label(component).unwrap_or_else(|| "Application".into())
}
pub fn decorate(data: &mut Value) {
    let target = data["runtime"]["current_app"].clone();
    if let Some(id) = target["app_id"].as_str() {
        let resolved = platform_name(
            id,
            target["pid"].as_i64().and_then(|p| i32::try_from(p).ok()),
        );
        let name = resolved
            .and_then(|s| label(&s))
            .unwrap_or_else(|| fallback(id));
        if let Ok(mut names) = cache().lock() {
            if names.len() >= 2000 && !names.contains_key(id) {
                names.clear();
            }
            names.insert(id.into(), name.clone());
        }
        data["runtime"]["current_app"]["display_name"] = json!(name);
    }
    let mut names = serde_json::Map::new();
    let ids: BTreeSet<String> = ["policies", "language_memory"]
        .into_iter()
        .filter_map(|g| data["config"]["apps"][g].as_object())
        .flat_map(|o| o.keys().cloned())
        .collect();
    let mut remaining = 16;
    for id in ids {
        let cached = cache().lock().ok().and_then(|c| c.get(&id).cloned());
        let name = if let Some(cached) = cached {
            cached
        } else if remaining > 0 {
            remaining -= 1;
            // Bounded metadata discovery for saved apps. Do not hold a lock over
            // an OS call, and never use a label as an exclusion identity.
            let name = platform_name(&id, None)
                .and_then(|s| label(&s))
                .unwrap_or_else(|| fallback(&id));
            if let Ok(mut cached) = cache().lock() {
                if cached.len() >= 2000 {
                    cached.clear();
                }
                cached.insert(id.clone(), name.clone());
            }
            name
        } else {
            fallback(&id)
        };
        names.insert(id, json!(name));
    }
    data["runtime"]["app_names"] = Value::Object(names);
}
#[cfg(target_os = "macos")]
fn platform_name(id: &str, pid: Option<i32>) -> Option<String> {
    use objc2_app_kit::NSRunningApplication;
    use objc2_foundation::NSString;
    let app = if let Some(pid) = pid {
        NSRunningApplication::runningApplicationWithProcessIdentifier(pid)?
    } else {
        NSRunningApplication::runningApplicationsWithBundleIdentifier(&NSString::from_str(id))
            .firstObject()?
    };
    // Opening the tray changes frontmostApplication. Resolve the exact cached
    // native target and reject a reused PID for a different application.
    if app.bundleIdentifier()?.to_string() != id {
        return None;
    }
    Some(app.localizedName()?.to_string())
}
#[cfg(windows)]
fn platform_name(id: &str, _pid: Option<i32>) -> Option<String> {
    if let Some(name) = cache().lock().ok()?.get(id).cloned() {
        return Some(name);
    }
    use windows_sys::Win32::Storage::FileSystem::*;
    let path: Vec<_> = id.encode_utf16().chain(Some(0)).collect();
    let size = unsafe { GetFileVersionInfoSizeW(path.as_ptr(), std::ptr::null_mut()) } as usize;
    if size == 0 || size > 4 * 1024 * 1024 {
        return None;
    }
    let mut data = vec![0u32; size.div_ceil(4)];
    if unsafe { GetFileVersionInfoW(path.as_ptr(), 0, size as u32, data.as_mut_ptr().cast()) } == 0
    {
        return None;
    }
    let query = |key: &str, unit_bytes: usize| -> Option<Vec<u8>> {
        let key: Vec<_> = key.encode_utf16().chain(Some(0)).collect();
        let mut value = std::ptr::null_mut();
        let mut count = 0u32;
        if unsafe { VerQueryValueW(data.as_ptr().cast(), key.as_ptr(), &mut value, &mut count) }
            == 0
        {
            return None;
        }
        let len = (count as usize).checked_mul(unit_bytes)?;
        let start = value as usize;
        let base = data.as_ptr() as usize;
        if len == 0 || start < base || start.checked_add(len)? > base.checked_add(size)? {
            return None;
        }
        Some(unsafe { std::slice::from_raw_parts(value.cast::<u8>(), len) }.to_vec())
    };
    let languages = query(r"\VarFileInfo\Translation", 1)?;
    for pair in languages.chunks_exact(4).take(64) {
        let lang = u16::from_le_bytes([pair[0], pair[1]]);
        let code = u16::from_le_bytes([pair[2], pair[3]]);
        for field in ["FileDescription", "ProductName"] {
            if let Some(bytes) = query(&format!(r"\StringFileInfo\{lang:04x}{code:04x}\{field}"), 2)
            {
                let units: Vec<_> = bytes
                    .chunks_exact(2)
                    .map(|p| u16::from_le_bytes([p[0], p[1]]))
                    .take_while(|u| *u != 0)
                    .collect();
                if let Ok(value) = String::from_utf16(&units) {
                    if let Some(value) = label(&value) {
                        return Some(value);
                    }
                }
            }
        }
    }
    None
}
#[cfg(not(any(windows, target_os = "macos")))]
fn platform_name(_id: &str, _pid: Option<i32>) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fallbacks_do_not_show_raw_bundle_or_executable_path() {
        assert_eq!(fallback("com.apple.antigravity"), "antigravity");
        assert_eq!(fallback(r"C:\Apps\Figma.EXE"), "Figma");
        assert_eq!(fallback("com.microsoft.VSCode"), "VSCode");
    }
    #[test]
    fn label_cannot_change_exclusion_identity() {
        let mut data = json!({"config":{"apps":{"policies":{"com.example.editor":{"input_bypass":true}}}},"runtime":{"current_app":{"app_id":"com.example.editor","pid":2147483647,"target_token":"keep-token","input_bypass":true}}});
        decorate(&mut data);
        assert_eq!(
            data["runtime"]["current_app"]["app_id"],
            "com.example.editor"
        );
        assert_eq!(data["runtime"]["current_app"]["target_token"], "keep-token");
        assert_eq!(data["runtime"]["current_app"]["input_bypass"], true);
        assert_eq!(data["runtime"]["current_app"]["display_name"], "editor");
    }
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "Requires a logged-in macOS desktop with Finder; run explicitly for native metadata QA"]
    fn native_metadata_uses_matching_pid_and_bundle() {
        use objc2_app_kit::NSRunningApplication;
        use objc2_foundation::NSString;
        let apps = NSRunningApplication::runningApplicationsWithBundleIdentifier(
            &NSString::from_str("com.apple.finder"),
        );
        let app = apps
            .firstObject()
            .expect("Finder must be running for this explicit native fixture");
        let pid = app.processIdentifier();
        assert_eq!(
            platform_name("com.apple.finder", Some(pid)),
            app.localizedName().map(|n| n.to_string())
        );
        assert!(platform_name("com.wrong.target", Some(pid)).is_none());
    }
}
