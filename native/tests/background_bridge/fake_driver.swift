import Foundation
// This fixture substitutes ONLY OS hook/permission I/O. Cache, timer, wire,
// recovery latch and real Rust composer are the production implementations.
final class EventDriver {
    var hasUnverifiedEvents: Bool { false }
    var hasComposition: Bool { !composer.buffer().isEmpty }
    let composer = FunputComposer()
    var diagnostics: [String:UInt32] { ["seen":0,"bypassed":0,"missing_target":0,"unverified":0,"rewrites":0,"verified":0,"failed":0] }
    var lastPrimedPID:Int32?
    var resetCount=0
    var failed = false
    var attached = false
    var permissionsReady = true
    #if VXKEY_SESSION_ADAPTER
    var permissionObserved = false
    var permissionDenied: Bool { permissionObserved && !permissionsReady }
    #else
    var permissionDenied: Bool { !permissionsReady }
    #endif
    var permissionStatus: [String:Bool] {
        #if VXKEY_SESSION_ADAPTER
        return ["accessibility":permissionsReady,"listen_access":false,"post_access":false,"attached":attached]
        #else
        return ["accessibility":true,"listen_access":true,"post_access":true,"attached":attached]
        #endif
    }
    func targetChanged() { }
    func prepare() -> Bool { permissionsReady }
    func setRunning(_ value: Bool) { attached = value }
    func reset() { resetCount += 1; lastPrimedPID=AppSettings.shared.targetPIDValue; composer.clear() }
    func verifyPending() { }
    func syncSettings() { composer.apply(ComposerConfiguration(settings:AppSettings.shared)) }
}
