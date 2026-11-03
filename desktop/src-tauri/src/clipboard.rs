//! Explicit copy only. No clipboard reads, monitoring, paste, or permission changes.
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
    #[cfg(not(target_os = "macos"))]
    Err("unsupported")
}
