# Nguồn gốc VXKey

Ngày: 2026-10-30.

- Upstream: https://github.com/Funput/Funput.git
- Base: `e63ac551967dcf17425c5f856591aace2fb8ace5`, lấy từ checkout local `xfunput`; không chỉnh checkout nguồn.
- Git giữ lịch sử upstream, remote tên `upstream`. Chưa tạo commit VXKey, chưa push/release.
- Rust engine/charset/conversion/FFI dùng lại upstream, không mô phỏng lại Telex bằng UI.
- `crates/vxkey-runtime` bổ sung policy, config, service, history RAM và resilience model.
- `native/macos` là adapter InputMethodKit ứng viên, controller dẫn xuất từ upstream. Các file SwiftUI/updater upstream vẫn nằm trong lịch sử/source tham khảo; không được link vào candidate VXKey.
- Candidate dùng namespace `com.vxkey.inputmethod`, connection `VXKey_1_Connection`. Đây chưa phải phê duyệt signer/production identity.
- [MIT notices](third_party_notices.md). Các README upstream còn lại mô tả Funput; không coi link tải/bản phát hành upstream là VXKey.
