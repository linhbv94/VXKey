import Foundation

/// Bounded event history, not AX-verified document ownership. Serialized on the
/// native tap loop; all configuration is a previously validated value snapshot.
final class StreamComposition {
    enum Decision { case pass, replace(EditPlan), rejected }
    let composer = FunputComposer()
    private(set) var rendered = ""
    private var completedWord = ""
    private var ownsSpace = false
    private var bypassWord = false
    var isComposing: Bool { !composer.buffer().isEmpty }
    init(configuration: ComposerConfiguration = ComposerConfiguration()) { composer.apply(configuration) }
    func configure(_ configuration: ComposerConfiguration, shortcuts: [TextShortcut]) {
        reset(); composer.apply(configuration); composer.clearShortcuts()
        for shortcut in shortcuts { composer.addShortcut(trigger: shortcut.trigger, expansion: shortcut.expansion) }
    }
    func reset() {
        if !vxkey_native_engine_faulted() { composer.clear() }
        rendered = ""; completedWord = ""; ownsSpace = false; bypassWord = false
    }
    private func decide(_ result: FunputResult, _ output: String) -> Decision {
        guard result.action == ACTION_SEND || result.action == ACTION_RESTORE,
              let plan = EditPlan.make(owned: rendered, backspaces: Int(result.backspace), output: output) else {
            reset(); return .rejected
        }
        rendered = plan.resultingText
        return .replace(plan)
    }
    func feed(_ scalar: Unicode.Scalar, source: FunputComposer.KeySource = .standard) -> Decision {
        guard !vxkey_native_engine_faulted() else { reset(); return .rejected }
        guard scalar.value >= 32, scalar.value <= 126 else { reset(); return .pass }
        if bypassWord {
            if scalar.properties.isWhitespace { reset() }
            return .pass
        }
        if rendered.unicodeScalars.count >= 128 { reset(); bypassWord = true; return .pass }
        completedWord = ""; ownsSpace = false
        let response = composer.processText(scalar, source: source)
        guard !vxkey_native_engine_faulted() else { reset(); return .rejected }
        let decision: Decision
        if response.result.action == ACTION_NONE { rendered += String(scalar); decision = .pass }
        else { decision = decide(response.result, response.output) }
        if composer.buffer().isEmpty {
            // Commit clears edit history, not engine sentence-capitalization state.
            if scalar == " ", rendered.hasSuffix(" ") {
                let word = String(rendered.dropLast())
                if word.unicodeScalars.count <= 128,
                   word.allSatisfy({ $0.isLetter }) { completedWord = word; ownsSpace = true }
            }
            rendered = ""
        }
        return decision
    }
    func backspace(retone: Bool) -> Decision {
        guard retone, !bypassWord, !vxkey_native_engine_faulted() else { reset(); return .pass }
        if ownsSpace {
            let word = completedWord
            completedWord = ""; ownsSpace = false
            guard !word.isEmpty, composer.adopt(word) else { reset(); return .pass }
            rendered = word
            return .pass // Physical Backspace removes only the space just observed.
        }
        guard !rendered.isEmpty,
              EditPlan.make(owned: rendered, backspaces: 1, output: "") != nil else { reset(); return .pass }
        let result = composer.backspace()
        guard !vxkey_native_engine_faulted() else { reset(); return .rejected }
        if result.action != ACTION_NONE {
            guard result.count <= CHARS_CAP else { reset(); return .rejected }
            return decide(result, FunputComposer.output(of: result))
        }
        rendered.removeLast()
        return .pass
    }
    func flip() -> Decision {
        guard !rendered.isEmpty, !vxkey_native_engine_faulted() else { return .pass }
        let result = composer.flipComposing()
        if result.action == ACTION_NONE { return .pass }
        guard result.count <= CHARS_CAP else { reset(); return .rejected }
        return decide(result, FunputComposer.output(of: result))
    }
}
