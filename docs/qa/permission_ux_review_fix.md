# Sửa findings UX review — 2026-11-03

Theo user: sửa code và báo cáo, **không build lại bundle**. Không cài đè/đổi quyền/ký/commit/push/release/gọi agent.

## UX_REVIEW_001 — đã sửa phạm vi và hướng dẫn ở source

`desktop/src/components/live_typing_test.tsx` nay hiển thị bài thử gõ tay trong TextEdit/ứng dụng ngoài trước, theo input method hiện hành; nhắc không paste, thử sau chọn-xóa/xuống dòng. Nói rõ Active chỉ xác nhận driver bật và app không đọc kết quả từ ứng dụng khác.

Sandbox hiện có được giữ dưới details đóng mặc định, đổi nhãn/copy thành thử quy tắc core Rust riêng; không còn gọi đây là kiểm đường gõ native hoặc đếm phím như evidence native. Rust test_session vẫn riêng, không sửa own-app bypass hoặc nới guard gõ để chạy demo. Đây là sửa UX/phạm vi bằng chứng, **không phải native typing đã tự động PASS**. Bài thử ứng dụng ngoài vẫn cần user/QA làm thật.

## UX_REVIEW_002 — đã sửa flow restart ở source

- `desktop/src-tauri/src/lib.rs`: `vxkey_restart_app(confirmed)` chưa confirmed chỉ mở/focus Settings và emit request_restart. Không stop_owned_engine hay bật may_exit trong request branch. Confirmed chỉ nhận từ settings label, rồi mới dừng helper và restart; tương tự boundary Quit đang dùng.
- `settings_view.tsx`: listener qua restartRef hiện hành → reveal Settings → guardAction('restart'). Nếu macroDraft dirty thì dùng modal Lưu/Bỏ nháp/Ở lại đã có. Save thất bại không performAction; Stay không gọi restart. Chỉ performAction restart mới ghi pending_resume và gọi confirmed=true. Nếu native call lỗi, xóa flag và báo lỗi.
- `app.tsx`: reveal Settings khi nhận request, để modal draft đang mounted không bị che bởi PermissionSetup; không đổi tab làm unmount draft.
- `permission_setup.tsx`, `system_tab.tsx`: request restart xong bỏ busy để không kẹt khi user cancel; không ghi pending_resume sớm. **Bỏ fallback vxkey_prepare_update** trong catch, vì lỗi yêu cầu restart không được làm helper ngừng chạy hoặc cho phép thoát.

Cổng confirmed có label restriction như Quit; đây không phải security boundary chống renderer Settings bị compromise. Không bổ sung quyền IPC mới hoặc nhượng quyền restart cho renderer khác.

## Validation

- `npm run build --prefix desktop`: Settings canonical integrity, TypeScript, Vite PASS. [Log](permission_review_fix_frontend.log).
- `cargo +stable check --manifest-path desktop/src-tauri/Cargo.toml --offline`: PASS. [Log](permission_review_fix_backend.log).
- Lần caller source: chỉ Settings performAction gửi confirmed=true; setup/system gửi request không confirmed. Hủy/lưu lỗi không đến branch dừng helper. Không giữ fallback prepare_update khi request fail.
- Không rerun52runtime tests vì Rust service/composer không đổi ở lượt này; kết quả trước ở [review](permission_ux_codex_review.md). Không gọi compile/check là native runtime test.

**Hai findings đã xử lý ở source, packaged/native verification còn mở.** Cần thử dirty draft → restart → Stay (nháp/helper giữ nguyên), Save lỗi (không restart), Discard/Save thành công→restart đúng1lần; setup overlay không che modal; return/relaunch, ngoại ứng dụng gõ thật, Dock. Không build bundle theo yêu cầu user; binary đang cài chưa chứa bản sửa này. Version/icon/signing/release1.0.1 giữ nguyên; release vẫn theo chỉ đạo user tiếp theo. Track dài hạn MỞ, Windows PLAN ONLY.
