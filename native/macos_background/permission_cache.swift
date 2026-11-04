import AppKit
import ApplicationServices

/// Read-only TCC preflights may block on tccd. Never run them on the tap loop.
final class PermissionCache {
    struct Value {
        let accessibility: Bool
        let listen: Bool
        let post: Bool
    }
    private let worker=DispatchQueue(label:"com.vxkey.permission_cache",qos:.utility)
    private var timer:DispatchSourceTimer?
    private var inFlight=false
    private var value=Value(accessibility:false,listen:false,post:false)
    private var updatedAt:TimeInterval?
    private let reader:()->Value
    private let now:()->TimeInterval
    private let requirePost: Bool
    private let expireObservation: Bool
    init(requirePost: Bool = true, expireObservation: Bool = true,
         reader:@escaping ()->Value = { Value(accessibility:AXIsProcessTrusted(),listen:CGPreflightListenEventAccess(),post:CGPreflightPostEventAccess()) }, now:@escaping ()->TimeInterval = { ProcessInfo.processInfo.systemUptime }) {
        self.reader=reader; self.now=now; self.requirePost=requirePost; self.expireObservation=expireObservation
        let timer=DispatchSource.makeTimerSource(queue:.main)
        timer.schedule(deadline:.now(),repeating:.seconds(1))
        timer.setEventHandler { [weak self] in self?.refresh() }
        self.timer=timer;timer.resume()
    }
    deinit { timer?.cancel() }
    func refresh() {
        precondition(Thread.isMainThread)
        guard !inFlight else { return }
        inFlight=true
        let began=now()
        worker.async { [weak self] in
            guard let self else { return }
            let next=self.reader()
            DispatchQueue.main.async { [weak self] in
                guard let self else { return }
                self.value=next;self.updatedAt=began;self.inFlight=false
            }
        }
    }
    var current:Value {
        precondition(Thread.isMainThread)
        guard let updatedAt,now()>=updatedAt,(!expireObservation || now()-updatedAt<2.5) else {
            return Value(accessibility:false,listen:false,post:false)
        }
        return value
    }
    // Unknown at startup is not evidence of a denied/revoked grant. Keep the
    // driver stopped via ready=false, but do not create a durable fault yet.
    var observed:Bool { precondition(Thread.isMainThread); return updatedAt != nil }
    var denied:Bool { observed && !ready }
    var ready:Bool { let v=current;return v.accessibility && (!requirePost || v.post) }
}
