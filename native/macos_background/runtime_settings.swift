// Background driver cache. Real Funput composer remains behind the Rust C ABI.
import AppKit
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
        guard combo.kind=="modifier_chord" else{return false}
        guard let family=names[event.keyCode] else { cancelled=true; armed=false; return false }
        let familyDown=KeyCombo.families(event.modifierFlags).contains(family)
        let pressed=familyDown && !down.contains(event.keyCode);let hadDown = !down.isEmpty
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
    private var foregroundMatches = false
    private var targetID: String?
    private var targetPID: Int32?
    private var timer: Timer?
    private var focusObserver: NSObjectProtocol?
    private var connected = false
    private var linkFault = false
    private var latched = true
    private var leaseDeadline: TimeInterval = 0
    #if VXKEY_SESSION_ADAPTER
    private var streamPolicy = StreamRunPolicy()
    #endif
    var driver: EventDriver?
    var ready: Bool { driver?.permissionsReady == true }
    var targetPIDValue: Int32? { targetPID }
    var targetIDValue: String? { targetID }
    #if VXKEY_SESSION_ADAPTER
    var composing: Bool { driver?.hasComposition == true }
    #else
    var composing: Bool { driver?.hasUnverifiedEvents == true || !(driver?.composer.buffer().isEmpty ?? true) }
    #endif
    var dataRoot: URL {
        if let path = ProcessInfo.processInfo.environment["VXKEY_DATA_DIR"], path.hasPrefix("/") {
            return URL(fileURLWithPath: path)
        }
        return FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("VXKey", isDirectory: true)
    }
    func start() {
        #if VXKEY_SESSION_ADAPTER
        guard dataRoot.path.withCString({ vxkey_native_start_session($0) }) else { return }
        #else
        guard dataRoot.path.withCString({ vxkey_native_start($0) }) else { return }
        #endif
        focusObserver=NSWorkspace.shared.notificationCenter.addObserver(forName:NSWorkspace.didActivateApplicationNotification,object:nil,queue:.main){[weak self] _ in self?.refreshTarget()}
        tick()
        let timer = Timer(timeInterval: 0.1, repeats: true) { [weak self] _ in self?.tick() }
        self.timer = timer
        RunLoop.main.add(timer, forMode: .common)
    }
    private func target() -> [String: Any]? {
        guard let id = targetID, let pid = targetPID else { return nil }
        return ["app_id": id, "pid": pid, "engine_session_id": engineSession,
                "focus_generation": String(generation)]
    }
    func updateExternalTarget(id: String?, pid: Int32?) {
        guard let id, let pid, pid > 0, !id.hasPrefix("com.vxkey.") else { if foregroundMatches { driver?.reset() }; foregroundMatches=false; return }
        let returningToTarget = !foregroundMatches
        foregroundMatches=true
        if targetID != id || targetPID != pid {
            generation &+= 1
            targetID=id; targetPID=pid
            if let mode=resolveVietnamese(for:id) { vietnameseEnabled=mode }
            // Publish identity before priming the worker for the first keystroke.
            driver?.reset(); driver?.syncSettings(); driver?.targetChanged()
        } else if returningToTarget {
            // A sticky negative proof from our own UI must not poison the
            // unchanged external PID when focus returns to it.
            driver?.reset()
        }
    }
    private func refreshTarget() {
        let front=NSWorkspace.shared.frontmostApplication
        updateExternalTarget(id:front?.bundleIdentifier,pid:front?.processIdentifier)
    }
    private func tick() {
        precondition(Thread.isMainThread)
        driver?.verifyPending()
        refreshTarget()
        sequence &+= 1
        let faulted = vxkey_native_engine_faulted()
        let composing = !faulted && self.composing
        let active = target() != nil
        #if VXKEY_SESSION_ADAPTER
        let faultCode: String? = faulted ? "engine_fault" : (driver?.failed == true ? "driver_fault" : (driver?.permissionDenied == true ? "permission_required" : nil))
        #else
        let faultCode: String? = faulted ? "engine_fault" : (driver?.failed == true ? "driver_fault" : (!ready ? "permission_required" : (linkFault ? "bridge_stale" : nil)))
        #endif
        let status: [String: Any] = ["diagnostics": ProcessInfo.processInfo.environment["VXKEY_BRIDGE_DEV_TRUST"] == "1" ? driver?.diagnostics as Any? ?? NSNull() : NSNull(), "driver": driver?.permissionStatus ?? ["accessibility": false, "listen_access": false, "post_access": false, "attached": false], "engine_session_id": engineSession,
            "loop_sequence": String(sequence), "active": active, "target": target() as Any? ?? NSNull(),
            "composing": composing, "mode": vietnameseEnabled ? "vi" : "en", "suspended": suspended,
            "applied_revision": revision, "result": result as Any? ?? NSNull(),
            "memory": pendingMemory as Any? ?? NSNull(),
            "fault": faultCode as Any? ?? NSNull()]
        guard let bytes = try? JSONSerialization.data(withJSONObject: status),
              let text = String(data: bytes, encoding: .utf8),
              let pointer = text.withCString({ vxkey_native_tick($0) }) else {
            #if VXKEY_SESSION_ADAPTER
            streamPolicy.fault()
            #endif
            linkFault=true; suspended = true; connected = false; driver?.setRunning(false); return
        }
        defer { vxkey_native_free(pointer) }
        let data = Data(bytes: pointer, count: strlen(pointer))
        guard let delivery = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any] else {
            #if VXKEY_SESSION_ADAPTER
            streamPolicy.fault()
            #endif
            linkFault=true; suspended = true; connected = false; driver?.setRunning(false); return
        }
        if delivery["busy"] as? Bool == true {
            // A try-lock miss is not a disconnect. Keep only the already-issued
            // short lease; sustained contention still expires to safe bypass.
            #if !VXKEY_SESSION_ADAPTER
            if !suspended && ProcessInfo.processInfo.systemUptime >= leaseDeadline {
                linkFault=true; suspended=true; driver?.setRunning(false)
            }
            #endif
            return
        }
        let freshConnection = delivery["connected"] as? Bool ?? false
        if connected && !freshConnection { linkFault = true }
        connected = freshConnection
        latched = delivery["latched"] as? Bool ?? true
        #if VXKEY_SESSION_ADAPTER
        streamPolicy.observe(connected:connected,latched:latched)
        #endif
        if let config = delivery["config"] as? [String: Any] { pendingConfig = config }
        // Safe-boundary apply includes macro table and per-app policies in one tick.
        // An ACK is published only after the active engine has consumed that cache.
        if !faulted, !composing, let config = pendingConfig {
            pendingConfig = nil
            if config["revision"] as? String != revision { apply(config) }
            driver?.syncSettings()
        }
        #if VXKEY_SESSION_ADAPTER
        suspended = !streamPolicy.canRun(configured:revision != "0",permission:ready,
            driverFault:driver?.failed == true,engineFault:vxkey_native_engine_faulted())
        #else
        suspended = linkFault || !connected || latched || revision == "0" || !ready || driver?.failed == true || vxkey_native_engine_faulted()
        leaseDeadline = suspended ? 0 : ProcessInfo.processInfo.systemUptime + 0.5
        #endif
        if let control = delivery["control"] as? [String: Any], let id = control["id"] as? String {
            var outcome = "unavailable"
            if control["action"] as? String == "request_permissions", control["engine_session_id"] as? String == engineSession {
                let options = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true] as CFDictionary
                let axTrusted = AXIsProcessTrustedWithOptions(options)
                #if VXKEY_SESSION_ADAPTER
                _ = axTrusted
                if let url = URL(string:"x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility") {
                    NSWorkspace.shared.open(url)
                }
                #else
                let targetURL = axTrusted && (!CGPreflightListenEventAccess() || !CGPreflightPostEventAccess())
                    ? "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"
                    : "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
                if let url = URL(string: targetURL) {
                    NSWorkspace.shared.open(url)
                }
                // ListenEvent and PostEvent are separate TCC services. Request only from this
                // explicit action and keep any OS prompt off the input run loop.
                DispatchQueue.global(qos: .userInitiated).async {
                    if !CGPreflightListenEventAccess() { _ = CGRequestListenEventAccess() }
                    if !CGPreflightPostEventAccess() { _ = CGRequestPostEventAccess() }
                }
                #endif
                outcome = "applied" // request delivered, not permission granted
            } else if control["action"] as? String == "suspend",
               control["engine_session_id"] as? String == engineSession,
               let requested = control["suspended"] as? Bool {
                // Resume is readiness ACK only; durable latch must clear on host
                // before a following authenticated tick can enable transformations.
                if requested {
                    #if VXKEY_SESSION_ADAPTER
                    streamPolicy.pause()
                    #endif
                    suspended = true; driver?.setRunning(false); outcome = "applied"
                }
                else if connected && !composing && revision != "0" && !vxkey_native_engine_faulted(), driver?.prepare() == true {
                    #if VXKEY_SESSION_ADAPTER
                    streamPolicy.preparedResume()
                    #endif
                    linkFault = false; outcome = "applied"
                }
            } else if control["action"] as? String == "set_language" {
                outcome = "stale_target"
                if active, !suspended, !composing,
                   let expected = control["target"] as? [String: Any],
                   NSDictionary(dictionary: expected).isEqual(to: target() ?? [:]),
                   let mode = control["mode"] as? String, ["vi", "en"].contains(mode) {
                    pinVietnamese(mode == "vi", to: targetID)
                    driver?.syncSettings(); outcome = "applied"
                }
            }
            result = ["id": id, "outcome": outcome]
        }
        driver?.setRunning(!suspended)
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
        directComposition = false // legacy storage field; background driver uses verified event edits
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
    func shouldBypass() -> Bool {
        #if VXKEY_SESSION_ADAPTER
        let expired = false
        #else
        let expired = ProcessInfo.processInfo.systemUptime >= leaseDeadline
        #endif
        return vxkey_native_engine_faulted() || suspended || expired || IsSecureEventInputEnabled() || targetID == nil ||
            !foregroundMatches ||
            (targetID?.hasPrefix("com.vxkey.") ?? true) || (targetID.flatMap { policies[$0] } ?? false)
    }
    func driverFailed() {
        #if VXKEY_SESSION_ADAPTER
        streamPolicy.fault()
        #endif
        suspended = true; driver?.setRunning(false)
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
