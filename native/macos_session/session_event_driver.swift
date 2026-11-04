#if VXKEY_SESSION_ADAPTER
import AppKit
import Carbon
import ApplicationServices
import os

/// Integrated session driver. Keyboard data stays in-process. Target/config are
/// cache-only in callbacks; no AX text proof, companion lease, disk or IPC here.
final class EventDriver {
    private let composition = StreamComposition()
    var composer: FunputComposer { composition.composer }
    var hasUnverifiedEvents: Bool { false }
    var hasComposition: Bool { composition.isComposing }
    private let permissions = PermissionCache(requirePost: false, expireObservation: false)
    private let eventSource = CGEventSource(stateID: .privateState)
    private let nonce = Int64.random(in: 1...Int64.max)
    private let gesture = NativeModifierGesture()
    private var consumed = Set<UInt16>()
    private var tap: CFMachPort?
    private var source: CFRunLoopSource?
    private var running = false
    private var tapEnabled = false
    private var layoutObserver: NSObjectProtocol?
    private(set) var failed = false
    private var configured: ComposerConfiguration?
    private var shortcutRevision = -1
    private let logger = Logger(subsystem: "com.vxkey.engine", category: "session_driver")
    private var counts: [String: UInt32] = ["seen":0,"bypassed":0,"missing_target":0,"unverified":0,"rewrites":0,"verified":0,"failed":0]
    var diagnostics: [String: UInt32] { counts }
    private func note(_ key: String) { counts[key] = min(1_000_000, (counts[key] ?? 0) + 1) }
    var attached: Bool { running && tapEnabled }
    var permissionsReady: Bool { permissions.ready }
    var permissionDenied: Bool { permissions.denied }
    var permissionStatus: [String: Bool] {
        let p = permissions.current
        return ["accessibility":p.accessibility,"listen_access":p.listen,"post_access":p.post,"attached":attached]
    }
    init() {
        layoutObserver = DistributedNotificationCenter.default().addObserver(
            forName: Notification.Name("com.apple.Carbon.TISNotifySelectedKeyboardInputSourceChanged"),
            object: nil, queue: .main) { [weak self] _ in self?.reset() }
    }
    func prepare() -> Bool {
        guard permissionsReady, eventSource != nil else { return false }
        if tap != nil { failed = false; return true }
        let mask = [CGEventType.keyDown,.keyUp,.flagsChanged,.leftMouseDown,.rightMouseDown,.otherMouseDown,
                    .leftMouseDragged,.rightMouseDragged,.otherMouseDragged,.scrollWheel]
            .reduce(CGEventMask(0)) { $0 | (CGEventMask(1) << $1.rawValue) }
        guard let port = CGEvent.tapCreate(tap: .cgSessionEventTap, place: .headInsertEventTap,
            options: .defaultTap, eventsOfInterest: mask, callback: { proxy,type,event,pointer in
                guard let pointer else { return Unmanaged.passUnretained(event) }
                return Unmanaged<EventDriver>.fromOpaque(pointer).takeUnretainedValue().handle(type,event,proxy:proxy)
            }, userInfo: Unmanaged.passUnretained(self).toOpaque()) else { return false }
        guard let runSource = CFMachPortCreateRunLoopSource(nil, port, 0) else { CFMachPortInvalidate(port); return false }
        tap = port; source = runSource; failed = false
        CFRunLoopAddSource(CFRunLoopGetMain(),runSource,.commonModes)
        return true
    }
    func targetChanged() { reset() } // One session tap, not rebuilt per application.
    func setRunning(_ enabled: Bool) {
        if !enabled { detach(); return }
        guard !failed, permissionsReady else { detach(); return }
        guard tap != nil || prepare() else { fault("tap_prepare_failed"); return }
        if !running { reset(); running = true }
        if let tap { CGEvent.tapEnable(tap:tap,enable:true); tapEnabled = CGEvent.tapIsEnabled(tap:tap) }
        if !tapEnabled { fault("tap_not_enabled") }
    }
    private func detach() {
        running = false; tapEnabled = false; reset(); consumed.removeAll()
        gesture.reset()
        if let tap { CFMachPortInvalidate(tap) }
        if let source { CFRunLoopRemoveSource(CFRunLoopGetMain(),source,.commonModes) }
        tap = nil; source = nil
    }
    func reset() { composition.reset() }
    func syncSettings() {
        guard !vxkey_native_engine_faulted() else { return }
        let settings = AppSettings.shared
        let next = ComposerConfiguration(settings: settings)
        if configured != next || shortcutRevision != settings.shortcutsRevision {
            composition.configure(next, shortcuts: settings.shortcuts)
            configured = next; shortcutRevision = settings.shortcutsRevision
        }
    }
    func verifyPending() {
        if running && tap.map({ CGEvent.tapIsEnabled(tap:$0) }) != true { fault("tap_not_enabled") }
    }
    private func fault(_ reason: String) {
        logger.error("driver_fault reason=\(reason, privacy: .public)")
        note("failed"); failed = true; detach(); AppSettings.shared.driverFailed()
    }
    private func deliver(_ decision: StreamComposition.Decision, event: CGEvent, proxy: CGEventTapProxy?) -> Unmanaged<CGEvent>? {
        let pass = Unmanaged.passUnretained(event)
        switch decision {
        case .pass: return pass
        case .rejected: reset(); return pass
        case .replace(let plan):
            guard let proxy, let delivery = StreamDelivery.prepare(plan, original:event, source:eventSource, nonce:nonce) else {
                reset(); return pass
            }
            if !delivery.suppressOriginal { delivery.apply(to:event,nonce:nonce) }
            note("rewrites")
            for deletion in delivery.deletions { deletion.tapPostEvent(proxy) }
            if delivery.suppressOriginal {
                consumed.insert(UInt16(clamping:event.getIntegerValueField(.keyboardEventKeycode)))
                return nil
            }
            return pass
        }
    }
    func handle(_ type: CGEventType, _ event: CGEvent, proxy: CGEventTapProxy?) -> Unmanaged<CGEvent>? {
        let pass = Unmanaged.passUnretained(event)
        if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
            if running { fault(type == .tapDisabledByTimeout ? "tap_timeout" : "tap_disabled") }
            return pass
        }
        if event.getIntegerValueField(.eventSourceUserData) == nonce &&
           event.getIntegerValueField(.eventSourceUnixProcessID) == Int64(getpid()) { return pass }
        let code = UInt16(clamping:event.getIntegerValueField(.keyboardEventKeycode))
        if type == .keyUp, consumed.remove(code) != nil { return nil }
        let settings = AppSettings.shared
        guard running, !settings.shouldBypass(), let pid = settings.targetPIDValue else {
            if type == .keyDown { note("bypassed") }; reset(); gesture.reset(); return pass
        }
        let destination = event.getIntegerValueField(.eventTargetUnixProcessID)
        guard destination <= 0 || destination == Int64(pid) else { reset(); note("missing_target"); return pass }
        if type == .flagsChanged {
            guard let ns = NSEvent(cgEvent:event) else { reset(); return pass }
            if let toggle = settings.toggleShortcut, gesture.handle(ns,combo:toggle) {
                reset(); settings.pinVietnamese(!settings.vietnameseEnabled,to:settings.targetIDValue); syncSettings()
            }
            return pass
        }
        guard type == .keyDown else {
            if type != .keyUp { reset(); gesture.reset() }
            return pass
        }
        note("seen"); gesture.nonModifier()
        if let ns = NSEvent(cgEvent:event), settings.toggleShortcut?.matches(ns) == true {
            if event.getIntegerValueField(.keyboardEventAutorepeat) == 0 {
                reset(); settings.pinVietnamese(!settings.vietnameseEnabled,to:settings.targetIDValue); syncSettings()
            }
            consumed.insert(code); return nil
        }
        if let ns = NSEvent(cgEvent:event), settings.flipShortcut?.matches(ns) == true {
            if event.getIntegerValueField(.keyboardEventAutorepeat) != 0 { consumed.insert(code); return nil }
            let decision = composition.flip()
            if case .replace(let plan) = decision {
                guard let proxy, let batch = StreamDelivery.prepareStandalone(plan, source:eventSource, nonce:nonce) else {
                    reset(); consumed.insert(code); return nil
                }
                note("rewrites")
                for e in batch { e.tapPostEvent(proxy) }
                consumed.insert(code); return nil
            }
            consumed.insert(code); return nil
        }
        guard event.flags.intersection([.maskCommand,.maskControl,.maskAlternate,.maskSecondaryFn]).isEmpty else { reset(); return pass }
        if event.getIntegerValueField(.keyboardEventAutorepeat) != 0 { reset(); return pass }
        if [36,48,53,76,115,116,117,119,121,123,124,125,126].contains(Int(code)) { reset(); return pass }
        if code == 51 { return deliver(composition.backspace(retone:settings.retoneAfterBackspace),event:event,proxy:proxy) }
        var units = [UniChar](repeating:0,count:8); var count = 0
        event.keyboardGetUnicodeString(maxStringLength:units.count,actualStringLength:&count,unicodeString:&units)
        guard count == 1, let scalar = Unicode.Scalar(units[0]) else { reset(); return pass }
        return deliver(composition.feed(scalar,source:event.flags.contains(.maskNumericPad) ? .numpad : .standard),event:event,proxy:proxy)
    }
    deinit {
        if let layoutObserver { DistributedNotificationCenter.default().removeObserver(layoutObserver) }
        if let tap { CFMachPortInvalidate(tap) }
        if let source { CFRunLoopRemoveSource(CFRunLoopGetMain(),source,.commonModes) }
    }
}
#endif
