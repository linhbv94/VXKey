import Foundation

/// Bounded speculation between exact AX checkpoints, never an application ACK.
/// A delayed checkpoint may confirm an older prefix without losing later keys.
struct StreamWindow {
    static let maximumKeys = 8
    static let maximumAge: TimeInterval = 0.12
    private(set) var pending: [(revision: UInt64, time: TimeInterval)] = []
    private(set) var confirmed: UInt64 = 0
    var isPending: Bool { !pending.isEmpty }
    mutating func record(_ revision: UInt64, at time: TimeInterval) {
        pending.append((revision,time))
    }
    mutating func acknowledge(_ revision: UInt64) -> Bool {
        guard revision > confirmed else { return false }
        confirmed=revision
        pending.removeAll { $0.revision <= revision }
        return true
    }
    func allows(at time: TimeInterval) -> Bool {
        pending.count < Self.maximumKeys && pending.first.map { time >= $0.time && time-$0.time <= Self.maximumAge } != false
    }
}
