# Yêu cầu Codex Tham vấn: Đánh giá Toàn diện Bản vá Session Driver, Định danh Phiên bản (SemVer) & Hướng dẫn Đóng gói Phát hành

> **Ràng buộc vai trò Codex:** Chỉ tham vấn/review chỉ đọc; không sửa nguồn; không gọi agent khác. Tuân thủ AGENTS.md, tiếng Việt chuyên nghiệp, snake_case, không dùng cú pháp LaTeX.

---

## 1. Bối cảnh và các thay đổi vừa thực hiện
Người dùng đã trực tiếp kiểm thử gõ phím trên macOS (TextEdit, Figma canvas, v.v.) và đánh giá bộ gõ hoạt động khá ổn định.
Trước khi bàn giao cho agent phát hành phiên bản mới, Antigravity cần Codex tham vấn, đánh giá kỹ thuật và hướng dẫn quy trình đóng gói.

Các thay đổi cốt lõi vừa hoàn tất:
1. **LOG_011 (Khắc phục flip shortcut mở AppKit completion menu):**
   - File: `native/macos_session/stream_delivery.swift`, `native/macos_session/session_event_driver.swift`, `native/tests/session_adapter/main.swift`.
   - Triển khai `StreamDelivery.prepareStandalone`: tổng hợp Backspaces và các cặp sự kiện phím chèn mang `virtualKey: 0` (`kVK_ANSI_A`), cờ rỗng `flags: []`, kiểm tra xác thực CoreGraphics chấp nhận chuỗi UTF-16 qua `keyboardGetUnicodeString`.
   - Trong `SessionEventDriver.flipShortcut`: nuốt phím khi autorepeat, phát gói standalone qua `proxy.tapPostEvent()` và nuốt trọn cả `keyDown` lẫn `keyUp` của phím tắt vật lý (F5 / keyCode 96), ngăn chặn tuyệt đối selector `complete:` của `NSTextView`.
2. **LOG_010 (Bảo toàn sửa startup permission false-latch):**
   - File: `native/macos_background/permission_cache.swift`, `native/macos_session/session_event_driver.swift`, `native/macos_background/runtime_settings.swift`.
   - Tách rõ `unknown` vs `denied`: không bịa `permission_required` khi chưa có quan sát thực tế từ hệ thống.
3. **UX Frontend:**
   - File: `desktop/src/components/permission_setup.tsx` (Step 3 giữ trạng thái kiểm thử với external editor thật, không tự đánh dấu xong khi hook active), `desktop/src/components/apps_tab.tsx` (nhãn xóa rule thể hiện rõ hành động và ID ứng dụng).
4. **Kết quả kiểm thử tự động toàn bộ đều PASS:**
   - `sh scripts/test_session_adapter.sh`: 111/111 checks PASS.
   - `VXKEY_SESSION_ADAPTER=1 python3 scripts/test_background_bridge.py`: 6/6 bridge checks PASS.
   - `cargo +stable test -p vxkey-runtime --offline`: 59/59 tests PASS.
   - `node scripts/test_native_permissions.mjs`: 145/145 assertions PASS.
   - `npm run build --prefix desktop`: PASS (Settings UI integrity PASS).
   - `cargo +stable check --manifest-path desktop/src-tauri/Cargo.toml --features custom-protocol --offline`: PASS.
   - Đã đóng gói candidate test tại `build/session_followup_candidate` (data `build/sf_data`, artifacts `docs/qa/session_followup_artifacts.json`).

---

## 2. Các câu hỏi tham vấn cần Codex phân tích & hướng dẫn

### Câu hỏi 1: Lựa chọn SemVer X.Y.Z (Tăng Y hay tăng Z?)
- Phiên bản phát hành đầu tiên công khai hiện tại là `v1.0.0` (phát hành ngày 2026-11-03 cho macOS Silicon).
- Thay đổi tổng thể đợt này bao gồm:
  - Thay đổi kiến trúc cốt lõi của driver gõ macOS: từ kiến trúc cũ (dựa trên AX text proof đồng bộ trong tap gây nghẽn/timeout) sang kiến trúc Session Tap mới (xử lý sự kiện tại chỗ, trả về sự kiện gốc và phát chèn Backspace/ký tự độc lập).
  - Bổ sung bridge protocol role `engine_stream` trong Rust runtime với khả năng kết nối bền bỉ vượt qua lease cũ.
  - Sửa chuỗi lỗi gõ thực tế: W -> Ư, thứ tự gõ dấu phức tạp (`ddwojwc`), chuyển dấu đầu câu/sau Cmd+A xóa/Shift+Enter, per-app exclusion, flip shortcut delivery, v.v.
- **Theo góc nhìn kiến trúc và quy chuẩn phát hành của hệ sinh thái VXUtilities:** Đợt phát hành tới nên tăng **Y** (lên `v1.1.0`) hay tăng **Z** (lên `v1.0.1`)? Đánh giá ưu/nhược điểm và tiền lệ của các app khác trong repo (như VXMedia tăng Y khi nâng cấp engine).

### Câu hỏi 2: Kế hoạch Promote Session Driver vào Pipeline Release Production
- Hiện tại, file build release production `scripts/build_release.py` chỉ biên dịch các file trong `native/macos_background/*.swift` (driver cũ) mà chưa có cờ `-D VXKEY_SESSION_ADAPTER` hay `native/macos_session/*.swift`.
- Để chuẩn bị sẵn sàng cho agent phát hành bản mới (`_devops1_desktop_release` hoặc release workflow):
  - Cần điều chỉnh `scripts/build_release.py` như thế nào để chuyển session driver thành driver mặc định cho production?
  - Có cần lưu ý gì về entitlements, signing, hoặc `release_identity.json` không?

### Câu hỏi 3: Rà soát an toàn & Các điểm cần chuẩn bị trước khi bàn giao Agent Release
- Xin Codex rà soát nhanh các thay đổi trong `native/macos_session/` và `desktop/` xem có điểm nghẽn, rủi ro hồi quy (regression) hoặc điểm thiếu sót nào cần hoàn thiện nốt trước khi đóng gói sẵn sàng cho agent release.
