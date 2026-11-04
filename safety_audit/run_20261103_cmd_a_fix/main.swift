import Foundation
import AppKit
import ApplicationServices
let el=AXUIElementCreateApplication(getpid())
let guardrail=ContextGuard { input in
    let initial=input.entries.first!
    return .verified(ContextGuard.Snapshot(pid:input.pid,element:el,caret:5,start:-1,text:"",verified:true,epoch:input.epoch,revision:initial.revision,time:ProcessInfo.processInfo.systemUptime))
}
guardrail.expect(pid:getpid(),text:"",start:-1,epoch:1,immediate:false)
guardrail.expect(pid:getpid(),text:"d",start:-1,epoch:1,immediate:false)
let deadline=Date().addingTimeInterval(0.2)
while guardrail.latest(pid:getpid(),epoch:1)==nil && Date()<deadline { RunLoop.current.run(until:Date().addingTimeInterval(0.001)) }
guard let proof=guardrail.latest(pid:getpid(),epoch:1) else { fatalError("missing fixture proof") }
let decision=ContextGuard.rewriteStart(snapshot:proof,pid:getpid(),epoch:1,owned:"d",start:-1,backspaces:1,window:StreamWindow(),now:proof.time)
assert(proof.text.isEmpty && proof.start == -1)
assert(decision == nil)
print("PASS: same empty-proof pending-d scenario is rejected by production rewrite gate; no OS input posted.")
guardrail.stop()
