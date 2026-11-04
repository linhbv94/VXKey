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
let start = -1
let resolvedStart=start >= 0 ? start : (proof.start >= 0 ? proof.start : proof.caret)
let sendGuardAccepts=proof.verified && resolvedStart >= 0
let plan=EditPlan.make(owned:"d",backspaces:1,output:"đ")
assert(proof.text.isEmpty && proof.start == -1)
assert(sendGuardAccepts && plan != nil)
print("CONFIRMED: real ContextGuard returns empty proof while d is pending; ACTION_SEND predicate accepts caret=5 and edit plan deletes 1 scalar despite no verified d slice. No OS events posted.")
guardrail.stop()
