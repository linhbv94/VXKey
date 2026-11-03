# VXKey 1.0.0 — macOS release

Version: 1.0.0 — Ngày update: 2026-11-03

Mac Apple Silicon/macOS26 trước; Windows PLAN ONLY. App com.vxkey.desktop chứa helper com.vxkey.engine. Signer riêng pin ở scripts/release_identity.json; bridge dùng compile-time VXKEY_SIGNER_SHA1, không dùng dev CDHash pins hoặc QA LSEnvironment. CI-only signer helper không được chạy trên workstation user. Khóa local .release_keys bị gitignore; cần backup ngoài máy an toàn. Không tải private key ngược từ GitHub Secrets được.

scripts/build_release.py build Rust release/Swift tối ưu helper trước Tauri release, ký helper trước seal app. --compile-only chỉ kiểm helper, không tạo gói ký/cài. finalize_release.py kiểm bundle/helper, archive, DMG/SDK/architecture/DR/version/noQAenv; check_release_signature.mjs kiểm signature/tamper. Workflow chỉ tạo draft; publish sau exact tagSHA/CI/package gates. Workflow upstream Funput được giữ ngoài thư mục workflows để tránh chạy Windows/Android hoặc publish upstream.

Updater một owner Settings: startup/manual/notice cùng state, draft/busy guard. Download/verify thất bại không dừng helper. Sau native bundle install thành công mới dừng helper do app sở hữu và restart. Không đổi bundle IDs/data root. First release chưa chứng minh N→N+1 giữ quyền. Certificate-bound identity không thay Developer ID/notarization.

General có trạng thái quyền từ helper, Request permissions/Check again/Resume; backend kiểm cửa sổ và session. Không tự Resume khi thiếu/thu hồi quyền; không add Input Source. Không cài đè candidate đang có quyền hoặc reset TCC.

LOG_005 còn mở theo user trial; fail-open phím gốc khi thiếu AX proof. VXKEY_SAFE_001 fixed source/fixture. Release không chứng minh lỗi typing đã hết. Track bảo trì dài hạn MỞ, chỉ user đóng.
