import AppKit

/// P1 diagnostic observer, never part of the product typing contract. Records
/// only events carrying a separate nonce from the constant-á probes. Does not
/// record physical input, composer output, application text or AX values.
final class FixedTransportTrace {
    let nonce = Int64.random(in: 1...Int64.max)
    private var observedNonce: Int64 = 0
    private var tap: CFMachPort?
    private var source: CFRunLoopSource?
    private(set) var expected: [[String: Any]] = []
    private(set) var observed: [[String: Any]] = []
    private(set) var disabled = false
    var attached: Bool { tap.map { CGEvent.tapIsEnabled(tap: $0) } ?? false }
    static func metadata(_ event: CGEvent) -> [String: Any] {
        var units = [UniChar](repeating: 0, count: 8); var n = 0
        event.keyboardGetUnicodeString(maxStringLength: 8, actualStringLength: &n, unicodeString: &units)
        let label: String
        if n == 0 { label = "empty" }
        else if n == 1 && units[0] == 0x00e1 { label = "fixed_acute_a" }
        else if n == 1 && units[0] == 0x0008 { label = "backspace" }
        else if n == 1 && units[0] == 0x0061 { label = "plain_a" }
        else { label = "other_not_recorded" }
        let appKitLabel: String
        switch NSEvent(cgEvent: event)?.characters {
        case "á": appKitLabel = "fixed_acute_a"
        case "a": appKitLabel = "plain_a"
        case "\u{8}": appKitLabel = "backspace"
        case "": appKitLabel = "empty"
        case nil: appKitLabel = "unavailable"
        default: appKitLabel = "other_not_recorded"
        }
        return ["type": event.type == .keyDown ? "down" : (event.type == .keyUp ? "up" : "other"),
            "key_code": event.getIntegerValueField(.keyboardEventKeycode),
            "payload_class": label, "appkit_payload_class": appKitLabel, "flags": event.flags.rawValue,
            "own_pid": event.getIntegerValueField(.eventSourceUnixProcessID) == Int64(getpid())]
    }
    static func outputClass(_ text: String) -> String {
        switch text {
        case "": return "empty"
        case "á": return "acute_a"
        case "à": return "grave_a"
        case "â": return "circumflex_a"
        default: return "other_not_recorded"
        }
    }
    func start(acceptingNonce: Int64? = nil) -> Bool {
        stop(); expected = []; observed = []; disabled = false
        observedNonce = acceptingNonce ?? nonce
        let mask = (CGEventMask(1) << CGEventType.keyDown.rawValue) | (CGEventMask(1) << CGEventType.keyUp.rawValue)
        guard let port = CGEvent.tapCreate(tap: .cgAnnotatedSessionEventTap, place: .tailAppendEventTap,
            options: .defaultTap, eventsOfInterest: mask, callback: { _, type, event, pointer in
                guard let pointer else { return Unmanaged.passUnretained(event) }
                let trace = Unmanaged<FixedTransportTrace>.fromOpaque(pointer).takeUnretainedValue()
                if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput { trace.disabled = true }
                else if event.getIntegerValueField(.eventSourceUserData) == trace.observedNonce, trace.observed.count < 64 {
                    trace.observed.append(FixedTransportTrace.metadata(event))
                }
                return Unmanaged.passUnretained(event)
            }, userInfo: Unmanaged.passUnretained(self).toOpaque()),
            let runSource = CFMachPortCreateRunLoopSource(nil, port, 0) else { return false }
        tap = port; source = runSource
        CFRunLoopAddSource(CFRunLoopGetMain(), runSource, .commonModes)
        CGEvent.tapEnable(tap: port, enable: true)
        return attached
    }
    func expect(_ events: [CGEvent]) {
        for event in events where expected.count < 64 { expected.append(Self.metadata(event)) }
    }
    func snapshot() -> [String: Any] {
        ["attached": attached, "disabled": disabled, "expected": expected, "observed": observed,
         "scope": "constant acute-a probes or prescribed as af aa fixture only; unknown payload contents are not recorded"]
    }
    func stop() {
        if let tap { CFMachPortInvalidate(tap) }
        if let source { CFRunLoopRemoveSource(CFRunLoopGetMain(), source, .commonModes) }
        tap = nil; source = nil
    }
    deinit { stop() }
}
