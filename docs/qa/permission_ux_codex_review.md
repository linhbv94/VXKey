# Review bản sửa UX Argon — 2026-11-03

## Kết luận

**Chưa nghiệm thu trọn luồng.** Đã bổ sung setup ba bước, trạng thái quyền, restart, CTA/tray/system và giữ General chỉ Theme/Language ở source. Còn2finding cần xử lý bên dưới. Đây là source/build/unit review, **chưa visual/native nghiệm thu bundle mới**; không lấy app production1.0.0 đang cài làm evidence UI mới.

Working tree nhánh release/v1_0_1, HEADab0f2b72; giữ nguyên thay đổi khác, không sửa nguồn/build bundle/cài đè/cấp quyền/publish hoặc gọi agent ở lượt review. Đọc report Argon và trực tiếp lần call chain ở live_typing_test, service, permission_setup, App routing, restart backend, guard Settings và helper target.

## UX_REVIEW_001 — Ô gõ thử core không phải đường gõ native

Mức medium, confirmed source. `desktop/src/components/live_typing_test.tsx` onKeyDown gọi preventDefault rồi `test_keystroke`, applyEngineEdit/setText. `crates/vxkey-runtime/src/service.rs:608` dùng `self.test_session` là InputSession Rust **riêng** trong host, không phải composer của helper. `runtime_settings.swift:updateExternalTarget` loại appidcom.vxkey.* khỏi target native.

Đủ quyền/attached chỉ là điều kiện mở sandbox, không chứng minh các phím trong ô đi qua EventTap/AX/edit injection. Copy “Mọi phím… nhân Rust… đang chạy” và đếm verifiedKeystrokes dễ khiến user nghĩ bộ gõ toàn hệ thống đã được thử thành công; core có thể đúng trong khi lỗi chữ đầu LOG_005 ở native vẫn tồn tại. Không phải fakeJS conversion, nhưng vẫn là **test core riêng**, không đạt yêu cầu gõ thử xác minh engine native đã giao.

Cần giữ một sandbox nếu hữu ích với nhãn đúng phạm vi; nghiệm thu onboarding phải có đường gõ qua driver thật trong editor phù hợp hoặc bài thử ứng dụng ngoài và readback/user confirmation. Không bỏ own-app bypass toàn cục để chữa nhanh, không sửa guard safety. Đừng ghi “đã QA native” từ test_keystroke policy test.

## UX_REVIEW_002 — Restart bỏ qua guard nháp chưa lưu

Mức medium, confirmed source path; chưa native reproduce mất dữ liệu thật. `desktop/src-tauri/src/lib.rs:187` vxkey_restart_app gọi stop_owned_engine → cho phép exit → app.restart, chỉ kiểm window label. Không đi qua request_quit/guardAction hoặc protocol xác nhận nháp như Quit/Close. Caller ở permission_setup và system_tab.

App auto mở PermissionSetup khi quyền bị revoke; SettingsView vẫn mounted với macroDraft dirty. User có thể restart từ setup trong trạng thái đó; process thoát và nháp mất, không có prompt như Quit thông thường. Label guard không bảo vệ nháp. Hướng sửa: đưa restart thành pending guarded action trong Settings, chỉ relaunch sau save/discard/cancel rõ ràng và backend authorization phù hợp; cancel không dừng helper. Test dirty→restart→cancel và dirty→discard→restart.

## Checks tự chạy

- npm run build --prefix desktop: PASS Settings integrity/TypeScript/Vite. [Log](permission_review_frontend.log).
- cargo +stable check --manifest-path desktop/src-tauri/Cargo.toml --offline: PASS. [Log](permission_review_backend.log).
- cargo +stable test -p vxkey-runtime --offline: **52tests PASS**. [Log](permission_review_runtime.log).

Test hiện có kiểm core command/permission gate, không exercise OS typing hay restart/draft lifecycle. Không chạy lại toàn Rust/Swift suite vì lượt này không sửa nguồn và report Argon đã nêu; không kế thừa suite claim đó thành kiểm chứng độc lập.

## Còn phải kiểm trên bundle signed

First-launch/deny/later/grant/return/restart/timeout/revoke; confirmation active; layout/scroll/focus vi/en các theme; đóng/reopen/Dock/Quit/helper lifecycle; updaterN→N+1 permission retention. Chưa build ký/cài mới ở lượt này; không thể kết luận các mục đó PASS. Bản1.0.1/release vẫn dừng theo user. Track bảo trì MỞ, Windows PLAN ONLY.

## Cập nhật theo yêu cầu sửa code

Hai findings đã được xử lý ở source: [report bản sửa](permission_ux_review_fix.md). Frontend build/Rust check PASS; chưa build bundle hoặc native nghiệm thu. Không xóa evidence finding gốc hoặc nâng trạng thái thành native PASS.
