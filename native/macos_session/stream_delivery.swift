import AppKit

/// Keep Unicode on the original physical event. CoreGraphics retains up to our
/// configured 8192-unit bound in measured round-trip; editor support is a native
/// gate. Never delete first and discover a truncated payload afterwards.
struct StreamDelivery {
    let deletions: [CGEvent]
    let units: [UInt16]
    var suppressOriginal: Bool { units.isEmpty }
    static func prepare(_ plan: EditPlan, original: CGEvent, source: CGEventSource?, nonce: Int64) -> StreamDelivery? {
        guard (0...128).contains(plan.backspaces), plan.output.utf16.count <= 8192,
              !plan.output.unicodeScalars.contains(where: { $0.value == 0 }),
              let copy = original.copy() else { return nil }
        let units = Array(plan.output.utf16)
        copy.keyboardSetUnicodeString(stringLength: units.count, unicodeString: units)
        var count = 0
        copy.keyboardGetUnicodeString(maxStringLength: 0, actualStringLength: &count, unicodeString: nil)
        guard count == units.count else { return nil }
        if count > 0 {
            var actual = [UInt16](repeating: 0, count: count)
            copy.keyboardGetUnicodeString(maxStringLength: actual.count, actualStringLength: &count, unicodeString: &actual)
            guard actual == units else { return nil }
        }
        var deletions: [CGEvent] = []
        for _ in 0..<plan.backspaces {
            for down in [true, false] {
                guard let event = CGEvent(keyboardEventSource: source, virtualKey: 51, keyDown: down) else { return nil }
                event.flags = []; event.setIntegerValueField(.eventSourceUserData, value: nonce)
                deletions.append(event)
            }
        }
        return StreamDelivery(deletions: deletions, units: units)
    }
    /// Prepare a complete batch of synthetic deletions and text insertion events
    /// for hotkeys/shortcuts where the triggering physical event must be suppressed
    /// (e.g. F5 or special keys whose virtual keycodes must never leak to AppKit/NSTextView).
    static func prepareStandalone(_ plan: EditPlan, source: CGEventSource?, nonce: Int64) -> [CGEvent]? {
        guard (0...128).contains(plan.backspaces), plan.output.utf16.count <= 8192,
              !plan.output.unicodeScalars.contains(where: { $0.value == 0 }) else { return nil }
        let units = Array(plan.output.utf16)
        var events: [CGEvent] = []
        for _ in 0..<plan.backspaces {
            for down in [true, false] {
                guard let event = CGEvent(keyboardEventSource: source, virtualKey: 51, keyDown: down) else { return nil }
                event.flags = []; event.setIntegerValueField(.eventSourceUserData, value: nonce)
                events.append(event)
            }
        }
        if !units.isEmpty {
            guard let down = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: true),
                  let up = CGEvent(keyboardEventSource: source, virtualKey: 0, keyDown: false) else { return nil }
            down.keyboardSetUnicodeString(stringLength: units.count, unicodeString: units)
            up.keyboardSetUnicodeString(stringLength: units.count, unicodeString: units)
            var count = 0
            down.keyboardGetUnicodeString(maxStringLength: 0, actualStringLength: &count, unicodeString: nil)
            guard count == units.count else { return nil }
            if count > 0 {
                var actual = [UInt16](repeating: 0, count: count)
                down.keyboardGetUnicodeString(maxStringLength: actual.count, actualStringLength: &count, unicodeString: &actual)
                guard actual == units else { return nil }
            }
            down.flags = []; up.flags = []
            down.setIntegerValueField(.eventSourceUserData, value: nonce)
            up.setIntegerValueField(.eventSourceUserData, value: nonce)
            events.append(down)
            events.append(up)
        }
        return events
    }
    func apply(to original: CGEvent, nonce: Int64) {
        original.keyboardSetUnicodeString(stringLength: units.count, unicodeString: units)
        original.setIntegerValueField(.eventSourceUserData, value: nonce)
    }
}
