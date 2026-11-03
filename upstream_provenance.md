# Nguồn gốc VXKey

Ngày: 2026-10-30.

- Upstream: https://github.com/Funput/Funput.git
- Base: `e63ac551967dcf17425c5f856591aace2fb8ace5`, lấy từ checkout local `xfunput`; không chỉnh checkout nguồn.
- Git giữ lịch sử upstream, remote tên `upstream`. Checkpoint ban đầu chưa tạo commit VXKey. Sau bàn giao UI, HEAD local là `26314079`, origin `https://github.com/linhbv94/VXKey.git`; lượt review Codex hiện tại không commit/push/release.
- Rust engine/charset/conversion/FFI dùng lại upstream, không mô phỏng lại Telex bằng UI.
- `crates/vxkey-runtime` bổ sung policy, config, service, history RAM và resilience model.
- `native/macos` là adapter InputMethodKit ứng viên, controller dẫn xuất từ upstream. Các file SwiftUI/updater upstream vẫn nằm trong lịch sử/source tham khảo; không được link vào candidate VXKey.
- Candidate dùng namespace `com.vxkey.inputmethod`, connection `VXKey_1_Connection`. Đây chưa phải phê duyệt signer/production identity.
- [MIT notices](third_party_notices.md). Các README upstream còn lại mô tả Funput; không coi link tải/bản phát hành upstream là VXKey.

## Delta native bridge — 2026-10-30

Giữ Rust orthography/engine upstream; thêm native-only authenticated bridge và safe-start supervisor trong `vxkey-runtime`. FFI thêm optional `vxkey-native`/development feature và atomic panic latch; default upstream ABI/behavior không bật feature này. Swift VXKey bỏ file polling để nhận validated cache/ACK, giới hạn main thread/preedit; clipboard không vào input path. Source/QA/gates: [bridge review](docs/qa/native_bridge_review.md). Chưa production identity hoặc native typing acceptance.
