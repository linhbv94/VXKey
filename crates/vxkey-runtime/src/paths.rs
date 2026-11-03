//! One trusted host data root shared by companion and IME. Never accept a JS path.
use std::path::PathBuf;
pub fn data_dir() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("VXKEY_DATA_DIR") {
        let path = PathBuf::from(path);
        return if path.is_absolute() {
            Ok(path)
        } else {
            Err("invalid_data_dir".into())
        };
    }
    #[cfg(target_os = "macos")]
    if let Some(home) = std::env::var_os("HOME") {
        return Ok(PathBuf::from(home).join("Library/Application Support/VXKey"));
    }
    #[cfg(target_os = "windows")]
    if let Some(root) = std::env::var_os("APPDATA") {
        return Ok(PathBuf::from(root).join("VXKey"));
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(root) = std::env::var_os("XDG_DATA_HOME") {
            return Ok(PathBuf::from(root).join("vxkey"));
        }
        if let Some(home) = std::env::var_os("HOME") {
            return Ok(PathBuf::from(home).join(".local/share/vxkey"));
        }
    }
    Err("data_dir_unavailable".into())
}
