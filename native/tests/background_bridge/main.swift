import Foundation
let settings = AppSettings.shared
let driver = EventDriver()
settings.driver = driver
settings.updateExternalTarget(id:"fixture.editor",pid:4242)
precondition(driver.lastPrimedPID == 4242, "target must publish before context reset")
settings.updateExternalTarget(id:"com.vxkey.desktop",pid:9999)
precondition(settings.targetPIDValue == 4242, "own UI must retain external target")
let beforeReturn=driver.resetCount
settings.updateExternalTarget(id:"fixture.editor",pid:4242)
precondition(driver.resetCount == beforeReturn+1,"returning from own UI must revoke old epoch even for unchanged external PID")
#if VXKEY_SESSION_ADAPTER
// First tick sees pending permission I/O, then an actual grant. Unknown must
// never become a durable permission_required incident.
driver.permissionsReady = false
let observation = Timer(timeInterval:0.4,repeats:false) { _ in
    driver.permissionsReady = true; driver.permissionObserved = true
}
RunLoop.main.add(observation,forMode:.common)
#endif
settings.start()
#if VXKEY_SESSION_ADAPTER
let report = Timer(timeInterval:0.25,repeats:true) { _ in
    let data = try! JSONSerialization.data(withJSONObject:["attached":driver.attached,"suspended":settings.suspended])
    FileHandle.standardOutput.write(data + Data([10]))
}
RunLoop.main.add(report,forMode:.common)
#endif
RunLoop.main.run()
