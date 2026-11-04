# VXKey1.0.1 — dừng theo user

Ngày2026-11-03. User yêu cầu “stop stop. tôi fix thêm lỗi đã”. Không tự tiếp tục release/CI/publish khi chưa có chỉ đạo mới.

- Branch local: release/v1_0_1 từ origin/main ab0f2b72; chưa commit/push/tag/CI1.0.1.
- Đã sửa local version1.0.1 tại desktop package/lock, Cargo/lock, tauri config và release_identity.json. Giữ nguyên certificate/public updater key của1.0.0.
- Icon: thêm ../../icon.icns trước PNG trong bundle.icon, dùng ICNS nền xanh đã có. PNG trong suốt giữ cho About/UI. finalize_release.py thêm gate đọc CFBundleIconFile và so bytes ICNS với root/icon.icns cho app/helper, archive và DMG.
- Release note1.0.1 mới ở docs/release_notes/1.0.1.md. Frontend build PASS; chưa build signed bundle hoặc nghiệm thu icon trong package1.0.1.
- File QA/handoff/docs bẩn trước lượt này được giữ nguyên, không stage blanket. Không đổi bản1.0.0 đã public hoặc cài đè app user.
- Khi user giao tiếp: đọc diff/fix mới của user, QA phần đó và icon, rồi mới commit/tag/build/publish. Track bảo trì dài hạn vẫn MỞ.

## Hotfix UX quyền/Dock Codex — 2026-11-03

User giao sửa thêm sau khi dừng release. [Report source/checks/gaps](../qa/permission_startup_review.md) phải đọc trước tiếp tục1.0.1. Production hiện đã Resume qua CUA sau restart và báo Sẵn sàng; UI/Dock source mới chưa cài. Các file mới/sửa permission_setup, app/settings_view/index.css, lib.rs Accessory, runtime_settings.swift mở trang quyền thuộc phạm vi hotfix; giữ thay đổi icon/version/signing hiện có. Không publish cho tới chỉ đạo user mới.

## User giao Argon hoàn thiện UX — chưa nghiệm thu

Codex hotfix source chưa được user chấp nhận hoặc native/visual QA; [packet Argon](argon_permission_ux_handoff.md) và mục ƯU TIÊN/CHƯA ĐẠT trong work log thay điểm tiếp tục. User tự giao Argon, Codex không gọi/sửa thêm. Không tự tiếp tục release từ build PASS.

## Codex sửa2findings UX sau review — 2026-11-03

User cho sửa code/báo cáo, không build bundle. [Report hiện hành](../qa/permission_ux_review_fix.md): core sandbox được ghi đúng phạm vi + native external typing guide; restart có Settings draft guard, cancel không stop helper, bỏ prepare_update fallback. Frontend build/Rust check PASS; native/bundle acceptance còn mở. Không tự tiếp tục publish.
