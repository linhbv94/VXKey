import AppKit
import ApplicationServices

/// Main-thread mailbox; AX work never holds a lock needed by the keyboard path.
/// One worker RPC chain in flight. New expectations coalesce without being lost.
final class ContextGuard {
    struct Expected {
        let revision: UInt64
        let text: String
        let start: Int
    }
    struct Snapshot {
        let pid: Int32
        let element: AXUIElement
        let caret: Int
        let start: Int
        let text: String
        let verified: Bool
        let epoch: UInt64
        let revision: UInt64
        let time: TimeInterval
    }
    struct ProbeInput {
        let pid: Int32
        let epoch: UInt64
        let entries: [Expected]
        let prior: AXUIElement?
    }
    enum ProbeResult {
        case verified(Snapshot)
        case waiting
        case invalid
    }
    private let queue = DispatchQueue(label: "com.vxkey.context")
    private var timer: DispatchSourceTimer?
    private var pid: Int32 = 0
    private var epoch: UInt64 = 0
    private var revision: UInt64 = 0
    private var entries: [Expected] = []
    private var snapshot: Snapshot?
    private var element: AXUIElement?
    private var inFlight = false
    private var revoked = false
    private var lastExpectTime: TimeInterval = 0
    private var retryScheduled = false
    private let reader: (ProbeInput) -> ProbeResult
    init(reader: @escaping (ProbeInput) -> ProbeResult = ContextGuard.read) {
        self.reader=reader
        let timer=DispatchSource.makeTimerSource(queue:.main)
        timer.schedule(deadline:.now(),repeating:.milliseconds(10))
        timer.setEventHandler { [weak self] in self?.pump() }
        self.timer=timer; timer.resume()
    }
    private func scheduleRetry() {
        precondition(Thread.isMainThread)
        guard !retryScheduled, !revoked, pid>0, !entries.isEmpty else { return }
        retryScheduled=true
        DispatchQueue.main.asyncAfter(deadline:.now() + .milliseconds(2)) { [weak self] in
            guard let self else { return }
            self.retryScheduled=false
            self.pump()
        }
    }
    @discardableResult
    func expect(pid: Int32, text: String, start: Int, epoch: UInt64, immediate: Bool = true) -> UInt64 {
        precondition(Thread.isMainThread)
        if self.pid != pid || self.epoch != epoch {
            entries=[]; snapshot=nil; element=nil; revoked=false
        }
        self.pid=pid; self.epoch=epoch; revision &+= 1
        lastExpectTime=ProcessInfo.processInfo.systemUptime
        entries.append(Expected(revision:revision,text:text,start:start))
        if entries.count>32 { entries.removeFirst(entries.count-32) }
        if immediate { pump() } else { scheduleRetry() }
        return revision
    }
    func isRevoked(pid: Int32, epoch: UInt64) -> Bool {
        precondition(Thread.isMainThread)
        return self.pid == pid && self.epoch == epoch && revoked
    }
    func latest(pid: Int32, epoch: UInt64) -> Snapshot? {
        precondition(Thread.isMainThread)
        guard !revoked, let s=snapshot, s.pid==pid, s.epoch==epoch,
              ProcessInfo.processInfo.systemUptime-s.time<0.08 else { return nil }
        return s
    }
    /// Empty probes bootstrap a caret, but never prove an existing owned prefix.
    /// Older nonempty checkpoints may support bounded speculation in the same epoch.
    static func rewriteStart(snapshot: Snapshot?, pid: Int32, epoch: UInt64,
                             owned: String, start: Int, backspaces: Int,
                             window: StreamWindow, now: TimeInterval) -> Int? {
        guard let s=snapshot, s.verified, s.pid==pid, s.epoch==epoch,
              now>=s.time, now-s.time<0.08, window.allows(at:now), backspaces>=0 else { return nil }
        if !owned.isEmpty || backspaces>0 {
            guard !s.text.isEmpty, s.start>=0 else { return nil }
            let resolved=start>=0 ? start : s.start
            return resolved>=0 ? resolved : nil
        }
        // A first leading-w insertion deletes nothing; an empty bootstrap may anchor it.
        let resolved=start>=0 ? start : (s.start>=0 ? s.start : s.caret)
        return resolved>=0 ? resolved : nil
    }
    deinit { timer?.cancel() }
    func stop() {
        precondition(Thread.isMainThread)
        pid=0; epoch &+= 1; entries=[]; snapshot=nil; element=nil; revoked=true
    }
    func pump() {
        precondition(Thread.isMainThread)
        guard !inFlight, pid>0, !entries.isEmpty else { return }
        inFlight=true
        let input=ProbeInput(pid:pid,epoch:epoch,entries:entries,prior:element)
        queue.async { [weak self] in
            guard let self else { return }
            let result=self.reader(input)
            DispatchQueue.main.async { [weak self] in self?.publish(result, input:input) }
        }
    }
    private func publish(_ result: ProbeResult, input: ProbeInput) {
        precondition(Thread.isMainThread)
        inFlight=false
        defer {
            if (entries.last?.revision ?? 0) > (input.entries.last?.revision ?? 0) {
                pump()
            } else if !revoked, (entries.last?.revision ?? 0) > (snapshot?.revision ?? 0),
                      ProcessInfo.processInfo.systemUptime - lastExpectTime < 0.15 {
                scheduleRetry()
            }
        }
        guard input.pid==pid, input.epoch==epoch else { return }
        switch result {
        case .invalid:
            revoked=true; snapshot=nil // revoke old proof on focus/selection/secure/AX failure
        case .waiting:
            break // in-flight edit may be between deletion and insertion
        case .verified(let s):
            guard entries.contains(where: { $0.revision==s.revision }),
                  // A late initial empty-field probe must not move the anchor or overwrite non-empty text.
                  !(s.start<0 && (entries.last?.start ?? -1) >= 0),
                  !(s.text.isEmpty && (snapshot.map { !$0.text.isEmpty } ?? false)) else { return }
            if let prior=element, !CFEqual(prior,s.element) { revoked=true; snapshot=nil; return }
            if snapshot.map({$0.revision>s.revision}) == true { return }
            if s.start >= 0 {
                entries = entries.map { $0.start < 0 && !$0.text.isEmpty ? Expected(revision: $0.revision, text: $0.text, start: s.start) : $0 }
            }
            snapshot=s
            if !s.text.isEmpty || s.start >= 0 { element=s.element }
        }
    }
    static func evaluate(_ input: ProbeInput, element el: AXUIElement, range: CFRange, began: TimeInterval = ProcessInfo.processInfo.systemUptime, sliceReader: (CFRange) -> String?) -> ProbeResult {
        let unanchoredEpoch = input.prior == nil && input.entries.allSatisfy({ $0.start < 0 })
        guard range.location >= 0, range.length == 0 else {
            return unanchoredEpoch ? .waiting : .invalid
        }
        // Read only ranges already claimed by this stream, never full AXValue.
        var reads:[String:String]=[:]
        for entry in input.entries.reversed() {
            if entry.start<0, entry.text.isEmpty, input.entries.allSatisfy({$0.start<0 && $0.text.isEmpty}) {
                return .verified(Snapshot(pid:input.pid,element:el,caret:range.location,start:-1,text:"",verified:true,epoch:input.epoch,revision:entry.revision,time:began))
            }
            let entryStart = entry.start >= 0 ? entry.start : range.location - entry.text.utf16.count
            guard entryStart>=0, !entry.text.isEmpty || entry.start>=0, entry.text.utf16.count<=8192,
                  range.location==entryStart+entry.text.utf16.count else { continue }
            let cacheKey = "\(entryStart):\(entry.text.utf16.count)"
            if reads[cacheKey] == nil {
                guard let text = sliceReader(CFRange(location:entryStart,length:entry.text.utf16.count)) else { continue }
                reads[cacheKey]=text
            }
            if reads[cacheKey]==entry.text {
                return .verified(Snapshot(pid:input.pid,element:el,caret:range.location,start:entryStart,text:entry.text,verified:true,epoch:input.epoch,revision:entry.revision,time:began))
            }
        }
        return .waiting
    }
    static func read(_ input: ProbeInput) -> ProbeResult {
        let began=ProcessInfo.processInfo.systemUptime
        guard input.pid>0 else { return .invalid }
        // AX RPCs enforce OS authorization themselves; avoid a TCC preflight for every probe.
        let system=AXUIElementCreateSystemWide()
        AXUIElementSetMessagingTimeout(system,0.04)
        var focusedApp:CFTypeRef?
        guard AXUIElementCopyAttributeValue(system,kAXFocusedApplicationAttribute as CFString,&focusedApp) == .success,
              let focusedApp, CFGetTypeID(focusedApp)==AXUIElementGetTypeID() else { return .invalid }
        var focusedPID:pid_t=0
        guard AXUIElementGetPid(unsafeBitCast(focusedApp,to:AXUIElement.self),&focusedPID) == .success, focusedPID==input.pid else { return .invalid }
        let app=AXUIElementCreateApplication(input.pid)
        AXUIElementSetMessagingTimeout(app,0.04)
        var value:CFTypeRef?
        guard AXUIElementCopyAttributeValue(app,kAXFocusedUIElementAttribute as CFString,&value) == .success,
              let value, CFGetTypeID(value)==AXUIElementGetTypeID() else { return .invalid }
        let el=unsafeBitCast(value,to:AXUIElement.self)
        if let prior=input.prior, !CFEqual(prior,el) { return .invalid }
        AXUIElementSetMessagingTimeout(el,0.04)
        var subrole:CFTypeRef?
        let subroleStatus=AXUIElementCopyAttributeValue(el,kAXSubroleAttribute as CFString,&subrole)
        if subroleStatus == .success, let name=subrole as? String {
            guard name != kAXSecureTextFieldSubrole else { return .invalid }
        } else {
            // Standard multiline editors may have no AXSubrole. Do not treat an
            // RPC failure or an unidentified single-line field as non-secure.
            guard subroleStatus == .attributeUnsupported || subroleStatus == .noValue || subroleStatus == .success else { return .invalid }
            var role:CFTypeRef?
            guard AXUIElementCopyAttributeValue(el,kAXRoleAttribute as CFString,&role) == .success,
                  (role as? String) == kAXTextAreaRole else { return .invalid }
        }
        var selected:CFTypeRef?
        let selectedStatus=AXUIElementCopyAttributeValue(el,kAXSelectedTextRangeAttribute as CFString,&selected)
        guard selectedStatus == .success, let selected, CFGetTypeID(selected)==AXValueGetTypeID() else {
            if (selectedStatus == .noValue || selectedStatus == .cannotComplete),
               input.prior == nil, input.entries.allSatisfy({ $0.start < 0 }) {
                return .waiting
            }
            return .invalid
        }
        var range=CFRange()
        guard AXValueGetValue(unsafeBitCast(selected,to:AXValue.self),.cfRange,&range) else { return .invalid }
        return evaluate(input, element:el, range:range, began:began) { owned in
            var ownedRange=owned
            guard let parameter=AXValueCreate(.cfRange,&ownedRange) else { return nil }
            var found:CFTypeRef?
            guard AXUIElementCopyParameterizedAttributeValue(el,kAXStringForRangeParameterizedAttribute as CFString,parameter,&found) == .success,
                  let text=found as? String else { return nil }
            return text
        }
    }
}
