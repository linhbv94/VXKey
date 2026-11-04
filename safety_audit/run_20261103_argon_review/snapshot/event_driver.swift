import AppKit
import Carbon
import ApplicationServices
import os

final class EventDriver {
    let composer = FunputComposer()
    // QA-only isolated TextEdit probe. Read once at launch, never in a callback.
    private let filterProbe: Bool = {
        guard ProcessInfo.processInfo.environment["VXKEY_BRIDGE_DEV_TRUST"] == "1" else { return false }
        let url=AppSettings.shared.dataRoot.appendingPathComponent("driver_probe_mode")
        return (try? String(contentsOf:url,encoding:.utf8).trimmingCharacters(in:.whitespacesAndNewlines)) == "filter"
    }()
    // Per-PID native probe on this machine honored in-place mutation but not
    // a nil callback return. Neutralize the original event in-place as well.
    static func suppress(_ event: CGEvent) -> Unmanaged<CGEvent> {
        event.type = .null
        return Unmanaged.passUnretained(event)
    }
    static func probeEvent(_ event: CGEvent) -> Unmanaged<CGEvent>? {
        let code=event.getIntegerValueField(.keyboardEventKeycode)
        if code == 6 { return suppress(event) } // z: neutralize in-place, no injection
        if code == 7 {
            let units=Array("X".utf16)
            event.keyboardSetUnicodeString(stringLength:units.count,unicodeString:units)
        }
        return Unmanaged.passUnretained(event)
    }
    private let context = ContextGuard()
    private let permissions = PermissionCache()
    private var tapEnabled=false
    final class TapContext {
        weak var driver: EventDriver?
        let generation: UInt64
        init(_ driver: EventDriver, _ generation: UInt64) { self.driver=driver; self.generation=generation }
    }
    private var tapGeneration: UInt64 = 0
    private var tapContext: TapContext?
    private let logger = Logger(subsystem: "com.vxkey.engine", category: "event_driver")
    private var tap: CFMachPort?
    private var source: CFRunLoopSource?
    private var running = false
    private var requested = false
    private var tapPID: Int32?
    private(set) var failed = false
    private var owned = ""
    private var start = -1
    private var epoch: UInt64 = 0
    private var window = StreamWindow()
    private var bypassWord = false
    var hasUnverifiedEvents: Bool { window.isPending }
    private var consumed = Set<UInt16>()
    private let gesture = NativeModifierGesture()
    private let nonce = Int64.random(in: 1...Int64.max)
    private var configuredRevision = -1
    private var counts: [String:UInt32] = ["seen":0,"bypassed":0,"missing_target":0,"unverified":0,"rewrites":0,"verified":0,"failed":0]
    var diagnostics: [String:UInt32] { counts }
    private func note(_ key:String) { counts[key] = min(1_000_000,(counts[key] ?? 0)+1) }
    var permissionStatus: [String: Bool] { let p=permissions.current; return ["accessibility":p.accessibility,"listen_access":p.listen,"post_access":p.post,"attached":attached] }
    var attached: Bool { running && tapEnabled }
    var permissionsReady: Bool { permissions.ready }
    func prepare() -> Bool {
        guard permissionsReady else { return false }
        guard let pid=AppSettings.shared.targetPIDValue else { failed=false; return true }
        if tap != nil && tapPID == pid { failed = false; return true }
        removeTap()
        let mask = [CGEventType.keyDown, .keyUp, .flagsChanged, .leftMouseDown, .rightMouseDown, .otherMouseDown]
            .reduce(CGEventMask(0)) { $0 | (CGEventMask(1) << $1.rawValue) }
        let callbackContext=TapContext(self,tapGeneration)
        guard let tap = CGEvent.tapCreateForPid(pid: pid, place: .headInsertEventTap,
            options: .defaultTap, eventsOfInterest: mask, callback: { proxy, type, event, pointer in
                guard let pointer else { return Unmanaged.passUnretained(event) }
                let context=Unmanaged<TapContext>.fromOpaque(pointer).takeUnretainedValue()
                return context.driver?.handle(type,event,generation:context.generation,proxy:proxy) ?? Unmanaged.passUnretained(event)
            }, userInfo: Unmanaged.passUnretained(callbackContext).toOpaque()),
              let source = CFMachPortCreateRunLoopSource(nil, tap, 0) else { logger.error("tap_prepare_failed"); return false }
        self.tapContext=callbackContext
        self.tap = tap; self.source = source; self.tapPID=pid
        // Newly created taps are enabled. Readiness callbacks pass through
        // while running is false. Disabling here queues a delayed notification
        // that can otherwise fault the next Resume.
        CFRunLoopAddSource(CFRunLoopGetMain(), source, .commonModes)
        failed = false
        return true
    }
    private func removeTap() {
        running=false; tapEnabled=false
        bypassWord=false
        consumed.removeAll(); gesture.reset(); context.stop()
        tapGeneration &+= 1
        if let tap { CFMachPortInvalidate(tap) }
        if let source { CFRunLoopRemoveSource(CFRunLoopGetMain(),source,.commonModes) }
        tap=nil; source=nil; tapPID=nil; tapContext=nil
    }
    func targetChanged() {
        let resume=requested
        removeTap()
        if resume { setRunning(true) }
    }
    func setRunning(_ enabled: Bool) {
        requested=enabled
        if !enabled { if running { reset() }; removeTap(); return }
        let allowed=enabled && permissionsReady && !failed
        if allowed, tapPID != AppSettings.shared.targetPIDValue || tap == nil {
            guard prepare() else { fault(); return }
        }
        let next=allowed && tap != nil
        if next != running { reset(); running=next; if let tap { CGEvent.tapEnable(tap:tap,enable:next); tapEnabled=CGEvent.tapIsEnabled(tap:tap) } }
    }
    func reset(immediate: Bool = true) {
        if !vxkey_native_engine_faulted() { composer.clear() }
        owned = ""; start = -1; bypassWord=false; window = StreamWindow(); epoch &+= 1
        if let pid = AppSettings.shared.targetPIDValue { context.expect(pid: pid, text: "", start: -1, epoch: epoch, immediate: immediate) }
    }
    func syncSettings() {
        guard !vxkey_native_engine_faulted() else { return }
        let s = AppSettings.shared
        composer.apply(ComposerConfiguration(settings: s))
        if configuredRevision != s.shortcutsRevision {
            composer.clearShortcuts()
            for shortcut in s.shortcuts { composer.addShortcut(trigger: shortcut.trigger, expansion: shortcut.expansion) }
            configuredRevision = s.shortcutsRevision
        }
    }
    private func expectOwned() {
        guard let pid=AppSettings.shared.targetPIDValue else { return }
        let revision=context.expect(pid:pid,text:owned,start:start,epoch:epoch,immediate:false)
        window.record(revision,at:ProcessInfo.processInfo.systemUptime)
    }
    private func abandonWord() {
        reset(); bypassWord=true; note("unverified")
    }
    func verifyPending() {
        if running {
            tapEnabled=tap.map { CGEvent.tapIsEnabled(tap:$0) } == true
            if !tapEnabled { fault("tap_not_enabled"); return }
        }
        guard let pid=AppSettings.shared.targetPIDValue else { return }
        if !window.isPending {
            if running, owned.isEmpty, composer.buffer().isEmpty, context.isRevoked(pid:pid,epoch:epoch) {
                reset()
            }
            return
        }
        if let s=context.latest(pid:pid,epoch:epoch), s.verified,
           window.acknowledge(s.revision) {
            if start < 0 && s.start >= 0 { start = s.start }
            note("verified")
        }
        if !window.allows(at:ProcessInfo.processInfo.systemUptime) { abandonWord() }
    }
    private func fault(_ reason: String = "edit_or_prepare_failed") {
        logger.error("driver_fault reason=\(reason, privacy: .public)")
        note("failed"); failed = true; setRunning(false); AppSettings.shared.driverFailed()
    }
    func batch(_ plan: EditPlan) -> [CGEvent]? {
        var events: [CGEvent] = []
        // Build every allocation before swallowing the original key. Unicode is
        // emitted in bounded UTF-16 chunks; surrogate pairs are never split.
        for _ in 0..<plan.backspaces {
            for down in [true, false] {
                guard let e = CGEvent(keyboardEventSource: nil, virtualKey: 51, keyDown: down) else { return nil }
                events.append(e)
            }
        }
        for units in EditPlan.unicodeChunks(plan.output) {
            for down in [true, false] {
                guard let e = CGEvent(keyboardEventSource: nil, virtualKey: 0, keyDown: down) else { return nil }
                e.keyboardSetUnicodeString(stringLength: units.count, unicodeString: units)
                events.append(e)
            }
        }
        guard events.count <= 1100 else { return nil }
        for e in events { e.flags = []; e.setIntegerValueField(.eventSourceUserData, value: nonce) }
        return events
    }
    func handle(_ type: CGEventType, _ event: CGEvent, generation: UInt64, proxy: CGEventTapProxy? = nil) -> Unmanaged<CGEvent>? {
        guard generation == tapGeneration else { return Unmanaged.passUnretained(event) }
        return handle(type,event,proxy:proxy)
    }
    func handle(_ type: CGEventType, _ event: CGEvent, proxy: CGEventTapProxy? = nil) -> Unmanaged<CGEvent>? {
        let pass = Unmanaged.passUnretained(event)
        if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput { if running { fault(type == .tapDisabledByTimeout ? "tap_timeout" : "tap_disabled_by_input") }; return pass }
        if event.getIntegerValueField(.eventSourceUserData) == nonce && event.getIntegerValueField(.eventSourceUnixProcessID) == Int64(getpid()) { return pass }
        if type == .keyDown { note("seen") }
        let code = UInt16(event.getIntegerValueField(.keyboardEventKeycode))
        if type == .keyUp, consumed.remove(code) != nil { return Self.suppress(event) }
        let settings = AppSettings.shared
        guard running, !settings.shouldBypass(), let pid = settings.targetPIDValue else { if type == .keyDown { note("bypassed") }; reset(); gesture.reset(); return pass }
        guard tapPID == pid else { if type == .keyDown { note("missing_target") }; reset(); gesture.reset(); return pass }
        if type == .flagsChanged {
            if let ns = NSEvent(cgEvent: event), let combo = settings.toggleShortcut, gesture.handle(ns, combo: combo) {
                reset(); settings.pinVietnamese(!settings.vietnameseEnabled, to: settings.targetIDValue); syncSettings()
            }
            return pass
        }
        guard type == .keyDown else { if type != .keyUp { reset(immediate: false); gesture.reset() }; return pass }
        gesture.nonModifier()
        if let ns = NSEvent(cgEvent: event), settings.toggleShortcut?.matches(ns) == true {
            if event.getIntegerValueField(.keyboardEventAutorepeat) != 0 { consumed.insert(code); return Self.suppress(event) }
            reset(); settings.pinVietnamese(!settings.vietnameseEnabled, to: settings.targetIDValue); syncSettings(); consumed.insert(code); return Self.suppress(event)
        }
        guard event.flags.intersection([.maskCommand, .maskControl, .maskAlternate]).isEmpty else { reset(immediate: false); return pass }
        if filterProbe {
            // Restrict diagnostic behavior to a collapsed, fresh TextEdit field.
            // No composer, AX writes, pasteboard, or synthetic posting in probe.
            guard settings.targetIDValue == "com.apple.TextEdit",
                  let proof=context.latest(pid:pid,epoch:epoch), proof.verified else { return pass }
            let result=Self.probeEvent(event)
            if code == 6 || code == 7 { note("rewrites") }
            if code == 6 { consumed.insert(code) }
            return result
        }
        // Navigation terminates ownership before checking any pending edit.
        if [36,48,53,115,116,117,119,121,123,124,125,126].contains(Int(code)) {
            reset(immediate: false); bypassWord=false; return pass
        }
        if bypassWord {
            var units=[UniChar](repeating:0,count:2); var n=0
            event.keyboardGetUnicodeString(maxStringLength:2,actualStringLength:&n,unicodeString:&units)
            if n==1, units[0]==32 { reset(immediate: false); bypassWord=false }
            return pass
        }
        guard let proxy else { reset(); return pass }
        if code == 51 {
            guard settings.retoneAfterBackspace, !owned.isEmpty,
                  let snapshot = context.latest(pid: pid, epoch: epoch), snapshot.verified,
                  snapshot.text == owned, snapshot.caret == start + owned.utf16.count,
                  EditPlan.make(owned: owned, backspaces: 1, output: "") != nil else { reset(immediate: false); return pass }
            if owned.last == " ", composer.buffer().isEmpty {
                owned.removeLast()
                guard composer.adopt(owned) else { reset(immediate: false); return pass }
            } else {
                let response = composer.backspace()
                if response.action != ACTION_NONE {
                    guard response.count < CHARS_CAP,
                          let plan = EditPlan.make(owned: owned, backspaces: Int(response.backspace), output: FunputComposer.output(of: response)),
                          let events = batch(plan) else { reset(immediate: false); return pass }
                    owned = plan.resultingText; expectOwned()
                    consumed.insert(code); note("rewrites")
                    for e in events { e.tapPostEvent(proxy) }; return Self.suppress(event)
                }
                owned.removeLast()
            }
            if owned.isEmpty { reset(immediate: false) } else { expectOwned() }
            return pass
        }
        var chars = [UniChar](repeating: 0, count: 8); var count = 0
        event.keyboardGetUnicodeString(maxStringLength: chars.count, actualStringLength: &count, unicodeString: &chars)
        guard count == 1, let scalar = Unicode.Scalar(UInt32(chars[0])), scalar.value >= 32, scalar.value <= 126 else { reset(immediate: false); return pass }
        let now = ProcessInfo.processInfo.systemUptime
        let snapshot = context.latest(pid: pid, epoch: epoch)
        if let snapshot, snapshot.verified {
            _ = window.acknowledge(snapshot.revision)
        } else if owned.isEmpty, composer.buffer().isEmpty, context.isRevoked(pid: pid, epoch: epoch) {
            start = -1; epoch &+= 1; window = StreamWindow()
            context.expect(pid: pid, text: "", start: -1, epoch: epoch, immediate: false)
        }
        guard !context.isRevoked(pid: pid, epoch: epoch), window.allows(at: now) else {
            abandonWord(); return pass
        }
        if owned.last == " ", composer.buffer().isEmpty {
            if let snapshot, snapshot.verified, !window.isPending { start = snapshot.caret }
            else if start >= 0 { start += owned.utf16.count }
            owned = ""
        }
        if start < 0, let snapshot, snapshot.verified, snapshot.start >= 0 {
            start = snapshot.start
        }
        let result = composer.processText(scalar, source: event.flags.contains(.maskNumericPad) ? .numpad : .standard)
        if vxkey_native_engine_faulted() { fault(); return pass }
        if result.result.action == ACTION_NONE {
            owned.append(Character(String(scalar)))
            if owned.utf16.count > 128 || owned.hasSuffix("  ") { reset(immediate: false) }
            else { expectOwned() }
            return pass
        }
        let resolvedStart = start >= 0 ? start : (snapshot.map { $0.start >= 0 ? $0.start : $0.caret } ?? -1)
        guard let snapshot, snapshot.verified, resolvedStart >= 0 else {
            abandonWord(); return pass
        }
        guard let plan = EditPlan.make(owned: owned, backspaces: Int(result.result.backspace), output: result.output),
              let events = batch(plan) else { reset(); return pass }
        owned = plan.resultingText
        expectOwned()
        consumed.insert(code); note("rewrites")
        for e in events { e.tapPostEvent(proxy) }
        return Self.suppress(event)
    }
    deinit {
        if let tap { CFMachPortInvalidate(tap) }
        if let source { CFRunLoopRemoveSource(CFRunLoopGetMain(), source, .commonModes) }
    }
}
