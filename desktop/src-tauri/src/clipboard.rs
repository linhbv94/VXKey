//! Explicit copy only. No clipboard reads, monitoring, paste, or permission changes.
#[cfg(not(windows))]
pub fn write(text: &str) -> Result<(), &'static str> {
    if text.len() > 1024 * 1024 {
        return Err("quota_exceeded");
    }
    #[cfg(target_os = "macos")]
    {
        use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString};
        use objc2_foundation::NSString;
        objc2::rc::autoreleasepool(|_| {
            let pasteboard = NSPasteboard::generalPasteboard();
            let value = NSString::from_str(text);
            pasteboard.clearContents();
            // AppKit exports a process-lifetime NSString constant for the text UTI.
            if pasteboard.setString_forType(&value, unsafe { NSPasteboardTypeString }) {
                Ok(())
            } else {
                Err("write_failed")
            }
        })
    }
    #[cfg(windows)]
    { return Err("owner_required"); }
    #[cfg(not(any(target_os = "macos",windows)))]
    Err("unsupported")
}

#[cfg(windows)]
pub fn write_windows(text:&str, owner:windows_sys::Win32::Foundation::HWND)->Result<(),&'static str>{
        if text.len()>1024*1024 || owner.is_null(){return Err("quota_exceeded")}
        use windows_sys::Win32::{Foundation::GlobalFree,System::{Memory::*,DataExchange::*}};
        let units:Vec<u16>=text.encode_utf16().chain(Some(0)).collect();
        unsafe {
            let memory=GlobalAlloc(GMEM_MOVEABLE,units.len()*2);
            if memory.is_null(){return Err("write_failed")}
            let data=GlobalLock(memory);
            if data.is_null(){GlobalFree(memory);return Err("write_failed")}
            std::ptr::copy_nonoverlapping(units.as_ptr(),data.cast::<u16>(),units.len());GlobalUnlock(memory);
            if OpenClipboard(owner)==0{GlobalFree(memory);return Err("clipboard_busy")}
            let success=EmptyClipboard()!=0 && !SetClipboardData(13,memory).is_null();
            CloseClipboard();
            if !success{GlobalFree(memory);return Err("write_failed")}
            Ok(())
        }
}
