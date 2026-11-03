//! VXKey bridge ticks run on a main-loop timer, never from a key handler.
use std::{
    ffi::{CStr, CString, c_char},
    sync::OnceLock,
};
use vxkey_runtime::bridge::{NativeStatus, local::Client};
static CLIENT: OnceLock<Client> = OnceLock::new();
/// # Safety
/// root must point to a valid NUL-terminated UTF-8 string for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vxkey_native_start(root: *const c_char) -> bool {
    std::panic::catch_unwind(|| {
        // This dedicated native process must not print a panic payload that could
        // include input text. Recovery uses typed metadata, never the panic body.
        static HOOK: std::sync::Once = std::sync::Once::new();
        HOOK.call_once(|| std::panic::set_hook(Box::new(|_| {})));
        if root.is_null() || CLIENT.get().is_some() {
            return false;
        }
        // SAFETY: the C caller provides a live terminated path for this call.
        let Ok(path) = unsafe { CStr::from_ptr(root) }.to_str() else {
            return false;
        };
        let Ok(client) = Client::start(std::path::Path::new(path)) else {
            return false;
        };
        CLIENT.set(client).is_ok()
    })
    .unwrap_or(false)
}
/// # Safety
/// status is a NUL-terminated JSON string, valid throughout this call. The result
/// must be released exactly once with vxkey_native_free; NULL means safe bypass.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vxkey_native_tick(status: *const c_char) -> *mut c_char {
    std::panic::catch_unwind(|| {
        if status.is_null() {
            return std::ptr::null_mut();
        }
        // SAFETY: caller's pointer lifetime and termination are required by this API.
        let bytes = unsafe { CStr::from_ptr(status) }.to_bytes();
        if bytes.len() > 16384 {
            return std::ptr::null_mut();
        }
        let Some(client) = CLIENT.get() else {
            return std::ptr::null_mut();
        };
        let Ok(status) = serde_json::from_slice::<NativeStatus>(bytes) else {
            return std::ptr::null_mut();
        };
        let delivery = match client.tick(status) {
            Ok(value) => value,
            Err(error) if error == "busy" => {
                return CString::new(r#"{"busy":true}"#).unwrap().into_raw();
            }
            Err(_) => return std::ptr::null_mut(),
        };
        serde_json::to_vec(&delivery)
            .ok()
            .and_then(|v| CString::new(v).ok())
            .map(CString::into_raw)
            .unwrap_or(std::ptr::null_mut())
    })
    .unwrap_or(std::ptr::null_mut())
}
/// # Safety
/// value must be NULL or the unreleased return value of vxkey_native_tick.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vxkey_native_free(value: *mut c_char) {
    if !value.is_null() {
        // SAFETY: exactly-once ownership transfer is the caller contract.
        drop(unsafe { CString::from_raw(value) });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn vxkey_native_engine_faulted() -> bool {
    crate::abi::native_faulted()
}
