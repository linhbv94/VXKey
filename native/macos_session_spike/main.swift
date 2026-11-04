import AppKit
import Darwin

final class SpikeApp: NSObject, NSApplicationDelegate {
    private let driver = SessionSpikeDriver()
    private var item: NSStatusItem!
    private var statusRow: NSMenuItem!
    private var permissionRow: NSMenuItem!
    private var filterRow: NSMenuItem!
    private var composeRow: NSMenuItem!
    private var controlWindow: NSWindow?
    private var statusLabel: NSTextField?
    private var filterButton: NSButton?
    private var composeButton: NSButton?
    private var sourceButton: NSButton?
    private var deliveryButton: NSButton?
    private var lockFD: Int32 = -1
    private let root = Bundle.main.bundleURL.deletingLastPathComponent().appendingPathComponent("ss_data")
    func applicationDidFinishLaunching(_ notification: Notification) {
        // The spike is its own owner process; quitting it destroys the hook.
        // No login item/helper/IPC or production preferences are created.
        do { try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true,
            attributes: [.posixPermissions: 0o700]) } catch { NSApp.terminate(nil); return }
        lockFD = open(root.appendingPathComponent("spike.lock").path, O_CREAT | O_RDWR | O_NOFOLLOW, 0o600)
        guard lockFD >= 0, flock(lockFD, LOCK_EX | LOCK_NB) == 0 else { NSApp.terminate(nil); return }
        item = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        let menu = NSMenu()
        let title = NSMenuItem(title: "VXKey Session Spike — P1, chỉ bản thử", action: nil, keyEquivalent: "")
        title.isEnabled = false; menu.addItem(title)
        statusRow = NSMenuItem(title: "Đang tạm dừng", action: nil, keyEquivalent: "")
        statusRow.isEnabled = false; menu.addItem(statusRow)
        permissionRow = NSMenuItem(title: "", action: nil, keyEquivalent: "")
        permissionRow.isEnabled = false; menu.addItem(permissionRow)
        menu.addItem(.separator())
        add(menu, "1. Mở quyền Trợ năng cho bản thử…", #selector(permission))
        filterRow = add(menu, "2. Probe azx → aX (TextEdit/Figma nháp)", #selector(filter))
        composeRow = add(menu, "3. Gõ Telex (TextEdit/Figma nháp)", #selector(compose))
        add(menu, "Tạm dừng ngay", #selector(pause))
        add(menu, "Hướng dẫn thử…", #selector(guide))
        add(menu, "Bảng trạng thái thử…", #selector(showControls))
        add(menu, "Lưu diagnostics không chứa nội dung gõ", #selector(export))
        menu.addItem(.separator())
        add(menu, "Thoát bản thử", #selector(quit))
        item.menu = menu
        driver.changed = { [weak self] in self?.update() }
        update()
        showControls()
    }
    @discardableResult private func add(_ menu: NSMenu, _ title: String, _ action: Selector) -> NSMenuItem {
        let row = NSMenuItem(title: title, action: action, keyEquivalent: "")
        row.target = self; menu.addItem(row); return row
    }
    private func update() {
        guard item != nil else { return }
        item.button?.title = driver.running ? "VX S" : "VX S‖"
        statusRow.title = driver.running ? "Đang thử \(driver.mode.rawValue) — TextEdit/Figma nháp" : "Tạm dừng — \(driver.reason)"
        permissionRow.title = driver.permissionSummary
        filterRow.isEnabled = driver.permissionReady
        composeRow.isEnabled = driver.permissionReady
        statusLabel?.stringValue = "\(driver.reason) | mode=\(driver.mode.rawValue)\n\(driver.permissionSummary)"
        filterButton?.isEnabled = driver.permissionReady
        composeButton?.isEnabled = driver.permissionReady
        sourceButton?.isEnabled = !driver.running
        sourceButton?.state = driver.usePrivateSource ? .on : .off
        deliveryButton?.isEnabled = !driver.running
        deliveryButton?.state = driver.returnOriginal ? .on : .off
    }
    @objc private func permission() { driver.requestAccessibility() }
    @objc private func filter() { _ = driver.resume(.filter) }
    @objc private func compose() { _ = driver.resume(.compose) }
    @objc private func matrixProbe() { _ = driver.resume(.transportMatrix) }
    @objc private func engineProbe() { _ = driver.resume(.engineFixture) }
    @objc private func pause() { driver.pause() }
    @objc private func quit() { NSApp.terminate(nil) }
    @objc private func showControls() {
        if controlWindow == nil {
            let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 540, height: 510),
                styleMask: [.titled, .closable, .miniaturizable], backing: .buffered, defer: false)
            window.title = "VXKey Session Spike — P1 v6 (delivery A/B)"
            window.isReleasedWhenClosed = false
            let stack = NSStackView()
            stack.orientation = .vertical; stack.alignment = .leading; stack.spacing = 14
            stack.translatesAutoresizingMaskIntoConstraints = false
            window.contentView!.addSubview(stack)
            NSLayoutConstraint.activate([
                stack.leadingAnchor.constraint(equalTo: window.contentView!.leadingAnchor, constant: 20),
                stack.trailingAnchor.constraint(equalTo: window.contentView!.trailingAnchor, constant: -20),
                stack.topAnchor.constraint(equalTo: window.contentView!.topAnchor, constant: 20)
            ])
            let instructions = NSTextField(wrappingLabelWithString:
                "Chỉ thử trên nháp TextEdit/Figma; tạm dừng bộ gõ khác.\n1. Cấp Trợ năng → 2. Probe azx phải ra aX → 3. Thử Telex.\npost/listen là giá trị quan sát, chưa chứng minh phát phím.\nĐóng cửa sổ giữ trạng thái hiện tại; Pause/Thoát nằm ngay dưới và ở menu VX S.")
            stack.addArrangedSubview(instructions)
            let status = NSTextField(wrappingLabelWithString: "")
            status.font = .monospacedSystemFont(ofSize: 13, weight: .regular)
            statusLabel = status; stack.addArrangedSubview(status)
            let sourceToggle = NSButton(checkboxWithTitle: "Private event source (A/B, đổi khi tạm dừng)",
                target: self, action: #selector(changeSource))
            sourceButton = sourceToggle; stack.addArrangedSubview(sourceToggle)
            let deliveryToggle = NSButton(checkboxWithTitle: "Trả Unicode trên phím gốc (A/B khi tạm dừng)",
                target: self, action: #selector(changeDelivery))
            deliveryButton = deliveryToggle; stack.addArrangedSubview(deliveryToggle)
            func button(_ title: String, _ action: Selector) -> NSButton {
                let button = NSButton(title: title, target: self, action: action)
                stack.addArrangedSubview(button); return button
            }
            _ = button("1. Mở Trợ năng cho bản thử", #selector(permission))
            filterButton = button("2. Bật probe azx → aX", #selector(filter))
            composeButton = button("3. Bật Telex thử", #selector(compose))
            _ = button("D. Probe x x ax → á á á (không dùng engine)", #selector(matrixProbe))
            _ = button("E. Trace fixture as af aa (chỉ chuỗi cố định)", #selector(engineProbe))
            let row = NSStackView(views: [NSButton(title: "Tạm dừng", target: self, action: #selector(pause)),
                NSButton(title: "Lưu diagnostics", target: self, action: #selector(export)),
                NSButton(title: "Thoát bản thử", target: self, action: #selector(quit))])
            row.spacing = 12; stack.addArrangedSubview(row)
            controlWindow = window; window.center()
        }
        update(); controlWindow?.makeKeyAndOrderFront(nil); NSApp.activate(ignoringOtherApps: true)
    }
    @objc private func changeSource() { driver.setPrivateSource(sourceButton?.state == .on) }
    @objc private func changeDelivery() { driver.setReturnOriginal(deliveryButton?.state == .on) }
    @objc private func guide() {
        let alert = NSAlert()
        alert.messageText = "Bản thử đường gõ macOS, chưa phải VXKey hằng ngày"
        alert.informativeText = "Bản này khởi động ở trạng thái tạm dừng.\n\nTạm tắt xử lý tiếng Việt của MKey/VXKey khác trước khi bật thử. Cấp Trợ năng cho đúng VXKey Session Spike, quay lại menu VX S‖.\n\nChỉ thử tài liệu NHÁP trong TextEdit và Figma: probe azx phải ra aX; sau đó chọn Gõ Telex và thử aa, ngaan, ngaan hangf, ddwojwc, casch cachs. Chưa thử trên tài liệu thật.\n\nMenu luôn có Tạm dừng và Thoát. Không đổi ABC, không dùng clipboard và không ghi nội dung gõ. VNI/macro/retone/hotkey chưa thuộc P1."
        alert.addButton(withTitle: "Đã hiểu — giữ tạm dừng")
        NSApp.activate(ignoringOtherApps: true); alert.runModal()
    }
    @objc private func export() {
        if let data = try? JSONSerialization.data(withJSONObject: driver.diagnostics(), options: [.prettyPrinted, .sortedKeys]) {
            try? data.write(to: root.appendingPathComponent("diagnostics.json"), options: .atomic)
        }
    }
    func applicationWillTerminate(_ notification: Notification) {
        driver.pause("quit"); export()
        if lockFD >= 0 { close(lockFD) }
    }
}

let app = NSApplication.shared
app.setActivationPolicy(.accessory)
let delegate = SpikeApp()
app.delegate = delegate
app.run()
