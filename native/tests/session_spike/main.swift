import AppKit
import Foundation

var checks = 0
func check(_ value: @autoclosure () -> Bool, _ message: String) {
    checks += 1
    if !value() { fputs("FAIL: \(message)\n", stderr); exit(1) }
}

// Independent editor model applies delete/output, NOT plan.resultingText. It is
// still a fixture, not evidence that a real editor consumes posted events.
let composer = SessionComposition()
func type(_ keys: String, prefix: String = "", suffix: String = "") -> String {
    composer.reset()
    var before = prefix
    for scalar in keys.unicodeScalars {
        switch composer.feed(scalar) {
        case .pass: before += String(scalar)
        case .rejected: check(false, "Unexpected rejection in corpus")
        case .replace(let plan):
            check(plan.backspaces <= before.unicodeScalars.count - prefix.unicodeScalars.count,
                "Deletion cannot reach prefix sentinel")
            before = String(String.UnicodeScalarView(before.unicodeScalars.dropLast(plan.backspaces))) + plan.output
        }
    }
    return before + suffix
}
let corpus = [("aa", "â"), ("ngaan", "ngân"), ("ngaan hangf", "ngân hàng"),
    ("ddwojwc", "được"), ("mwojwt", "mượt"), ("casch", "cách"), ("cachs", "cách"),
    ("w", "ư"), ("W", "Ư"), ("thwr nhwng chwx vwaf", "thử nhưng chữ vừa"),
    ("casi", "cái"), ("tieengs Vieetj", "tiếng Việt"), ("aaaaaa", "aaaaa"),
    ("hoaf thuyr", "hoà thuỷ")]
for (keys, expected) in corpus {
    check(type(keys) == expected, "Core/local suffix: \(keys) -> \(expected)")
    let prefix = "PREFIX 👨‍👩‍👧‍👦 e\u{301} 🧪 "
    let suffix = " 🛰️ SUFFIX"
    check(type(keys, prefix: prefix, suffix: suffix) == prefix + expected + suffix,
        "Prefix/suffix including combining/non-BMP survive fixture edit")
}
for _ in 0..<30 {
    check(type("ngaan hangf ddwojwc casi casch cachs") == "ngân hàng được cái cách cách",
        "Repeated sequence has no AX-warmup dependency")
}
for (code, flags) in [(UInt16(0), CGEventFlags.maskCommand), (36, []), (36, .maskShift),
                       (51, []), (48, []), (123, []), (0, .maskControl), (0, .maskAlternate)] {
    _ = composer.feed("d")
    check(SessionSpikeDriver.isBoundary(code: code, flags: flags, repeated: false), "Explicit boundary")
    composer.reset()
    check(type("ddwojwc casi") == "được cái", "Immediate first word after boundary")
}
check(SessionSpikeDriver.isBoundary(code: 0, flags: [], repeated: true), "Held repeat resets")
check(!SessionSpikeDriver.isBoundary(code: 0, flags: .maskShift, repeated: false), "Shifted letter is composable")
_ = composer.feed("a")
_ = composer.feed("🧪")
check(composer.rendered.isEmpty, "Non-BMP input resets before any deletion")
_ = composer.feed("d")
composer.reset()
if case .replace(let firstW) = composer.feed("w") {
    check(firstW.backspaces == 0 && firstW.output == "ư", "Leading W inserts without old-word deletion")
} else { check(false, "Leading W must compose on first key") }
check(EditPlan.make(owned: "e\u{301}", backspaces: 1, output: "é") == nil,
    "Combining cluster cannot enter scalar-backspace plan")

let plan = EditPlan.make(owned: "ngan", backspaces: 2, output: "ân")!
let batch = SessionSpikeDriver.makeBatch(plan, nonce: 917)!
check(batch.count == 6, "Two complete Backspace pairs then one Unicode pair")
for index in 0..<4 {
    check(batch[index].getIntegerValueField(.keyboardEventKeycode) == 51, "Backspaces precede Unicode")
    check(batch[index].type == (index.isMultiple(of: 2) ? .keyDown : .keyUp), "Down/up paired")
}
for event in batch {
    check(event.getIntegerValueField(.eventSourceUserData) == 917, "Own-event nonce stamped")
    check(event.flags.isEmpty, "Replacement does not leak shortcut modifiers")
}
var units = [UniChar](repeating: 0, count: 20); var count = 0
batch[4].keyboardGetUnicodeString(maxStringLength: 20, actualStringLength: &count, unicodeString: &units)
check(String(utf16CodeUnits: units, count: count) == "ân", "Ordered batch has exact Unicode payload")
let z = CGEvent(keyboardEventSource: nil, virtualKey: 6, keyDown: true)!
check(SessionSpikeDriver.filterEvent(z) == nil && z.type == .keyDown,
    "Session smoke selects nil suppression, without .null mutation")
let x = CGEvent(keyboardEventSource: nil, virtualKey: 7, keyDown: true)!
_ = SessionSpikeDriver.filterEvent(x)
x.keyboardGetUnicodeString(maxStringLength: 20, actualStringLength: &count, unicodeString: &units)
check(String(utf16CodeUnits: units, count: count) == "X", "Probe x is mutated to X")
let a = CGEvent(keyboardEventSource: nil, virtualKey: 0, keyDown: true)!
check(SessionSpikeDriver.filterEvent(a) != nil, "Probe a passes")
guard let privateSource = CGEventSource(stateID: .privateState) else {
    check(false, "WindowServer must allow creating the private event source (no tap/posting requested)")
    exit(1)
}
let privateBatch = SessionSpikeDriver.makeBatch(plan, nonce: 917, eventSource: privateSource)!
check(privateBatch.count == batch.count, "A/B changes source, not batch cardinality")
check(privateBatch.allSatisfy { $0.getIntegerValueField(.eventSourceStateID) == Int64(privateSource.sourceStateID.rawValue) },
    "Private batch uses its independent keyboard state table")
check(privateBatch.allSatisfy { $0.getIntegerValueField(.eventSourceUnixProcessID) == Int64(getpid()) },
    "Private-source events retain own-event producer identity")
let sensitiveFixture = CGEvent(keyboardEventSource: nil, virtualKey: 0, keyDown: true)!
let sensitiveUnits = Array("FIXTURE_ONLY".utf16)
sensitiveFixture.keyboardSetUnicodeString(stringLength: sensitiveUnits.count, unicodeString: sensitiveUnits)
let metadataJSON = String(data: try! JSONSerialization.data(withJSONObject: FixedTransportTrace.metadata(sensitiveFixture)), encoding: .utf8)!
check(!metadataJSON.contains("FIXTURE") && metadataJSON.contains("other_not_recorded"),
    "Fixed-probe metadata never serializes unknown payload text")
let physicalTone = CGEvent(keyboardEventSource: nil, virtualKey: 1, keyDown: true)!
physicalTone.flags = .maskNonCoalesced
let returnDeletions = SessionSpikeDriver.returnPacket(EditPlan.make(owned: "a", backspaces: 1, output: "á")!,
    on: physicalTone, nonce: 917, eventSource: privateSource)!
check(returnDeletions.count == 2 && returnDeletions.allSatisfy { $0.getIntegerValueField(.keyboardEventKeycode) == 51 },
    "Returned route posts only one Backspace pair")
check(physicalTone.getIntegerValueField(.keyboardEventKeycode) == 1 && physicalTone.type == .keyDown && physicalTone.flags == .maskNonCoalesced,
    "Unicode return preserves original physical key identity, type and flags")
check(FixedTransportTrace.metadata(physicalTone)["payload_class"] as? String == "fixed_acute_a",
    "Original S carries acute-a payload")
let tooLong = CGEvent(keyboardEventSource: nil, virtualKey: 1, keyDown: true)!
check(SessionSpikeDriver.returnPacket(EditPlan.make(owned: "a", backspaces: 1, output: String(repeating: "a", count: 21))!,
    on: tooLong, nonce: 917, eventSource: privateSource) == nil,
    "P1 rejects oversized returned payload before any deletion can be posted")
print(String(data: try! JSONSerialization.data(withJSONObject: [
    "status": "pass", "checks": checks, "native_event_tap_tested": false,
    "event_posting_performed": false, "document_verified": false,
    "scope": "P1 real core, stream state, bounded edit and event construction fixtures"
], options: [.prettyPrinted, .sortedKeys]), encoding: .utf8)!)
