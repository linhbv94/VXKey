import AppKit
import Foundation

guard let source = CGEventSource(stateID: .privateState) else { exit(2) }
var results: [[String: Any]] = []
for length in [1, 20, 21, 64, 128, 1024, 8192] {
    let text = String(repeating: "á", count: length)
    let units = Array(text.utf16)
    let event = CGEvent(keyboardEventSource: source, virtualKey: 49, keyDown: true)!
    event.keyboardSetUnicodeString(stringLength: units.count, unicodeString: units)
    var actual = 0
    event.keyboardGetUnicodeString(maxStringLength: 0, actualStringLength: &actual, unicodeString: nil)
    var readback = [UInt16](repeating: 0, count: max(actual, 1))
    event.keyboardGetUnicodeString(maxStringLength: readback.count, actualStringLength: &actual, unicodeString: &readback)
    results.append(["requested_utf16": length, "actual_utf16": actual,
        "round_trip_exact": actual == units.count && Array(readback.prefix(actual)) == units])
}
print(String(data: try! JSONSerialization.data(withJSONObject: ["event_posting_performed": false, "results": results],
    options: [.prettyPrinted, .sortedKeys]), encoding: .utf8)!)
