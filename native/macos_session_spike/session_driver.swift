import AppKit
import Carbon
import ApplicationServices

/// Isolated P1 harness, never selected by the production build. One process owns
/// tap, composer and mouse-accessible Pause/Quit. No companion or heartbeat.
final class SessionSpikeDriver {
    enum Mode: String { case filter, compose, directUnicode, postUnicode, replaceUnicode, transportMatrix, engineFixture }
    private let composition = SessionComposition()
    private var tap: CFMachPort?
    private var source: CFRunLoopSource?
    private var consumed = Set<UInt16>()
    private var targetPID: pid_t?
    private var targetID: String?
    private let nonce = Int64.random(in: 1...Int64.max)
    private let privateSource = CGEventSource(stateID: .privateState)
    private(set) var usePrivateSource = true
    private(set) var returnOriginal = true
    private let fixedTrace = FixedTransportTrace()
    private var fixedSeed = false
    private var fixedStep = 0
    private static let fixtureKeys = Array("as af aa".unicodeScalars)
    private var fixtureIndex = 0
    private var fixtureAbort: String?
    private var fixtureRecords: [[String: Any]] = []
    private var fixtureOracle: [[String: Any]] = []
    private var focusObserver: NSObjectProtocol?
    private var layoutObserver: NSObjectProtocol?
    private var permissionTimer: Timer?
    private let permissionWorker = DispatchQueue(label: "com.vxkey.spike.permissions")
    private var permissionInFlight = false
    private var accessibility = false
    private var postAccess = false
    private var listenAccess = false
    private var permissionTime: TimeInterval = 0
    private(set) var running = false
    private(set) var mode: Mode = .filter
    private(set) var reason = "paused"
    private(set) var seen: UInt64 = 0
    private(set) var replacements: UInt64 = 0
    private(set) var suppressed: UInt64 = 0
    private var durations: [UInt64] = []
    var changed: (() -> Void)?
    // QA scope, not a production compatibility/role allowlist. User must use a
    // scratch text field in these clients; this is not a daily-driver build.
    static let testClients: Set<String> = ["com.apple.TextEdit", "com.figma.Desktop"]
    var permissionSummary: String {
        "AX=\(accessibility), post=\(postAccess), listen=\(listenAccess)"
    }
    var permissionReady: Bool {
        // A session filtering tap is attempted with Accessibility. In particular,
        // the filter probe never posts an event, so PostEvent preflight cannot be
        // its gate. Proxy posting remains a separate physical native test.
        // Delayed tccd replies do not create another typing lease.
        accessibility
    }
    private func resetStream() {
        if mode == .engineFixture && fixtureIndex > 0 && fixtureIndex < Self.fixtureKeys.count {
            fixtureAbort = "boundary_during_known_fixture"
        }
        composition.reset(); fixedSeed = false
    }
    private static func coreMetadata(_ result: FunputResult, _ output: String) -> [String: Any] {
        ["action": Int(result.action), "backspaces": Int(result.backspace), "inline_count": Int(result.count),
         "callback_output_class": FixedTransportTrace.outputClass(output),
         "inline_output_class": result.count <= CHARS_CAP ? FixedTransportTrace.outputClass(FunputComposer.output(of: result)) : "invalid_inline_count",
         "callback_utf8_count": output.utf8.count, "callback_utf16_count": output.utf16.count]
    }
    init() {
        focusObserver = NSWorkspace.shared.notificationCenter.addObserver(
            forName: NSWorkspace.didActivateApplicationNotification, object: nil, queue: .main) { [weak self] _ in
                self?.refreshTarget()
            }
        layoutObserver = DistributedNotificationCenter.default().addObserver(
            forName: Notification.Name("com.apple.Carbon.TISNotifySelectedKeyboardInputSourceChanged"),
            object: nil, queue: .main) { [weak self] _ in self?.resetStream() }
        refreshTarget()
        refreshPermissions()
        let timer = Timer(timeInterval: 1, repeats: true) { [weak self] _ in self?.refreshPermissions() }
        permissionTimer = timer
        RunLoop.main.add(timer, forMode: .common)
    }
    private func refreshTarget() {
        let app = NSWorkspace.shared.frontmostApplication
        let pid = app?.processIdentifier
        let id = app?.bundleIdentifier
        if pid != targetPID || id != targetID { resetStream() }
        targetPID = pid; targetID = id
    }
    private func refreshPermissions() {
        guard !permissionInFlight else { return }
        permissionInFlight = true
        let began = ProcessInfo.processInfo.systemUptime
        permissionWorker.async { [weak self] in
            let ax = AXIsProcessTrusted()
            let post = CGPreflightPostEventAccess()
            let listen = CGPreflightListenEventAccess()
            DispatchQueue.main.async { [weak self] in
                guard let self else { return }
                self.permissionInFlight = false
                self.accessibility = ax; self.postAccess = post; self.listenAccess = listen
                self.permissionTime = began
                if self.running && !self.permissionReady { self.pause("permission_missing") }
                self.changed?()
            }
        }
    }
    // Only an explicit menu action requests rights. Listen is observed but is
    // neither requested nor a prerequisite; tap creation is the runtime gate.
    func requestAccessibility() {
        let options = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true] as CFDictionary
        _ = AXIsProcessTrustedWithOptions(options)
        if let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility") {
            NSWorkspace.shared.open(url)
        }
        refreshPermissions()
    }
    @discardableResult func resume(_ nextMode: Mode) -> Bool {
        pause("paused")
        guard permissionReady else { reason = "permission_missing"; changed?(); return false }
        guard !vxkey_native_engine_faulted() else { reason = "engine_fault"; changed?(); return false }
        guard !usePrivateSource || privateSource != nil else { reason = "event_source_failed"; changed?(); return false }
        fixedStep = 0
        if nextMode == .engineFixture {
            fixtureIndex = 0; fixtureAbort = nil; fixtureRecords = []; fixtureOracle = []
            let oracle = SessionComposition()
            for scalar in Self.fixtureKeys {
                _ = oracle.feed(scalar, onResult: { result, output in
                    self.fixtureOracle.append(Self.coreMetadata(result, output))
                })
            }
            guard fixedTrace.start(acceptingNonce: nonce) else {
                reason = "fixture_observer_failed"; changed?(); return false
            }
        }
        if [.directUnicode, .postUnicode, .replaceUnicode, .transportMatrix].contains(nextMode), !fixedTrace.start() {
            reason = "probe_observer_failed"; changed?(); return false
        }
        let mask = [CGEventType.keyDown, .keyUp, .flagsChanged, .leftMouseDown,
                    .rightMouseDown, .otherMouseDown, .leftMouseDragged, .rightMouseDragged,
                    .otherMouseDragged, .scrollWheel].reduce(CGEventMask(0)) { $0 | 1 << $1.rawValue }
        guard let port = CGEvent.tapCreate(tap: .cgSessionEventTap, place: .headInsertEventTap,
            options: .defaultTap, eventsOfInterest: mask, callback: { proxy, type, event, pointer in
                guard let pointer else { return Unmanaged.passUnretained(event) }
                return Unmanaged<SessionSpikeDriver>.fromOpaque(pointer).takeUnretainedValue()
                    .handle(proxy: proxy, type: type, event: event)
            }, userInfo: Unmanaged.passUnretained(self).toOpaque()) else {
            fixedTrace.stop(); reason = "tap_create_failed"; changed?(); return false
        }
        guard let runSource = CFMachPortCreateRunLoopSource(nil, port, 0) else {
            CFMachPortInvalidate(port); fixedTrace.stop(); reason = "tap_source_failed"; changed?(); return false
        }
        tap = port; source = runSource; mode = nextMode; running = true; reason = "running"
        CFRunLoopAddSource(CFRunLoopGetMain(), runSource, .commonModes)
        CGEvent.tapEnable(tap: port, enable: true)
        guard CGEvent.tapIsEnabled(tap: port) else { pause("tap_not_enabled"); return false }
        refreshTarget(); changed?(); return true
    }
    func pause(_ why: String = "paused") {
        running = false; reason = why; resetStream(); consumed.removeAll(); fixedTrace.stop()
        if let tap { CFMachPortInvalidate(tap) }
        if let source { CFRunLoopRemoveSource(CFRunLoopGetMain(), source, .commonModes) }
        tap = nil; source = nil; changed?()
    }
    func setPrivateSource(_ enabled: Bool) {
        guard !running else { return }
        resetStream(); usePrivateSource = enabled; changed?()
    }
    func setReturnOriginal(_ enabled: Bool) {
        guard !running else { return }
        resetStream(); returnOriginal = enabled; changed?()
    }
    /// Preserve the physical key's identity and down/up lifecycle. Only the
    /// preceding deletions are synthetic; the Unicode travels on the original
    /// keyDown. P1 bounds one returned packet; long expansions remain a P2 gate.
    static func returnPacket(_ plan: EditPlan, on event: CGEvent, nonce: Int64,
                             eventSource: CGEventSource?) -> [CGEvent]? {
        let units = Array(plan.output.utf16)
        guard !units.isEmpty, units.count <= 20, (0...128).contains(plan.backspaces),
              let deletion = EditPlan.make(owned: String(repeating: "a", count: plan.backspaces),
                  backspaces: plan.backspaces, output: ""),
              let events = makeBatch(deletion, nonce: nonce, eventSource: eventSource) else { return nil }
        // Allocate the entire deletion batch before mutating/swallowing anything.
        event.keyboardSetUnicodeString(stringLength: units.count, unicodeString: units)
        event.setIntegerValueField(.eventSourceUserData, value: nonce)
        return events
    }
    static func isBoundary(code: UInt16, flags: CGEventFlags, repeated: Bool) -> Bool {
        repeated || !flags.intersection([.maskCommand, .maskControl, .maskAlternate, .maskSecondaryFn]).isEmpty ||
        [36, 48, 51, 53, 76, 115, 116, 117, 119, 121, 123, 124, 125, 126].contains(code)
    }
    static func filterEvent(_ event: CGEvent) -> Unmanaged<CGEvent>? {
        let code = event.getIntegerValueField(.keyboardEventKeycode)
        if code == 6 { return nil } // z, canonical session-tap suppression; verify physically.
        if code == 7 { event.keyboardSetUnicodeString(stringLength: 1, unicodeString: [88]) }
        return Unmanaged.passUnretained(event)
    }
    static func makeBatch(_ plan: EditPlan, nonce: Int64, eventSource: CGEventSource? = nil) -> [CGEvent]? {
        var events: [CGEvent] = []
        for _ in 0..<plan.backspaces {
            for down in [true, false] {
                guard let e = CGEvent(keyboardEventSource: eventSource, virtualKey: 51, keyDown: down) else { return nil }
                events.append(e)
            }
        }
        for units in EditPlan.unicodeChunks(plan.output) {
            for down in [true, false] {
                guard let e = CGEvent(keyboardEventSource: eventSource, virtualKey: 0, keyDown: down) else { return nil }
                e.keyboardSetUnicodeString(stringLength: units.count, unicodeString: units)
                events.append(e)
            }
        }
        guard events.count <= 512 else { return nil }
        for e in events { e.flags = []; e.setIntegerValueField(.eventSourceUserData, value: nonce) }
        return events
    }
    func handle(proxy: CGEventTapProxy?, type: CGEventType, event: CGEvent) -> Unmanaged<CGEvent>? {
        let began = DispatchTime.now().uptimeNanoseconds
        defer {
            // Bounded, content-free in-memory timing. No per-key logging/I/O.
            if durations.count < 8192 { durations.append(DispatchTime.now().uptimeNanoseconds - began) }
        }
        let pass = Unmanaged.passUnretained(event)
        if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
            pause(type == .tapDisabledByTimeout ? "tap_timeout" : "tap_disabled")
            return pass // P1 does not auto-resume faults.
        }
        if event.getIntegerValueField(.eventSourceUserData) == nonce &&
           event.getIntegerValueField(.eventSourceUnixProcessID) == Int64(getpid()) { return pass }
        if event.getIntegerValueField(.eventSourceUserData) == fixedTrace.nonce { return pass }
        let code = UInt16(clamping: event.getIntegerValueField(.keyboardEventKeycode))
        if type == .keyUp, consumed.remove(code) != nil { suppressed &+= 1; return nil }
        guard running, permissionReady else { resetStream(); return pass }
        guard let id = targetID, Self.testClients.contains(id), targetPID != nil,
              !IsSecureEventInputEnabled() else { resetStream(); return pass }
        // When macOS supplies a destination, reject a stale focus notification.
        // A zero destination is not document/caret proof; see P2 focus gate.
        let destination = event.getIntegerValueField(.eventTargetUnixProcessID)
        if destination > 0 && destination != Int64(targetPID!) { resetStream(); return pass }
        guard type == .keyDown || type == .keyUp else {
            if type != .flagsChanged || !event.flags.intersection([.maskCommand, .maskControl, .maskAlternate]).isEmpty {
                resetStream()
            }
            return pass
        }
        if type == .keyUp { return pass }
        seen &+= 1
        if Self.isBoundary(code: code, flags: event.flags,
            repeated: event.getIntegerValueField(.keyboardEventAutorepeat) != 0) {
            resetStream(); return pass
        }
        if mode == .filter {
            if code == 6 { consumed.insert(code); suppressed &+= 1 }
            return Self.filterEvent(event)
        }
        if mode != .compose && mode != .engineFixture {
            let hadSeed = fixedSeed
            fixedSeed = code == 0 && event.flags.intersection([.maskShift, .maskAlphaShift]).isEmpty
            guard code == 7 else { return pass }
            let probeMode: Mode
            if mode == .transportMatrix {
                guard fixedStep < 3 else { return pass }
                probeMode = [.directUnicode, .postUnicode, .replaceUnicode][fixedStep]
                fixedStep += 1
            } else { probeMode = mode }
            if probeMode == .directUnicode {
                event.keyboardSetUnicodeString(stringLength: 1, unicodeString: [0x00e1])
                event.setIntegerValueField(.eventSourceUserData, value: fixedTrace.nonce)
                fixedTrace.expect([event]); return pass
            }
            guard probeMode != .replaceUnicode || hadSeed else { return pass }
            let deletes = probeMode == .replaceUnicode ? 1 : 0
            guard let proxy, let plan = EditPlan.make(owned: deletes == 1 ? "a" : "", backspaces: deletes, output: "á"),
                  let events = Self.makeBatch(plan, nonce: fixedTrace.nonce,
                    eventSource: usePrivateSource ? privateSource : nil) else { resetStream(); return pass }
            fixedTrace.expect(events)
            consumed.insert(code); replacements &+= 1; suppressed &+= 1
            for e in events { e.tapPostEvent(proxy) }
            return nil
        }
        var units = [UniChar](repeating: 0, count: 8); var length = 0
        event.keyboardGetUnicodeString(maxStringLength: units.count, actualStringLength: &length, unicodeString: &units)
        guard length == 1, let scalar = Unicode.Scalar(units[0]), scalar.value >= 32, scalar.value <= 126 else {
            resetStream(); return pass
        }
        if mode == .engineFixture {
            guard fixtureAbort == nil, fixtureIndex < Self.fixtureKeys.count else { return pass }
            guard scalar == Self.fixtureKeys[fixtureIndex] else {
                fixtureAbort = "unexpected_input_not_recorded"; resetStream(); return pass
            }
        }
        let onResult: ((FunputResult, String) -> Void)? = mode == .engineFixture ? { result, output in
            var record = Self.coreMetadata(result, output)
            record["fixture_index"] = self.fixtureIndex
            record["key_code"] = Int(code)
            record["flags"] = event.flags.rawValue
            self.fixtureRecords.append(record)
        } : nil
        let decision = composition.feed(scalar, onResult: onResult)
        if mode == .engineFixture { fixtureIndex += 1 }
        switch decision {
        case .pass: return pass
        case .rejected: resetStream(); return pass
        case .replace(let plan):
            if returnOriginal {
                guard let proxy, let events = Self.returnPacket(plan, on: event, nonce: nonce,
                    eventSource: usePrivateSource ? privateSource : nil) else { resetStream(); return pass }
                if mode == .engineFixture { fixedTrace.expect(events + [event]) }
                replacements &+= 1
                for e in events { e.tapPostEvent(proxy) }
                // Its physical keyUp passes unchanged, paired with this keyDown.
                return pass
            }
            guard let proxy, let events = Self.makeBatch(plan, nonce: nonce,
                eventSource: usePrivateSource ? privateSource : nil) else {
                resetStream(); return pass
            }
            // All allocations succeeded before suppressing the physical key.
            if mode == .engineFixture { fixedTrace.expect(events) }
            consumed.insert(code); replacements &+= 1; suppressed &+= 1
            for e in events { e.tapPostEvent(proxy) }
            return nil
        }
    }
    func diagnostics() -> [String: Any] {
        let sorted = durations.sorted()
        func percentile(_ p: Double) -> Double {
            guard !sorted.isEmpty else { return 0 }
            return Double(sorted[min(sorted.count - 1, Int(Double(sorted.count - 1) * p))]) / 1000
        }
        return ["running": running, "reason": reason, "mode": mode.rawValue,
            "delivery": returnOriginal ? "return_original" : "post_pair",
            "fixed_transport_trace": fixedTrace.snapshot(),
            "fixed_probe_steps": fixedStep,
            "engine_fixture_index": fixtureIndex, "engine_fixture_abort": fixtureAbort as Any? ?? NSNull(),
            "engine_fixture_records": fixtureRecords, "engine_fixture_oracle": fixtureOracle,
            "synthetic_source": usePrivateSource ? "private" : "default",
            "seen": seen, "replacements": replacements, "suppressed": suppressed,
            "accessibility": accessibility, "post": postAccess, "listen_observed": listenAccess,
            "permission_observation_age_s": max(0, ProcessInfo.processInfo.systemUptime - permissionTime),
            "tap_attached": tap.map { CGEvent.tapIsEnabled(tap: $0) } ?? false,
            "callback_samples": sorted.count, "callback_us_p50": percentile(0.50),
            "callback_us_p95": percentile(0.95), "callback_us_p99": percentile(0.99),
            "document_verified": false]
    }
    deinit {
        permissionTimer?.invalidate()
        if let focusObserver { NSWorkspace.shared.notificationCenter.removeObserver(focusObserver) }
        if let layoutObserver { DistributedNotificationCenter.default().removeObserver(layoutObserver) }
        if let tap { CFMachPortInvalidate(tap) }
        if let source { CFRunLoopRemoveSource(CFRunLoopGetMain(), source, .commonModes) }
    }
}
