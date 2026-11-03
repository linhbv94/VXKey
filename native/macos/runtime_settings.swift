// VXKey native cache adapter. Upstream composer/controller remain Rust C-ABI based.
import AppKit
import InputMethodKit
import Foundation
import Carbon

struct TextShortcut {let trigger:String;let expansion:String;var isComplete:Bool{!trigger.isEmpty && !expansion.isEmpty}}
struct KeyCombo {
    let kind:String;let modifiers:Set<String>;let keyCode:UInt16?
    static let defaultToggle=KeyCombo(kind:"modifier_chord",modifiers:["control","alt"],keyCode:nil)
    static func decode(_ value:Any?)->KeyCombo?{
        guard let v=value as? [String:Any],let kind=v["kind"] as? String,let raw=v["modifiers"] as? [String],!raw.isEmpty else{return nil}
        let keys:[String:UInt16]=[
            "KeyA":0,"KeyS":1,"KeyD":2,"KeyF":3,"KeyH":4,"KeyG":5,"KeyZ":6,"KeyX":7,"KeyC":8,"KeyV":9,"KeyB":11,
            "KeyQ":12,"KeyW":13,"KeyE":14,"KeyR":15,"KeyY":16,"KeyT":17,"Digit1":18,"Digit2":19,"Digit3":20,"Digit4":21,
            "Digit6":22,"Digit5":23,"Equal":24,"Digit9":25,"Digit7":26,"Minus":27,"Digit8":28,"Digit0":29,"BracketRight":30,
            "KeyO":31,"KeyU":32,"BracketLeft":33,"KeyI":34,"KeyP":35,"Enter":36,"KeyL":37,"KeyJ":38,"Quote":39,"KeyK":40,
            "Semicolon":41,"Backslash":42,"Comma":43,"Slash":44,"KeyN":45,"KeyM":46,"Period":47,"Tab":48,"Space":49,"Backquote":50,
            "Backspace":51,"Escape":53,"F1":122,"F2":120,"F3":99,"F4":118,"F5":96,"F6":97,"F7":98,"F8":100,"F9":101,
            "F10":109,"F11":103,"F12":111,"Home":115,"PageUp":116,"Delete":117,"End":119,"PageDown":121,
            "ArrowLeft":123,"ArrowRight":124,"ArrowDown":125,"ArrowUp":126
        ]
        if kind=="modifier_chord",raw.count>=2{return KeyCombo(kind:kind,modifiers:Set(raw),keyCode:nil)}
        guard kind=="key_chord",let key=v["key_code"] as? String,let code=keys[key]else{return nil}
        return KeyCombo(kind:kind,modifiers:Set(raw),keyCode:code)
    }
    static func families(_ f:NSEvent.ModifierFlags)->Set<String>{var result=Set<String>();if f.contains(.control){result.insert("control")};if f.contains(.option){result.insert("alt")};if f.contains(.shift){result.insert("shift")};if f.contains(.command){result.insert("meta")};return result}
    func matches(_ e:NSEvent)->Bool{kind=="key_chord" && keyCode==e.keyCode && Self.families(e.modifierFlags)==modifiers}
}
final class NativeModifierGesture {
    private var down:Set<UInt16>=[];private var armed=false;private var cancelled=false
    private let names:[UInt16:String]=[59:"control",62:"control",58:"alt",61:"alt",56:"shift",60:"shift",55:"meta",54:"meta"]
    func reset(){down=[];armed=false;cancelled=false}
    func nonModifier(){if !down.isEmpty{cancelled=true;armed=false}}
    func handle(_ event:NSEvent,combo:KeyCombo)->Bool{
        guard combo.kind=="modifier_chord",let family=names[event.keyCode]else{return false}
        let pressed=CGEventSource.keyState(.combinedSessionState,key:CGKeyCode(event.keyCode));let hadDown = !down.isEmpty
        if pressed{down.insert(event.keyCode)}else{down.remove(event.keyCode)}
        if !combo.modifiers.contains(family){cancelled=true;armed=false}
        let families=Set(down.compactMap{names[$0]})
        if families==combo.modifiers && !cancelled {armed=true}
        if hadDown && down.isEmpty{let fire=armed && !cancelled;reset();return fire}
        return false
    }
}

/// Main-thread immutable-cache owner. All I/O and signature checks live in Rust's
/// bridge worker. No timer can declare the event loop healthy if that loop stalls.
final class AppSettings {
    static let shared = AppSettings()
    var inputMethod: InputMethod = .telex
    var toneStyle: ToneStyle = .modern
    var vietnameseEnabled = true
    var smartEnglishRestore = true
    var eagerRestore = true
    var spellCheckEnabled = false
    var autoCapitalizeEnabled = false
    var extraOnsets: ExtraOnsetLetters = []
    var retoneAfterBackspace = true
    var directComposition = false
    var shortcutsEnabled = true
    var shortcutSmartCase = true
    var shortcutsInEnglish = true
    var shortcuts: [TextShortcut] = []
    var shortcutsRevision = 0
    var toggleShortcut: KeyCombo? = .defaultToggle
    var flipShortcut: KeyCombo?
    private(set) var suspended = true
    private var policies: [String: Bool] = [:]
    private var memory: [String: Bool] = [:]
    private var localMemory: [String: Bool] = [:]
    private var remember = true
    private var revision = "0"
    private var savedMode = "vi"
    private var pendingConfig: [String: Any]?
    private var pendingMemory: [String: Any]?
    private var result: [String: String]?
    private let engineSession = UUID().uuidString.lowercased()
    private var generation: UInt64 = 0
    private var sequence: UInt64 = 0
    private var activeInput = false
    private var targetID: String?
    private var targetPID: Int32?
    private var timer: Timer?
    private var connected = false
    private var linkFault = false
    private var latched = true
    private var leaseDeadline: TimeInterval = 0
    weak var activeController: FunputInputController?
    var dataRoot: URL {
        if let path = ProcessInfo.processInfo.environment["VXKEY_DATA_DIR"], path.hasPrefix("/") {
            return URL(fileURLWithPath: path)
        }
        return FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("VXKey", isDirectory: true)
    }
    func start() {
        guard dataRoot.path.withCString({ vxkey_native_start($0) }) else { return }
        tick()
        let timer = Timer(timeInterval: 0.1, repeats: true) { [weak self] _ in self?.tick() }
        self.timer = timer
        RunLoop.main.add(timer, forMode: .common)
    }
    func activate(_ controller: FunputInputController, client: IMKTextInput?) {
        generation &+= 1
        activeController = controller
        activeInput = true
        let front = NSWorkspace.shared.frontmostApplication
        let id = client?.bundleIdentifier()
        if let id, !id.hasPrefix("com.vxkey."), front?.bundleIdentifier == id {
            targetID = id; targetPID = front?.processIdentifier
        } else { targetID = nil; targetPID = nil }
    }
    func deactivate(_ controller: FunputInputController) {
        // Keep the original external client while our tray/Settings has focus.
        // A different external foreground app invalidates it on the next tick.
        if activeController === controller {
            activeInput = false
            if !vxkey_native_engine_faulted() { controller.composer.clear(); controller.session.reset() }
        }
    }
    private func target() -> [String: Any]? {
        guard let id = targetID, let pid = targetPID else { return nil }
        return ["app_id": id, "pid": pid, "ime_session_id": engineSession,
                "focus_generation": String(generation)]
    }
    private func refreshTarget() {
        guard let front = NSWorkspace.shared.frontmostApplication else { return }
        if front.bundleIdentifier?.hasPrefix("com.vxkey.") == true { return }
        if targetID != front.bundleIdentifier || targetPID != front.processIdentifier {
            generation &+= 1; targetID = nil; targetPID = nil
        }
    }
    private func tick() {
        precondition(Thread.isMainThread)
        refreshTarget()
        sequence &+= 1
        let faulted = vxkey_native_engine_faulted()
        let composing = !faulted && !(activeController?.composer.buffer().isEmpty ?? true)
        let active = vxkey_native_source_active() && target() != nil
        let faultCode: String? = faulted ? "engine_fault" : (linkFault ? "bridge_stale" : nil)
        let status: [String: Any] = ["ime_session_id": engineSession,
            "loop_sequence": String(sequence), "active": active, "target": target() as Any? ?? NSNull(),
            "composing": composing, "mode": vietnameseEnabled ? "vi" : "en", "suspended": suspended,
            "applied_revision": revision, "result": result as Any? ?? NSNull(),
            "memory": pendingMemory as Any? ?? NSNull(),
            "fault": faultCode as Any? ?? NSNull()]
        guard let bytes = try? JSONSerialization.data(withJSONObject: status),
              let text = String(data: bytes, encoding: .utf8),
              let pointer = text.withCString({ vxkey_native_tick($0) }) else {
            suspended = true; connected = false; return
        }
        defer { vxkey_native_free(pointer) }
        let data = Data(bytes: pointer, count: strlen(pointer))
        guard let delivery = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any] else {
            suspended = true; connected = false; return
        }
        let freshConnection = delivery["connected"] as? Bool ?? false
        if connected && !freshConnection { linkFault = true }
        connected = freshConnection
        latched = delivery["latched"] as? Bool ?? true
        if let config = delivery["config"] as? [String: Any] { pendingConfig = config }
        // Safe-boundary apply includes macro table and per-app policies in one tick.
        // An ACK is published only after the active engine has consumed that cache.
        if !faulted, !composing, let config = pendingConfig {
            apply(config); pendingConfig = nil
            activeController?.syncSettings()
        }
        suspended = linkFault || !connected || latched || revision == "0" || vxkey_native_engine_faulted()
        leaseDeadline = suspended ? 0 : ProcessInfo.processInfo.systemUptime + 0.5
        if let control = delivery["control"] as? [String: Any], let id = control["id"] as? String {
            var outcome = "unavailable"
            if control["action"] as? String == "suspend",
               control["ime_session_id"] as? String == engineSession,
               let requested = control["suspended"] as? Bool {
                // Resume is readiness ACK only; durable latch must clear on host
                // before a following authenticated tick can enable transformations.
                if requested { suspended = true; outcome = "applied" }
                else if connected && !composing && revision != "0" && !vxkey_native_engine_faulted() { linkFault = false; outcome = "applied" }
            } else if control["action"] as? String == "set_language" {
                outcome = "stale_target"
                if active, !suspended, !composing,
                   let expected = control["target"] as? [String: Any],
                   NSDictionary(dictionary: expected).isEqual(to: target() ?? [:]),
                   let mode = control["mode"] as? String, ["vi", "en"].contains(mode) {
                    pinVietnamese(mode == "vi", to: targetID)
                    activeController?.syncSettings(); outcome = "applied"
                }
            }
            result = ["id": id, "outcome": outcome]
        }
    }
    private func apply(_ doc: [String: Any]) {
        // Strict schema/limits/shortcut validation has already run in shared Rust.
        guard let rev = doc["revision"] as? String, let typing = doc["typing"] as? [String: Any],
              let macros = doc["macros"] as? [String: Any] else { suspended = true; return }
        let method = typing["input_method"] as? String ?? "telex"
        inputMethod = InputMethod(rawValue: method) ?? .telex
        toneStyle = typing["tone_style"] as? String == "traditional" ? .traditional : .modern
        let newMode = typing["language_mode"] as? String ?? "vi"
        if revision == "0" || savedMode != newMode { vietnameseEnabled = newMode == "vi" }
        savedMode = newMode
        smartEnglishRestore = typing["smart_restore"] as? Bool ?? true
        eagerRestore = typing["eager_restore"] as? Bool ?? true
        spellCheckEnabled = typing["spell_check"] as? Bool ?? false
        autoCapitalizeEnabled = typing["auto_capitalize"] as? Bool ?? false
        retoneAfterBackspace = typing["retone"] as? Bool ?? true
        extraOnsets = ExtraOnsetLetters(configValue: (typing["extra_onsets"] as? [String] ?? []).joined())
        directComposition = false // preedit until native app matrix accepts direct writes
        remember = doc["remember_per_app"] as? Bool ?? true
        policies = doc["input_policies"] as? [String: Bool] ?? [:]
        let nextMemory = (doc["language_memory"] as? [String: String] ?? [:]).mapValues { $0 == "vi" }
        for key in memory.keys where nextMemory[key] == nil { localMemory.removeValue(forKey: key) }
        memory = nextMemory
        if let pending = pendingMemory, let id = pending["app_id"] as? String,
           let mode = pending["mode"] as? String, memory[id] == (mode == "vi") { pendingMemory = nil }
        if !remember { localMemory = [:]; pendingMemory = nil }
        shortcutsEnabled = macros["enabled"] as? Bool ?? true
        shortcutsInEnglish = macros["in_en"] as? Bool ?? true
        shortcutSmartCase = macros["smart_case"] as? Bool ?? true
        shortcuts = (macros["entries"] as? [[String: Any]] ?? []).compactMap { entry in
            guard let trigger = entry["trigger"] as? String, let replacement = entry["replacement"] as? String else { return nil }
            return TextShortcut(trigger: trigger, expansion: replacement)
        }
        shortcutsRevision += 1
        toggleShortcut = KeyCombo.decode(doc["toggle"])
        flipShortcut = KeyCombo.decode(doc["flip"])
        revision = rev
        if let mode = resolveVietnamese(for: targetID) { vietnameseEnabled = mode }
    }
    func shouldBypass(_ client: IMKTextInput, controller: FunputInputController) -> Bool {
        let id = client.bundleIdentifier()
        return !activeInput || activeController !== controller || vxkey_native_engine_faulted() || suspended || ProcessInfo.processInfo.systemUptime >= leaseDeadline || IsSecureEventInputEnabled() || id == nil || id != targetID ||
            (id?.hasPrefix("com.vxkey.") ?? false) || (id.flatMap { policies[$0] } ?? false)
    }
    func resolveVietnamese(for id: String?) -> Bool? {
        guard let id, !id.hasPrefix("com.vxkey."), remember else { return nil }
        return localMemory[id] ?? memory[id]
    }
    func pinVietnamese(_ value: Bool, to id: String?) {
        vietnameseEnabled = value
        if remember, let id, !id.hasPrefix("com.vxkey.") {
            localMemory[id] = value
            pendingMemory = ["app_id": id, "mode": value ? "vi" : "en", "base_revision": revision]
        }
    }
}
