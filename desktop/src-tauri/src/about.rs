//! The system About panel, shared by app menu and tray; never a Settings modal.
pub const REPOSITORY: &str = "https://github.com/linhbv94/VXKey";
pub const COPYRIGHT: &str = "© 2026 Bùi Việt Linh. All rights reserved.";
pub fn summary(vi: bool) -> &'static str {
    if vi {
        "Bộ gõ tiếng Việt với Telex, VNI và gõ tắt."
    } else {
        "Vietnamese input method with Telex, VNI, and text expansion."
    }
}
pub fn show(vi: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use objc2::{AnyThread, MainThreadMarker, runtime::AnyObject};
        use objc2_app_kit::{
            NSApplication, NSFont, NSImage, NSLineBreakMode, NSMutableParagraphStyle,
            NSTextAlignment,
        };
        use objc2_foundation::{NSAttributedString, NSData, NSDictionary, NSSize, NSString};
        let mtm = MainThreadMarker::new().ok_or("not_main_thread")?;
        let icon = NSImage::initWithData(
            NSImage::alloc(),
            &NSData::with_bytes(include_bytes!("../../public/app_icon.png")),
        )
        .ok_or("invalid_icon")?;
        // The artwork is RGBA 512px. AppKit chooses its display size in the panel.
        icon.setSize(NSSize::new(512.0, 512.0));
        let name = NSString::from_str("VXKey");
        let version = NSString::from_str(env!("CARGO_PKG_VERSION"));
        let build = NSString::from_str("");
        let copyright = NSString::from_str(COPYRIGHT);
        let credit_text = NSString::from_str(&format!(
            "Bùi Việt Linh\n{}\nGitHub: {}",
            summary(vi),
            REPOSITORY
        ));
        let font = NSFont::systemFontOfSize(11.0);
        let paragraph = NSMutableParagraphStyle::new();
        paragraph.setAlignment(NSTextAlignment::Center);
        paragraph.setLineBreakMode(NSLineBreakMode::ByWordWrapping);
        paragraph.setLineSpacing(1.0);
        let font_key = NSString::from_str("NSFont");
        let paragraph_key = NSString::from_str("NSParagraphStyle");
        let attribute_values: [&AnyObject; 2] = [font.as_ref(), paragraph.as_ref()];
        let attribute_keys: [&NSString; 2] = [font_key.as_ref(), paragraph_key.as_ref()];
        let attributes = NSDictionary::from_slices(&attribute_keys, &attribute_values);
        // Concrete NSFont/NSParagraphStyle objects match their AppKit attribute keys.
        let credits = unsafe {
            NSAttributedString::initWithString_attributes(
                NSAttributedString::alloc(),
                &credit_text,
                Some(&attributes),
            )
        };
        let keys = [
            "ApplicationName",
            "ApplicationVersion",
            "Version",
            "ApplicationIcon",
            "Credits",
            "Copyright",
        ]
        .map(NSString::from_str);
        let key_refs: Vec<&NSString> = keys.iter().map(|k| k.as_ref()).collect();
        let values: [&AnyObject; 6] = [
            name.as_ref(),
            version.as_ref(),
            build.as_ref(),
            icon.as_ref(),
            credits.as_ref(),
            copyright.as_ref(),
        ];
        let options = NSDictionary::from_slices(&key_refs, &values);
        let app = NSApplication::sharedApplication(mtm);
        app.activate();
        // Types and values above match the documented standard About options.
        unsafe { app.orderFrontStandardAboutPanelWithOptions(&options) };
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = vi;
        Err("unsupported".into())
    }
}
