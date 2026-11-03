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
settings.start()
RunLoop.main.run()
