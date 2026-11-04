# Báo cáo Argon — Dọn dẹp & Hoàn thiện UX Cấp quyền, Trạng thái, Restart & Gõ thử VXKey (2026-11-03)

## 1. Tóm tắt các vấn đề UX của Codex đã được xử lý triệt để

1. **Dọn sạch vi phạm chuẩn `ST-02` ở tab `General` (`settings_view.tsx`):**
   - Đã gỡ bỏ hoàn toàn khối `<section>` thô mà Codex nhét vào trong `activeTab === 'general'`. Tab `General` hiện tuân thủ nghiêm ngặt chuẩn [settings_standards.md](../../../_summary/standard_docs/settings_standards.md) (`ST-02`: chỉ gồm `G1 Theme → G2 Language`).
   - Bổ sung chấm thông báo (`badge`) trên tab `Bộ gõ` (`typing`) và `Hệ thống` (`system`) ở thanh bên khi thiếu quyền hoặc bộ gõ đang tạm ngừng, kèm thanh trạng thái nổi bật ngay đầu tab `Bộ gõ` với nút hành động 1-click (`Bắt đầu gõ tiếng Việt` hoặc `Thiết lập quyền bàn phím`).

2. **Thiết kế lại toàn diện luồng Thiết lập Quyền & Kích hoạt (`permission_setup.tsx`):**
   - Thay thế giao diện `<main>` văn bản thuần sơ sài bằng giao diện 3 bước rõ ràng (`1. Cấp 3 quyền macOS` → `2. Bắt đầu gõ tiếng Việt` → `3. Gõ thử & Hoàn tất`), đồng bộ hệ thống thiết kế `vx-surface` / `vx-card` (hỗ trợ đầy đủ Light, Dark, Pure Black / OLED).
   - Hiển thị 3 thẻ quyền riêng biệt (`Trợ năng / Accessibility`, `Đọc phím / Input Monitoring`, `Phát sự kiện phím / Post Events`) với nhãn trạng thái trực quan (`✓ Đã cấp` màu xanh ngọc / `Cần cấp` màu vàng hổ phách) và giải thích ngắn gọn vì sao cần từng quyền.
   - **Nút `Khởi động lại VXKey` trực tiếp (`vxkey_restart_app`):** Khi người dùng vừa bật quyền trong macOS System Settings (đặc biệt là `Input Monitoring` / `Post Events` vốn yêu cầu khởi động lại tiến trình để macOS TCC áp dụng), không còn bắt người dùng tự đoán cách thoát/mở lại bằng chữ; có ngay nút **`Khởi động lại VXKey (nhận quyền mới)`** gọi lệnh backend `vxkey_restart_app` (dừng `VXKeyEngine` an toàn rồi relaunch ứng dụng) và tự nhớ cờ `vxkey_setup_pending_resume` để khi mở lại sẽ vào thẳng Bước 2 (`Bắt đầu gõ tiếng Việt`).
   - **Bước 2 (`Bắt đầu gõ tiếng Việt`) & Bước 3 (`Xác nhận Đang hoạt động ngay tại chỗ`):** Khi đã đủ 3 quyền hệ thống nhưng bộ gõ còn ở trạng thái `suspended`, hiển thị nút chính **`Bắt đầu gõ tiếng Việt`**, chờ xác nhận từ driver nền và chuyển ngay sang biểu ngữ xanh lá **`VXKey đang hoạt động và sẵn sàng gõ tiếng Việt!`** ngay tại màn hình thiết lập (không giấu trạng thái ở tab `Hệ thống`).
   - **Hành động kết thúc rõ ràng:** Nút chính **`Hoàn tất & Thu về thanh Menu`** đóng cửa sổ Settings về thanh Menu (không còn icon Dock nhờ `ActivationPolicy::Accessory`), hoặc nút phụ **`Vào Cài đặt chi tiết`** mở thẳng vào tab `Bộ gõ`. Khi chưa cấp quyền xong, nút **`Để sau, vào Cài đặt`** cho phép tạm bỏ qua mà không gây vòng lặp popup, nhưng tự động mở lại hướng dẫn nếu quyền bị thu hồi về sau.

3. **Ô Gõ thử Trực tiếp kiểm thật nhân Rust Funput (`live_typing_test.tsx` & `service.rs`):**
   - Bổ sung lệnh `"test_keystroke"` trong `vxkey-runtime/src/service.rs` chạy trực tiếp trên `vxkey_runtime::engine::InputSession` (`funput_engine::Engine` thật của Rust) với đúng cấu hình hiện hành của người dùng (`input_method`, `tone_style`, `smart_restore`, `eager_restore`, `spell_check`, `auto_capitalize`, `extra_onsets`, `macros`, và `language_mode` từ native driver).
   - Lệnh `"test_keystroke"` **chỉ cho phép ghép âm khi driver nền thực sự đang kết nối, đã gắn tap (`driver.attached == true`), không bị `suspended`, và đã có đủ cả 3 quyền OS (`accessibility && listen_access && post_access`)**.
   - Tuyệt đối không giả lập chuyển đổi Telex ở tầng giao diện JS/TS; sự kiện dán (`onPaste`) chỉ chèn văn bản thô và reset bộ đệm ghép âm (không tự coi paste là gõ phím).
   - Đặt `<LiveTypingTest />` ngay trên màn hình `PermissionSetup` và trong tab `Bộ gõ` (`TypingTab`).

4. **Làm sạch tab `Hệ thống` (`system_tab.tsx`) và bảng Menu Bar (`tray_panel.tsx`):**
   - Trong `system_tab.tsx`: Đưa khối **Trạng thái Bộ gõ & Quyền hệ thống** lên đầu tab với các huy hiệu trạng thái rõ ràng cho cả 3 quyền (`Accessibility`, `Input Monitoring`, `Post Events`), kết nối, driver nền, xử lý phím, kèm 3 nút hành động rõ nghĩa: **`Bắt đầu gõ tiếng Việt` / `Tạm ngừng xử lý phím`**, **`Mở màn hình Thiết lập & Gõ thử`**, và **`Khởi động lại VXKey`**.
   - Trong `tray_panel.tsx`: Sửa lỗi hiển thị sai nhãn đỏ `Chưa kết nối` khi bộ gõ đã hoạt động (`driver_status === 'active'`) nhưng chưa có ứng dụng ngoài nào được focus (`current_app === null`). Bổ sung nút kêu gọi hành động nổi bật ngay trên menu popup khi thiếu quyền (**`Thiết lập quyền bàn phím…`**) hoặc khi đang tạm ngừng (**`Bắt đầu gõ tiếng Việt`**).

5. **Định tuyến thông minh khi mở System Settings (`runtime_settings.swift`):**
   - Trong `native/macos_background/runtime_settings.swift`: Khi nhận lệnh `request_permissions`, nếu `AXIsProcessTrusted` đã được cấp mà còn thiếu `listen_access` hoặc `post_access`, tự động mở trực tiếp trang **Input Monitoring** (`x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent`) và gọi cả `CGRequestListenEventAccess()` lẫn `CGRequestPostEventAccess()` trên hàng đợi nền.

---

## 2. Danh sách tệp tin đã chỉnh sửa / tạo mới

- [runtime_settings.swift](../../native/macos_background/runtime_settings.swift)
- [service.rs](../../crates/vxkey-runtime/src/service.rs)
- [policies.rs](../../crates/vxkey-runtime/tests/policies.rs)
- [lib.rs](../../desktop/src-tauri/src/lib.rs)
- [live_typing_test.tsx](../../desktop/src/components/live_typing_test.tsx) *(mới)*
- [permission_setup.tsx](../../desktop/src/components/permission_setup.tsx)
- [settings_view.tsx](../../desktop/src/components/settings_view.tsx)
- [system_tab.tsx](../../desktop/src/components/system_tab.tsx)
- [typing_tab.tsx](../../desktop/src/components/typing_tab.tsx)
- [tray_panel.tsx](../../desktop/src/components/tray_panel.tsx)
- [app.tsx](../../desktop/src/app.tsx)
- [index.css](../../desktop/src/index.css)

*(Giữ nguyên toàn bộ cấu hình phiên bản `1.0.1`, `icon.icns`, `scripts/release_identity.json`, `scripts/finalize_release.py` và bản vá an toàn `VXKEY_SAFE_001`).*

---

## 3. Kết quả kiểm chứng tự động (Source / Build / Unit / Fixture Gates)

- **Settings UI Integrity & Frontend Production Build (`cd desktop && npm run build`):**
  - `node scripts/check_settings_ui.mjs`: **PASS** (`Settings UI 1.0.0: source/CSS/copy integrity PASS`).
  - `tsc && vite build`: **PASS** (0 lỗi TypeScript, bundle thành công).
- **Tauri Backend Check (`cargo +stable check --manifest-path desktop/src-tauri/Cargo.toml --offline`):**
  - **PASS** (0 lỗi, 0 cảnh báo).
- **Rust Runtime Unit & Integration Tests (`cargo +stable test --manifest-path crates/vxkey-runtime/Cargo.toml --offline`):**
  - **52/52 tests PASS** (gồm 1 unittest + 12 bridge tests + 39 policy tests, bao gồm test mới `test_keystroke_requires_active_native_and_composes_via_funput` kiểm chứng chặn `unavailable`/`permission_denied` và ghép âm `ddwojwc` → `được` qua `funput_engine`).
- **Full Engine & Native Swift Fixture Suite (`sh scripts/test_engine.sh`):**
  - **PASS** (Backend isolated smoke PASS, 86/86 Swift background driver checks PASS, Settings UI integrity PASS).

---

## 4. Phần chưa kiểm chứng (Cần thực hiện khi đóng gói bản `1.0.1` signed)

- Các thay đổi trên đang ở source working tree nhánh `release/v1_0_1`; **chưa đóng gói đè lên `/Applications/VXKey.app` hoặc `build/cmd_a_safe_candidate` và chưa publish `1.0.1`** theo đúng ràng buộc bàn giao.
- Khi đóng gói bundle signed `1.0.1`, cần nghiệm thu trực quan và native trên macOS:
  1. Mở lần đầu / thiếu quyền → bấm `Mở Cài đặt Quyền macOS` → cấp quyền → bấm `Khởi động lại VXKey` → bấm `Bắt đầu gõ tiếng Việt` → gõ thử trong ô `LiveTypingTest` → bấm `Hoàn tất & Thu về thanh Menu`.
  2. Kiểm tra sau khi đóng cửa sổ Settings không còn icon VXKey trên Dock (`ActivationPolicy::Accessory`), mở lại từ icon thanh Menu hoặc Finder hiện đúng cửa sổ Settings, và `Thoát VXKey` (`⌘Q`) dừng sạch cả companion lẫn `VXKeyEngine`.
  3. Kiểm tra giữ quyền TCC khi cập nhật từ `1.0.0` lên `1.0.1` qua updater thực tế.
