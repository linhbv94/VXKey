import Foundation

/// P1 experiment: this suffix is event history, NOT verified document ownership.
/// No AX, bridge, timer, clipboard or document reads participate in composition.
final class SessionComposition {
    enum Decision {
        case pass
        case replace(EditPlan)
        case rejected
    }
    private let composer = FunputComposer()
    private(set) var rendered = ""
    init() {
        composer.configure(FunputConfig(method: UInt8(METHOD_TELEX), tone_style: 1,
            smart_restore: true, eager_restore: true, spell_check: false,
            auto_capitalize: false))
        composer.setEnabled(true)
        // P1 deliberately excludes expansion/parity work. Never import user data.
        composer.setShortcutsEnabled(false)
    }
    func reset() {
        if !vxkey_native_engine_faulted() { composer.clear() }
        rendered = ""
    }
    func feed(_ scalar: Unicode.Scalar, onResult: ((FunputResult, String) -> Void)? = nil) -> Decision {
        guard !vxkey_native_engine_faulted() else { reset(); return .rejected }
        // Telex spike admits printable ASCII only. Unicode pasted/typed by another
        // input source starts a boundary; never backspace across grapheme clusters.
        guard scalar.value >= 32, scalar.value <= 126 else { reset(); return .pass }
        if rendered.unicodeScalars.count >= 128 { reset() }
        let response = composer.processText(scalar)
        guard !vxkey_native_engine_faulted() else { reset(); return .rejected }
        onResult?(response.result, response.output)
        let isLetter = (65...90).contains(scalar.value) || (97...122).contains(scalar.value)
        let decision: Decision
        if response.result.action == ACTION_NONE {
            rendered += String(scalar)
            decision = .pass
        } else if response.result.action == ACTION_SEND,
                  let plan = EditPlan.make(owned: rendered,
                    backspaces: Int(response.result.backspace), output: response.output),
                  plan.resultingText.utf16.count <= 128 {
            rendered = plan.resultingText
            decision = .replace(plan)
        } else {
            reset()
            return .rejected
        }
        // Process the delimiter first (English restore can emit a final edit),
        // then forget the word so no later edit reaches across that boundary.
        if !isLetter { reset() }
        return decision
    }
}
