// Controlled local IMK QA and recovery. No keyboard capture, hooks or TCC changes.
import Foundation
import AppKit
import Carbon
let application = NSApplication.shared
application.setActivationPolicy(.prohibited)
let vxIDs = ["com.vxkey.inputmethod.vxkey", "com.vxkey.inputmethod.vxkey.vietnamese"]
func property(_ source: TISInputSource, _ key: CFString) -> Any? {
    guard let p = TISGetInputSourceProperty(source, key) else { return nil }
    return Unmanaged<AnyObject>.fromOpaque(p).takeUnretainedValue()
}
func id(_ source: TISInputSource) -> String { property(source,kTISPropertyInputSourceID) as? String ?? "" }
func sources(_ all: Bool = true) -> [TISInputSource] {
    let general = TISCreateInputSourceList(nil,all).map { $0.takeRetainedValue() as NSArray as! [TISInputSource] } ?? []
    let filter = [kTISPropertyBundleID as String: "com.vxkey.inputmethod.vxkey"] as CFDictionary
    let scoped = TISCreateInputSourceList(filter,all).map { $0.takeRetainedValue() as NSArray as! [TISInputSource] } ?? []
    var seen = Set<String>()
    return (general + scoped).filter { seen.insert(id($0)).inserted }
}
func current() -> String { id(TISCopyCurrentKeyboardInputSource().takeRetainedValue()) }
func find(_ identifier: String) -> TISInputSource? { sources().first { id($0)==identifier } }
func result(_ code: OSStatus, _ action: String) -> Never {
    let data=try! JSONSerialization.data(withJSONObject:["action":action,"os_status":code,"selected":current()],options:[.sortedKeys])
    print(String(data:data,encoding:.utf8)!);exit(code == noErr ? 0 : 1)
}
let action=CommandLine.arguments.dropFirst().first ?? "status"
switch action {
case "status":
    let rows=sources().filter { vxIDs.contains(id($0)) || (property($0,kTISPropertyBundleID) as? String)=="com.vxkey.inputmethod.vxkey" || id($0)=="com.apple.keylayout.ABC" || id($0)==current() }.map { source -> [String:Any] in
        ["id":id(source),"bundle_id":property(source,kTISPropertyBundleID) as? String ?? "", "name":property(source,kTISPropertyLocalizedName) as? String ?? "", "enabled":property(source,kTISPropertyInputSourceIsEnabled) as? Bool ?? false,
         "selectable":property(source,kTISPropertyInputSourceIsSelectCapable) as? Bool ?? false,
         "enable_capable":property(source,kTISPropertyInputSourceIsEnableCapable) as? Bool ?? false,
         "languages":property(source,kTISPropertyInputSourceLanguages) as? [String] ?? [],
         "mode_id":property(source,kTISPropertyInputModeID) as? String ?? "",
         "type":property(source,kTISPropertyInputSourceType) as? String ?? ""]
    }
    let data=try! JSONSerialization.data(withJSONObject:["selected":current(),"sources":rows,"launchservices_app":NSWorkspace.shared.urlForApplication(withBundleIdentifier:"com.vxkey.inputmethod.vxkey")?.path ?? ""],options:[.sortedKeys]);print(String(data:data,encoding:.utf8)!)
case "register":
    let app=URL(fileURLWithPath:NSHomeDirectory()+"/Library/Input Methods/VXKeyInput.app",isDirectory:true)
    guard Bundle(url:app)?.bundleIdentifier=="com.vxkey.inputmethod.vxkey" else {result(-50,action)}
    result(TISRegisterInputSource(app as CFURL),action)
case "enable":
    let ordered=sources().filter{vxIDs.contains(id($0))}.sorted{a,b in id(a).count < id(b).count}
    guard !ordered.isEmpty else {result(-50,action)}
    for source in ordered where (property(source,kTISPropertyInputSourceIsEnabled) as? Bool) != true {
        let code=TISEnableInputSource(source);if code != noErr {result(code,action)}
        RunLoop.current.run(until:Date().addingTimeInterval(0.5))
    }
    result(noErr,action)
case "select-vxkey":
    guard let source=find("com.vxkey.inputmethod.vxkey.vietnamese") ?? sources().first(where:{vxIDs.contains(id($0)) && (property($0,kTISPropertyInputSourceIsSelectCapable) as? Bool)==true}) else {result(-50,action)}
    result(TISSelectInputSource(source),action)
case "recover-abc", "recover":
    guard vxIDs.contains(current()) else {result(noErr,action)}
    for identifier in ["com.apple.keylayout.ABC", "com.apple.keylayout.US"] {
        if let source=find(identifier),(property(source,kTISPropertyInputSourceIsEnabled) as? Bool)==true {result(TISSelectInputSource(source),action)}
    }
    result(-50,action)
case "restore":
    guard CommandLine.arguments.count==3,let source=find(CommandLine.arguments[2]),(property(source,kTISPropertyInputSourceIsEnabled) as? Bool)==true else {result(-50,action)}
    result(TISSelectInputSource(source),action)
case "disable-vxkey":
    guard !vxIDs.contains(current()) else {result(-50,action)}
    let ordered=sources().filter{vxIDs.contains(id($0))}.sorted{a,b in id(a).count>id(b).count}
    for source in ordered where (property(source,kTISPropertyInputSourceIsEnabled) as? Bool)==true {
        let code=TISDisableInputSource(source);if code != noErr {result(code,action)}
    }
    result(noErr,action)
default: result(-50,"invalid_action")
}
