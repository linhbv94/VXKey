import AppKit
import Darwin

// A per-root lifetime lock prevents duplicate taps, including manual helper starts.
let application = NSApplication.shared
application.setActivationPolicy(.prohibited)
let settings = AppSettings.shared
let lockURL = settings.dataRoot.appendingPathComponent("engine.lock")
let lockFD = open(lockURL.path, O_CREAT | O_RDWR | O_NOFOLLOW, S_IRUSR | S_IWUSR)
guard lockFD >= 0, flock(lockFD, LOCK_EX | LOCK_NB) == 0 else { exit(2) }
let driver = EventDriver()
settings.driver = driver
settings.start()
// Parent death cannot leave a detached keyboard utility behind. The tap is owned
// by this process and disappears on exit. No unrelated process is signalled.
let parent = getppid()
let parentTimer = Timer(timeInterval: 0.25, repeats: true) { _ in
    if getppid() != parent || parent <= 1 { driver.setRunning(false); application.terminate(nil) }
}
RunLoop.main.add(parentTimer, forMode: .common)
application.run()
close(lockFD)
