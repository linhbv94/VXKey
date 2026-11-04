/// No wall-clock lease. A disconnected control plane cannot authorize a fresh
/// start, but does not revoke an already-authorized owner-process session.
struct StreamRunPolicy {
    private(set) var authenticated = false
    private(set) var paused = true
    private(set) var localFault = false
    mutating func observe(connected: Bool, latched: Bool) {
        if latched { paused = true }
        else if connected { authenticated = true; paused = false }
    }
    mutating func pause() { paused = true }
    mutating func fault() { localFault = true }
    mutating func preparedResume() { localFault = false }
    func canRun(configured: Bool, permission: Bool, driverFault: Bool, engineFault: Bool) -> Bool {
        authenticated && !paused && !localFault && configured && permission && !driverFault && !engineFault
    }
}
