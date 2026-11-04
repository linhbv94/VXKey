import AppKit
import Foundation

var checks = 0
func check(_ value: @autoclosure () -> Bool, _ reason: String) {
    checks += 1
    if !value() { fputs("FAIL: \(reason)\n",stderr); exit(1) }
}
final class Editor {
    let stream: StreamComposition
    var before = "PRE 🧪 "
    let suffix = " SUF"
    init(_ configuration: ComposerConfiguration = ComposerConfiguration(), shortcuts: [TextShortcut] = []) {
        stream = StreamComposition(configuration:configuration)
        stream.configure(configuration,shortcuts:shortcuts)
    }
    func apply(_ decision: StreamComposition.Decision, physical: String) {
        switch decision {
        case .pass: before += physical
        case .rejected: check(false,"Unexpected rejection")
        case .replace(let plan):
            check(plan.backspaces <= before.unicodeScalars.count - "PRE 🧪 ".unicodeScalars.count,"Prefix ownership bound")
            before = String(String.UnicodeScalarView(before.unicodeScalars.dropLast(plan.backspaces))) + plan.output
        }
    }
    func type(_ keys: String, source: FunputComposer.KeySource = .standard) {
        for scalar in keys.unicodeScalars { apply(stream.feed(scalar,source:source),physical:String(scalar)) }
    }
    func backspace(_ retone: Bool = true) {
        let decision = stream.backspace(retone:retone)
        if case .pass = decision { before.removeLast() }
        else { apply(decision,physical:"") }
    }
    var text: String { before + suffix }
}
for (keys,expected) in [("as af aa ngaan hangf casch cachs ddwojwc w","á à â ngân hàng cách cách được ư"),
    ("tieengs Vieetj","tiếng Việt"),("card text word,","card text word,"),("mixx ","mix ")] {
    let editor = Editor(); editor.type(keys)
    check(editor.text == "PRE 🧪 " + expected + " SUF","Core/restoration/Telex plus independent editor sentinels: \(keys)")
}
let vni = Editor(ComposerConfiguration(inputMethod:.vni)); vni.type("a1 a6 d9u7o7c5")
check(vni.text == "PRE 🧪 á â được SUF","VNI uses shared engine")
let pad = Editor(ComposerConfiguration(inputMethod:.vni)); pad.type("a"); pad.type("1",source:.numpad)
check(pad.text == "PRE 🧪 a1 SUF","Numeric pad remains literal in VNI")
let advanced = Editor(ComposerConfiguration(inputMethod:.telexAdvanced)); advanced.type("[ ]")
check(advanced.text == "PRE 🧪 ơ ư SUF","Advanced-only shortcuts retain their composition")
let back = Editor(); back.type("chaof"); back.backspace(); back.type("s")
check(back.text == "PRE 🧪 chá SUF","Backspace and retone")
let reopen = Editor(); reopen.type("chaof "); reopen.backspace(); reopen.type("s")
check(reopen.text == "PRE 🧪 cháo SUF","Reopen only word/space seen by this stream")
let flip = Editor(); flip.type("pass"); flip.apply(flip.stream.flip(),physical:"")
check(flip.text == "PRE 🧪 pass SUF","Flip brings back swallowed key per upstream oracle")
let long = String(repeating:"Việt Nam 🧪 ",count:70)
for enabled in [true,false] {
    let macro = Editor(ComposerConfiguration(enabled:enabled),shortcuts:[TextShortcut(trigger:"vx",expansion:long)])
    macro.type("vx ")
    check(macro.text == "PRE 🧪 " + long + "  SUF","Full macro output in VI/EN")
}
let cap = Editor(ComposerConfiguration(autoCapitalizeEnabled:true)); cap.type("chao. xin")
check(cap.text.contains("Xin"),"Word commits retain sentence capitalization state")
let reset = Editor(); reset.type("dd"); reset.stream.reset(); reset.before = "PRE 🧪 "
reset.type("ddwojwc casi")
check(reset.text == "PRE 🧪 được cái SUF","Reset permits immediate first word")
var policy = StreamRunPolicy()
func allowed(_ p: StreamRunPolicy) -> Bool { p.canRun(configured:true,permission:true,driverFault:false,engineFault:false) }
check(!allowed(policy),"No offline authorization at startup")
policy.observe(connected:false,latched:false)
check(!allowed(policy),"Unlatched cache alone cannot start")
policy.observe(connected:true,latched:false)
check(allowed(policy),"Authenticated unlatched session starts")
policy.observe(connected:false,latched:false)
check(allowed(policy),"Control disconnect preserves authorized typing")
policy.pause(); policy.observe(connected:false,latched:false)
check(!allowed(policy),"Pause survives disconnected control")
policy.observe(connected:true,latched:false); policy.fault()
check(!allowed(policy),"Real local fault stops typing")
policy.observe(connected:true,latched:false)
check(!allowed(policy),"Reconnect alone cannot reset local fault")
policy.preparedResume(); policy.observe(connected:false,latched:true)
check(!allowed(policy),"Durable fault marker still requires host ACK")
policy.observe(connected:true,latched:false)
check(allowed(policy),"Prepared resume plus host ACK restarts")
check(!policy.canRun(configured:true,permission:false,driverFault:false,engineFault:false),"Permission loss blocks")
check(!policy.canRun(configured:true,permission:true,driverFault:true,engineFault:false),"Driver fault blocks")
check(!policy.canRun(configured:true,permission:true,driverFault:false,engineFault:true),"Engine fault blocks")
for allowed in [true, false] {
    let cache = PermissionCache(requirePost:false, expireObservation:false,
        reader:{ PermissionCache.Value(accessibility:allowed,listen:false,post:false) })
    check(!cache.observed && !cache.ready && !cache.denied,"Startup observation pending keeps hook stopped without inventing a permission fault")
    cache.refresh()
    let deadline=Date().addingTimeInterval(2)
    while !cache.observed && Date()<deadline { _ = RunLoop.main.run(mode:.default,before:Date().addingTimeInterval(0.01)) }
    check(cache.observed,"Asynchronous permission observation completes")
    check(cache.ready == allowed,"Only observed grant permits running")
    check(cache.denied == !allowed,"Observed denial still creates a real fault")
}
guard let source = CGEventSource(stateID:.privateState) else { exit(2) }
for text in ["á","à","â",long,String(repeating:"á",count:8192)] {
    let original = CGEvent(keyboardEventSource:nil,virtualKey:1,keyDown:true)!
    original.flags = .maskNonCoalesced
    let plan = EditPlan.make(owned:"a",backspaces:1,output:text)!
    let delivery = StreamDelivery.prepare(plan,original:original,source:source,nonce:123)!
    check(delivery.deletions.count == 2,"Only bounded deletion pair synthesized")
    check(original.getIntegerValueField(.eventSourceUserData) == 0,"Prepare never mutates original input")
    delivery.apply(to:original,nonce:123)
    var count = 0; var actual = [UInt16](repeating:0,count:text.utf16.count)
    original.keyboardGetUnicodeString(maxStringLength:actual.count,actualStringLength:&count,unicodeString:&actual)
    check(actual == Array(text.utf16) && count == actual.count,"Returned packet retains complete long Unicode/emoji output")
    check(original.getIntegerValueField(.keyboardEventKeycode)==1 && original.flags == .maskNonCoalesced,"Physical identity preserved")
}
let deletionOnly = StreamDelivery.prepare(EditPlan.make(owned:"a",backspaces:1,output:"")!,
    original:CGEvent(keyboardEventSource:nil,virtualKey:51,keyDown:true)!,source:source,nonce:123)!
check(deletionOnly.suppressOriginal && deletionOnly.deletions.count == 2,"Delete-only result consumes original exactly once")
let flipPlan = EditPlan.make(owned:"pas",backspaces:3,output:"pass")!
guard let standalone = StreamDelivery.prepareStandalone(flipPlan,source:source,nonce:456) else { exit(3) }
check(standalone.count == 8,"Standalone shortcut replacement synthesizes exactly 6 deletions and 2 insertion events")
check(standalone[0...5].allSatisfy { $0.getIntegerValueField(.keyboardEventKeycode) == 51 },"All deletions carry backspace keycode")
check(standalone[6].getIntegerValueField(.keyboardEventKeycode) == 0 && standalone[7].getIntegerValueField(.keyboardEventKeycode) == 0,"Standalone insertions use safe neutral virtualKey 0, never special/hotkey code like F5")
check(standalone[6].flags.isEmpty && standalone[7].flags.isEmpty,"Standalone insertions strip shortcut modifier flags")
check(standalone.allSatisfy { $0.getIntegerValueField(.eventSourceUserData) == 456 },"All standalone events tagged with nonce")
var flipCount = 0; var flipActual = [UInt16](repeating:0,count:"pass".utf16.count)
standalone[6].keyboardGetUnicodeString(maxStringLength:flipActual.count,actualStringLength:&flipCount,unicodeString:&flipActual)
check(flipActual == Array("pass".utf16) && flipCount == "pass".utf16.count,"Standalone insertion carries exact replacement string")
let emptyPlan = EditPlan.make(owned:"abc",backspaces:2,output:"")!
guard let emptyBatch = StreamDelivery.prepareStandalone(emptyPlan,source:source,nonce:789) else { exit(4) }
check(emptyBatch.count == 4 && emptyBatch.allSatisfy { $0.getIntegerValueField(.keyboardEventKeycode) == 51 },"Standalone deletion-only batch carries only backspaces")
print(String(data:try! JSONSerialization.data(withJSONObject:["status":"pass","checks":checks,
    "native_typing_tested":false,"event_posting_performed":false,"scope":"integrated stream composition/delivery fixtures"],
    options:[.prettyPrinted,.sortedKeys]),encoding:.utf8)!)
