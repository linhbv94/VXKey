// Real AppSettings + real Rust FFI/composer; IMK controller is replaced only to
// avoid registering an input source or editing any document during this fixture.
import AppKit
import InputMethodKit
final class FixtureSession { func reset() {} }
final class FunputInputController {
    let composer = FunputComposer()
    let session = FixtureSession()
    func syncSettings() { composer.apply(ComposerConfiguration(settings: AppSettings.shared)) }
}
let controller = FunputInputController()
let settings = AppSettings.shared
settings.activeController = controller
settings.start()
func respond(_ value: [String: Any]) {
    let data = try! JSONSerialization.data(withJSONObject: value, options: [.sortedKeys])
    print(String(data: data, encoding: .utf8)!)
    fflush(stdout)
}
respond(["ready": true])
DispatchQueue.global().async {
    while let line = readLine(), let data = line.data(using: .utf8),
          let request = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any] {
        DispatchQueue.main.async {
            switch request["fixture"] as? String {
            case "compose": _ = controller.composer.process("a")
            case "clear": controller.composer.clear()
            case "stall": Thread.sleep(forTimeInterval: 3.6)
            default: break
            }
            respond(["input_method": settings.inputMethod.rawValue,
                     "suspended": settings.suspended,
                     "macro_count": settings.shortcuts.count,
                     "composing": !controller.composer.buffer().isEmpty])
        }
    }
    exit(0)
}
RunLoop.main.run()
