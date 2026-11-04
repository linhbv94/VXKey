import Foundation
import CoreGraphics
import AppKit
import ApplicationServices

var checks = [String]()
func check(_ condition: @autoclosure () -> Bool, _ name: String) {
    guard condition() else { fatalError("FAILED: \(name)") }
    checks.append(name)
}
let composer = FunputComposer()
composer.apply(ComposerConfiguration())
func type(_ text: String) -> String {
    composer.clear()
    var displayed = ""
    for scalar in text.unicodeScalars {
        let response = composer.processText(scalar)
        if response.result.action == ACTION_NONE { displayed.append(Character(String(scalar))) }
        else {
            guard let plan = EditPlan.make(owned: displayed, backspaces: Int(response.result.backspace), output: response.output) else { fatalError("Invalid plan") }
            displayed = plan.resultingText
        }
    }
    return displayed
}
check(type("tieengs") == "tiếng", "Funput Telex through real edit plans")
check(type("Vieetj") == "Việt", "Uppercase Vietnamese")
composer.clearShortcuts(); composer.addShortcut(trigger: "vx", expansion: String(repeating: "Việt Nam ", count: 50))
check(type("vx ").contains(String(repeating: "Việt Nam ", count: 50)), "Full macro output beyond inline 64 scalars")
check(EditPlan.make(owned: "a😀", backspaces: 1, output: "x") == nil, "Reject surrogate ownership")
check(EditPlan.make(owned: "a\u{301}", backspaces: 1, output: "x") == nil, "Reject combining ownership")
check(EditPlan.make(owned: "á", backspaces: 2, output: "x") == nil, "Reject deletion before owned region")
check(EditPlan.make(owned: "abc", backspaces: 1, output: "😀")?.resultingText == "ab😀", "Unicode output preserves owned prefix")
check(EditPlan.make(owned: "abc", backspaces: 1, output: String(repeating: "x", count: 8193)) == nil, "Bound batch size")
let prefix = "KEEP_PREFIX ", suffix = " KEEP_SUFFIX"
let edited = EditPlan.make(owned: "tieeng", backspaces: 6, output: "tiếng")!
check(prefix + edited.resultingText + suffix == "KEEP_PREFIX tiếng KEEP_SUFFIX", "Sentinels remain outside edit plan")
check(type("chaof") == "chào", "Seed owned word for backspace")
check(composer.backspace().action == ACTION_NONE, "Backspace passes once and resynchronizes engine")
let retone = composer.processText("s")
check(EditPlan.make(owned: "chà", backspaces: Int(retone.result.backspace), output: retone.output)?.resultingText == "chá", "Retone only inside owned word")
composer.clear(); check(composer.adopt("chào"), "Adopt previously owned word after deleting its space")
let reopened = composer.processText("s")
check(EditPlan.make(owned: "chào", backspaces: Int(reopened.result.backspace), output: reopened.output)?.resultingText == "cháo", "Retone reopened owned word")
let chunkText = String(repeating: "x", count: 19) + "😀" + String(repeating: "Việt ", count: 100)
let chunks = EditPlan.unicodeChunks(chunkText)
check(chunks.allSatisfy { $0.count <= 20 } && chunks.flatMap { $0 } == Array(chunkText.utf16), "UTF16 chunk cap and exact output")
check(chunks.allSatisfy { $0.last.map { !(0xd800...0xdbff).contains($0) } ?? true }, "Never split a surrogate pair")
composer.apply(ComposerConfiguration(inputMethod: .vni))
check(type("a1") == "á", "Standard VNI digit is a tone")
composer.clear(); _ = composer.processText("a")
check(composer.processText("1", source: .numpad).result.action == ACTION_NONE, "Numpad VNI digit stays literal")
let idleDriver = EventDriver()
if let idleEvent = CGEvent(source: nil) {
    _ = idleDriver.handle(.keyDown,idleEvent,generation:UInt64.max)
    check(idleDriver.diagnostics["seen"] == 0,"Retired tap callback cannot reach current driver")
    _ = idleDriver.handle(.tapDisabledByTimeout,idleEvent,generation:UInt64.max)
    check(!idleDriver.failed,"Retired tap timeout cannot fault replacement")
    _ = idleDriver.handle(.tapDisabledByUserInput, idleEvent)
    check(!idleDriver.failed, "Intentional inactive tap disable is not a runtime fault")
}
func modifier(_ code:UInt16,_ flags:CGEventFlags)->NSEvent {
    let event=CGEvent(keyboardEventSource:nil,virtualKey:code,keyDown:true)!
    event.type = .flagsChanged; event.flags=flags
    return NSEvent(cgEvent:event)!
}
let chord=NativeModifierGesture()
check(!chord.handle(modifier(59,.maskControl),combo:.defaultToggle),"Native Ctrl alone never toggles")
check(!chord.handle(modifier(58,[.maskControl,.maskAlternate]),combo:.defaultToggle),"Native chord waits for release")
_ = chord.handle(modifier(59,.maskAlternate),combo:.defaultToggle)
check(chord.handle(modifier(58,[]),combo:.defaultToggle),"Native modifier-only chord fires after final release")
_ = chord.handle(modifier(59,.maskControl),combo:.defaultToggle)
_ = chord.handle(modifier(58,[.maskControl,.maskAlternate]),combo:.defaultToggle)
chord.nonModifier()
_ = chord.handle(modifier(59,.maskAlternate),combo:.defaultToggle)
check(!chord.handle(modifier(58,[]),combo:.defaultToggle),"Native third key cancels chord without querying OS state")
let injected=idleDriver.batch(EditPlan.make(owned:"a",backspaces:1,output:"á😀")!)!
check(injected.allSatisfy{$0.getIntegerValueField(.eventSourceUnixProcessID)==Int64(getpid())},"Synthetic event producer PID is the helper")
check(Set(injected.map{$0.getIntegerValueField(.eventSourceUserData)}).count==1 && injected[0].getIntegerValueField(.eventSourceUserData) != 0,"Every synthetic down/up carries one nonzero nonce")
check(injected.allSatisfy{$0.flags.intersection([.maskCommand,.maskControl,.maskAlternate,.maskShift]).isEmpty},"Replacement does not synthesize modifiers")
let probeZ=CGEvent(keyboardEventSource:nil,virtualKey:6,keyDown:true)!
let suppressed=EventDriver.probeEvent(probeZ)
check(suppressed?.takeUnretainedValue() === probeZ && probeZ.type == .null,"Filter probe neutralizes z in-place without posting")
let probeUp=CGEvent(keyboardEventSource:nil,virtualKey:6,keyDown:false)!
_ = EventDriver.suppress(probeUp)
check(probeUp.type == .null,"Consumed keyUp is neutralized in-place")
let probeX=CGEvent(keyboardEventSource:nil,virtualKey:7,keyDown:true)!
_ = EventDriver.probeEvent(probeX)
var probeUnits=[UniChar](repeating:0,count:8); var probeCount=0
probeX.keyboardGetUnicodeString(maxStringLength:8,actualStringLength:&probeCount,unicodeString:&probeUnits)
check(probeCount == 1 && probeUnits[0] == 88,"Filter probe mutates returned x to X without injection")
let probeA=CGEvent(keyboardEventSource:nil,virtualKey:0,keyDown:true)!
check(EventDriver.probeEvent(probeA)?.takeUnretainedValue() === probeA,"Filter probe passes unrelated event unchanged")
// Deterministic delayed-checkpoint state-machine tests (no OS input posting).
var burst=StreamWindow()
for rev in 1...3 { burst.record(UInt64(rev),at:Double(rev)*0.01) }
check(burst.allows(at:0.035),"Three rolled-over keys remain within bounded checkpoint window")
check(burst.acknowledge(1) && burst.pending.map{$0.revision} == [2,3],"Late AX acknowledgement preserves newer unverified keys")
check(!burst.acknowledge(1) && burst.pending.count==2,"Repeated old proof cannot reset speculative age")
check(!burst.allows(at:0.15),"Unverified stream expires from oldest remaining key")
_ = burst.acknowledge(3)
check(!burst.isPending && burst.allows(at:10),"Exact newest checkpoint drains pending state")
var cap=StreamWindow()
for rev in 1...StreamWindow.maximumKeys { cap.record(UInt64(rev),at:0) }
check(!cap.allows(at:0.01),"Speculation cannot grow beyond eight physical keys")
cap=StreamWindow()
check(!cap.isPending,"New stream cannot inherit previous speculative keys")
for spelling in ["casch","cachs"] {
    composer.apply(ComposerConfiguration())
    check(type(spelling)=="cách","Flexible tone order: \(spelling)")
}
for (spelling, expected) in [("w","ư"),("W","Ư"),("thw","thư"),("vwng","vưng"),("thwr","thử"),("nhwng","nhưng"),("chwx","chữ"),("vwaf","vừa"),("casi","cái"),("ddwojwc","được"),("ddwojwc ddwojwc ddwojwc muwotj","được được được mượt")] {
    composer.apply(ComposerConfiguration(inputMethod: .telex))
    check(type(spelling)==expected,"LOG_001/LOG_002/LOG_005 Telex composition: \(spelling) -> \(expected)")
}
func spin(_ until:()->Bool, timeout:TimeInterval=1) {
    let end=Date().addingTimeInterval(timeout)
    while !until() && Date()<end { RunLoop.main.run(until:Date().addingTimeInterval(0.002)) }
    check(until(),"Async probe fixture deadline")
}
let fakeElement=AXUIElementCreateApplication(getpid())
let mailbox=ContextGuard { input in
    Thread.sleep(forTimeInterval:0.025)
    let e=input.entries.last!
    return .verified(ContextGuard.Snapshot(pid:input.pid,element:fakeElement,caret:e.start+e.text.utf16.count,start:e.start,text:e.text,verified:true,epoch:input.epoch,revision:e.revision,time:ProcessInfo.processInfo.systemUptime))
}
_ = mailbox.expect(pid:getpid(),text:"a",start:0,epoch:1); mailbox.pump()
var newest:UInt64=0
for i in 2...20 { newest=mailbox.expect(pid:getpid(),text:String(repeating:"a",count:i),start:0,epoch:1) }
spin { mailbox.latest(pid:getpid(),epoch:1)?.revision == newest }
check(mailbox.latest(pid:getpid(),epoch:1)?.text.count == 20,"All expectations submitted during blocked AX coalesce to newest without loss")
let modeLock=NSLock(); var reject=false
let revocation=ContextGuard { input in
    modeLock.lock(); let invalid=reject; modeLock.unlock()
    if invalid { return .invalid }
    let e=input.entries.last!
    return .verified(ContextGuard.Snapshot(pid:input.pid,element:fakeElement,caret:0,start:e.start,text:e.text,verified:true,epoch:input.epoch,revision:e.revision,time:ProcessInfo.processInfo.systemUptime))
}
_ = revocation.expect(pid:getpid(),text:"",start:-1,epoch:1)
spin { revocation.latest(pid:getpid(),epoch:1) != nil }
modeLock.lock(); reject=true; modeLock.unlock()
spin { revocation.latest(pid:getpid(),epoch:1) == nil }
modeLock.lock(); reject=false; modeLock.unlock()
RunLoop.main.run(until:Date().addingTimeInterval(0.04))
check(revocation.latest(pid:getpid(),epoch:1)==nil,"Invalid focus/selection proof stays revoked even after later positive probe in same epoch")
_ = revocation.expect(pid:getpid(),text:"",start:-1,epoch:2)
spin { revocation.latest(pid:getpid(),epoch:2) != nil }
check(revocation.latest(pid:getpid(),epoch:1)==nil,"Old epoch cannot revive after reset")
check(!revocation.isRevoked(pid:getpid(),epoch:2),"New epoch starts unrevoked")
let unanchored=ContextGuard { input in
    let e=input.entries.last!
    let anchor = e.start >= 0 ? e.start : 10
    return .verified(ContextGuard.Snapshot(pid:input.pid,element:fakeElement,caret:anchor+e.text.utf16.count,start:anchor,text:e.text,verified:true,epoch:input.epoch,revision:e.revision,time:ProcessInfo.processInfo.systemUptime))
}
_ = unanchored.expect(pid:getpid(),text:"ca",start:-1,epoch:1)
spin { unanchored.latest(pid:getpid(),epoch:1)?.text == "ca" }
check(unanchored.latest(pid:getpid(),epoch:1)?.start == 10,"Unanchored start-of-sentence prefix resolves start offset from AX")
unanchored.stop()
// LOG_005 regression checks: Cmd+A -> delete transient selection, Shift+Enter <br> transition, and unanchored first-word prefix.
let cmdASelectionInput=ContextGuard.ProbeInput(pid:getpid(),epoch:3,entries:[
    ContextGuard.Expected(revision:1,text:"",start:-1),
    ContextGuard.Expected(revision:2,text:"d",start:-1)
],prior:nil)
if case .waiting = ContextGuard.evaluate(cmdASelectionInput,element:fakeElement,range:CFRange(location:0,length:18),sliceReader:{ _ in nil }) {
    check(true,"LOG_005 Cmd+A transient non-collapsed selection in new unanchored epoch waits instead of revoking")
} else {
    check(false,"LOG_005 Cmd+A transient non-collapsed selection in new unanchored epoch waits instead of revoking")
}
let anchoredSelectionInput=ContextGuard.ProbeInput(pid:getpid(),epoch:3,entries:[
    ContextGuard.Expected(revision:2,text:"d",start:0)
],prior:fakeElement)
if case .invalid = ContextGuard.evaluate(anchoredSelectionInput,element:fakeElement,range:CFRange(location:0,length:1),sliceReader:{ _ in "d" }) {
    check(true,"LOG_005 Selection change after word anchor is established still revokes proof")
} else {
    check(false,"LOG_005 Selection change after word anchor is established still revokes proof")
}
if case .waiting = ContextGuard.evaluate(cmdASelectionInput,element:fakeElement,range:CFRange(location:11,length:0),sliceReader:{ _ in nil }) {
    check(true,"LOG_005 Shift+Enter transient nil slice across <br> waits instead of revoking epoch")
} else {
    check(false,"LOG_005 Shift+Enter transient nil slice across <br> waits instead of revoking epoch")
}
if case .verified(let shifted) = ContextGuard.evaluate(cmdASelectionInput,element:fakeElement,range:CFRange(location:12,length:0),sliceReader:{ r in
    r.location == 11 && r.length == 1 ? "d" : nil
}) {
    check(shifted.start == 11 && shifted.text == "d" && shifted.revision == 2,"LOG_005 Shift+Enter unanchored first key resolves post-newline start=11")
} else {
    check(false,"LOG_005 Shift+Enter unanchored first key resolves post-newline start=11")
}
let phaseLock=NSLock(); var axPhase=0
let cmdADriverGuard=ContextGuard { input in
    phaseLock.lock(); let currentPhase=axPhase; phaseLock.unlock()
    if currentPhase == 0 {
        // Initial empty probe completing after 'd' was already queued, or while selection collapses.
        return .verified(ContextGuard.Snapshot(pid:input.pid,element:fakeElement,caret:0,start:-1,text:"",verified:true,epoch:input.epoch,revision:1,time:ProcessInfo.processInfo.systemUptime))
    }
    if currentPhase == 1 {
        phaseLock.lock(); axPhase = 2; phaseLock.unlock()
        return .waiting
    }
    return ContextGuard.evaluate(input,element:fakeElement,range:CFRange(location:1,length:0)) { r in
        r.location == 0 && r.length == 1 ? "d" : nil
    }
}
_ = cmdADriverGuard.expect(pid:getpid(),text:"",start:-1,epoch:1,immediate:false)
_ = cmdADriverGuard.expect(pid:getpid(),text:"d",start:-1,epoch:1,immediate:false)
spin { cmdADriverGuard.latest(pid:getpid(),epoch:1) != nil }
check(cmdADriverGuard.latest(pid:getpid(),epoch:1)?.verified == true,"LOG_005 Initial empty proof is preserved when snapshot is nil while first key is pending")
let emptyProof=cmdADriverGuard.latest(pid:getpid(),epoch:1)!
let proofTime=emptyProof.time
check(ContextGuard.rewriteStart(snapshot:emptyProof,pid:getpid(),epoch:1,owned:"d",start:-1,backspaces:1,window:StreamWindow(),now:proofTime)==nil,"Safety: empty proof cannot authorize deletion of unverified d")
check(ContextGuard.rewriteStart(snapshot:emptyProof,pid:getpid(),epoch:1,owned:"d",start:0,backspaces:0,window:StreamWindow(),now:proofTime)==nil,"Safety: numeric anchor cannot promote empty proof for a pending word")
check(ContextGuard.rewriteStart(snapshot:emptyProof,pid:getpid(),epoch:1,owned:"",start:-1,backspaces:0,window:StreamWindow(),now:proofTime)==0,"Leading-w insertion with no deletion retains bootstrap")
phaseLock.lock(); axPhase = 1; phaseLock.unlock()
_ = cmdADriverGuard.expect(pid:getpid(),text:"d",start:-1,epoch:1,immediate:false)
spin { cmdADriverGuard.latest(pid:getpid(),epoch:1)?.text == "d" }
check(cmdADriverGuard.latest(pid:getpid(),epoch:1)?.start == 0 && !cmdADriverGuard.isRevoked(pid:getpid(),epoch:1),"LOG_005 Fast 2ms retry resolves unanchored first key after .waiting without revoking")
let wordProof=cmdADriverGuard.latest(pid:getpid(),epoch:1)!
check(ContextGuard.rewriteStart(snapshot:wordProof,pid:getpid(),epoch:1,owned:"d",start:-1,backspaces:1,window:StreamWindow(),now:wordProof.time)==0,"Verified d authorizes rewrite and resolves anchor")
var bounded=StreamWindow(); bounded.record(wordProof.revision+1,at:wordProof.time)
check(ContextGuard.rewriteStart(snapshot:wordProof,pid:getpid(),epoch:1,owned:"đa",start:0,backspaces:1,window:bounded,now:wordProof.time+0.01)==0,"Nonempty checkpoint retains bounded speculation for later keys")
check(ContextGuard.rewriteStart(snapshot:wordProof,pid:getpid(),epoch:2,owned:"d",start:0,backspaces:1,window:bounded,now:wordProof.time)==nil,"Old epoch cannot authorize rewrite")
check(ContextGuard.rewriteStart(snapshot:wordProof,pid:getpid()+1,epoch:1,owned:"d",start:0,backspaces:1,window:bounded,now:wordProof.time)==nil,"Wrong target cannot authorize rewrite")
check(ContextGuard.rewriteStart(snapshot:wordProof,pid:getpid(),epoch:1,owned:"d",start:0,backspaces:1,window:bounded,now:wordProof.time+0.09)==nil,"Expired proof cannot authorize rewrite")
for rev in 1...8 { bounded.record(wordProof.revision+UInt64(rev)+1,at:wordProof.time) }
check(ContextGuard.rewriteStart(snapshot:wordProof,pid:getpid(),epoch:1,owned:"d",start:0,backspaces:1,window:bounded,now:wordProof.time)==nil,"Pending key cap still blocks rewrite")
cmdADriverGuard.stop()
revocation.stop(); mailbox.stop()
check(revocation.latest(pid:getpid(),epoch:2)==nil,"Stopping driver revokes proof and stops probing target")
let permissionEntered=DispatchSemaphore(value:0), permissionRelease=DispatchSemaphore(value:0)
let permissionLock=NSLock(); var permissionFinished=false; var permissionTime:TimeInterval=0
let permissionCache=PermissionCache(reader:{
    permissionEntered.signal()
    _ = permissionRelease.wait(timeout:.now()+0.5)
    permissionLock.lock();permissionFinished=true;permissionLock.unlock()
    return PermissionCache.Value(accessibility:!Thread.isMainThread,listen:true,post:true)
},now:{permissionTime})
check(!permissionCache.ready,"Uninitialized permission cache fails closed")
permissionCache.refresh()
check(permissionEntered.wait(timeout:.now()+0.5) == .success,"Permission preflight worker started")
_ = permissionCache.current
permissionLock.lock();let didFinish=permissionFinished;permissionLock.unlock()
check(!didFinish,"Permission status getter returns while TCC worker is blocked")
permissionRelease.signal()
spin { permissionCache.ready }
check(permissionCache.current.accessibility,"TCC reader executes off keyboard/main run loop")
permissionTime=3
check(!permissionCache.ready && !permissionCache.current.post,"Expired permission cache fails closed without a synchronous preflight")
check(ContextGuard.allowsSubrole(status:.attributeUnsupported,subrole:nil),"Custom editor without named role may attempt exact proof")
check(ContextGuard.allowsSubrole(status:.noValue,subrole:nil),"Missing subrole does not exclude an application")
check(!ContextGuard.allowsSubrole(status:.success,subrole:kAXSecureTextFieldSubrole),"Secure text field is still excluded")
check(!ContextGuard.allowsSubrole(status:.cannotComplete,subrole:nil),"AX RPC failure cannot claim nonsecure context")
let canvasInput=ContextGuard.ProbeInput(pid:getpid(),epoch:9,entries:[ContextGuard.Expected(revision:1,text:"d",start:-1)],prior:nil)
if case .waiting = ContextGuard.evaluate(canvasInput,element:fakeElement,range:CFRange(location:1,length:0),sliceReader:{_ in nil}) {
    check(true,"Any editor without exact text slice cannot authorize rewrite")
} else { check(false,"Any editor without exact text slice cannot authorize rewrite") }
if case .verified(let canvasProof) = ContextGuard.evaluate(canvasInput,element:fakeElement,range:CFRange(location:1,length:0),sliceReader:{_ in "d"}) {
    check(canvasProof.start==0 && canvasProof.text=="d","Capability-based editor requires exact suffix before rewrite")
} else { check(false,"Capability-based editor requires exact suffix before rewrite") }
for (keys,expected) in [("ddwojwc","được"),("casch","cách"),("cachs","cách"),("mwojwt","mượt")] {
    for delayedKeys in [2,3] {
        composer.apply(ComposerConfiguration(inputMethod:.telex)); composer.clear()
        var logical="", actual=""
        for (index,key) in keys.unicodeScalars.enumerated() {
            let response=composer.processText(key)
            if response.result.action == ACTION_NONE { logical += String(key) }
            else { logical=EditPlan.make(owned:logical,backspaces:Int(response.result.backspace),output:response.output)!.resultingText }
            if index<delayedKeys { actual += String(key) }
            else { actual=EditPlan.reconcile(owned:actual,target:logical)!.resultingText }
        }
        check(actual==expected,"Delayed-prefix reconciliation: \(keys) after \(delayedKeys) raw keys")
    }
}
let boundedReconcile=EditPlan.reconcile(owned:"dd",target:"đ")!
check(boundedReconcile.backspaces==2 && boundedReconcile.resultingText=="đ","Deferred correction deletes only the observed owned prefix")
check(EditPlan.reconcile(owned:"a\u{301}",target:"á")==nil,"Deferred reconciliation retains combining ownership rejection")
print(String(data: try! JSONSerialization.data(withJSONObject: ["status":"pass","checks":checks,"native_event_tap_tested":false], options:[.prettyPrinted,.sortedKeys]), encoding:.utf8)!)
