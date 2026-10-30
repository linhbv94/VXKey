import AppKit
import InputMethodKit
final class VXKeyDelegate:NSObject,NSApplicationDelegate {
    private var server:IMKServer?
    func applicationDidFinishLaunching(_ notification:Notification){
        AppSettings.shared.start()
        server=IMKServer(name:"VXKey_1_Connection",bundleIdentifier:Bundle.main.bundleIdentifier)
    }
}
let app=NSApplication.shared
let delegate=VXKeyDelegate()
app.delegate=delegate
app.setActivationPolicy(.accessory)
app.run()
