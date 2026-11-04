# Luồng quyền và Dock — sửa sau feedback production 2026-11-03

## Native kiểm tra trực tiếp production

User nói đã cấp quyền nhưng không biết bước tiếp theo; kiểm `/Applications/VXKey.app` qua CUA: card Chung hiện Accessibility=true, InputMonitoring=true, connected và Resume disabled, còn Hệ thống hiện post_access chưa cấp, driver inactive/suspended. Đã thử yêu cầu quyền từ UI, sau đó thoát/mở lại cùng app; Resume khả dụng. Bấm Resume rồi đọc Hệ thống xác nhận: **Đang dùng, Sẵn sàng, Trợ năng Đã cấp, Phát phím Đã cấp**. Không đổi TCC/keychain/cài đè hay thay signer. Không có raw keystroke acceptance trong lượt này.

## Source đã sửa, chưa có trong app đang cài

- `desktop/src/components/permission_setup.tsx`: hướng dẫn trong chính native Settings window; hiện riêng từng quyền, trạng thái helper/kết nối. Một primary CTA thay đổi theo trạng thái và secondary Để sau; không còn ba nút ngang hàng. Request chỉ khi user bấm; không tự bật OS quyền. Khi đủ quyền có Bắt đầu gõ, đợi native active rồi xác nhận sẵn sàng/Xong. Lỗi có thông báo; sau request còn thiếu quyền có hướng dẫn thoát/mở lại. Tự refresh theo snapshot/focus hiện có.
- `desktop/src/app.tsx`: tự hiện setup khi helper báo thiếu quyền; giữ SettingsView mounted để handler native close/quit và bảo vệ draft còn hoạt động. Để sau chỉ có hiệu lực phiên UI, không giả granted.
- `settings_view.tsx`: bỏ card boolean/connected thô và ba button không style; một nút Thiết lập quyền bàn phím mở flow. Không sửa canonical Settings vendor.
- `desktop/src/index.css`: button app-owned có padding/minheight40px/focus-visible, disabled/busy; phần UI còn lại user hoãn, không redesign toàn app.
- `desktop/src-tauri/src/lib.rs`: macOS Accessory policy cho utility menubar, không giữ icon Dock khi Settings đóng. Native runtime behavior mới này chưa kiểm trên bundle đã build.
- `native/macos_background/runtime_settings.swift`: sau explicit permission action, mở đúng trang Accessibility bằng NSWorkspace URL cố định; không auto request lúc launch, không sửa TCC.

## Validation và giới hạn

Frontend build (Settings gate/TypeScript/Vite) PASS, Rust backend check PASS; helper/native fixture86checks PASS. [Frontend](permission_startup_frontend.log), [backend](permission_startup_backend.log), [native](permission_startup_native.log). Screenshot CUA trả trắng trong lượt xem button, không dùng nó để claim visual QA bản mới. Native AX trên production đã đọc được như trên, nhưng source mới chưa đóng gói/cài/visual acceptance. Không suy Dock/source compile thành native PASS.

Không bump version, commit/push/publish/cài đè hoặc ký mới. Giữ thay đổi icon/version1.0.1 của agent release đã dừng theo user. Release agent đọc lại source và QA permission entry/deny/later/grant/return/restart/resume/close/Dock trên signed bundle mới trước publish. Không tuyên bố production hiện tại đã có UI mới. Long-term track MỞ, Windows PLAN ONLY.

## User không nghiệm thu — bàn giao Argon

User phản đối luồng còn thiếu: trạng thái Sẵn sàng bị giấu trong tab Hệ thống, người mới không có đường tự hoàn tất setup; source hotfix chưa có nút restart/ô gõ thử hoặc native visual proof. Codex dừng sửa theo user, [packet Argon](../handoff/argon_permission_ux_handoff.md) là điểm tiếp tục; không coi các mục source phía trên là acceptance.
