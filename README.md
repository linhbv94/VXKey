# VXKey

[Windows adaptation plan](windows_adaptation_plan.md) - PLAN ONLY

[Nhật ký lỗi gõ](typing_issue_log.md) | [Track bao tri dai han](maintenance_track.md)

> Kiến trúc hiện hành 2026-11-02: bộ gõ chạy nền, giữ ABC/nguồn nhập đang dùng, Rust Funput + native keyboard driver + Tauri UI. Không yêu cầu Input Source VXKey. Đọc [hợp đồng background driver](docs/spec/09_background_driver.md) trước các hướng dẫn IME lịch sử.

Bộ gõ tiếng Việt phát triển từ Funput: Rust engine + Swift/CoreGraphics background driver trên macOS, UI đích Tauri 2 + React dùng chung Windows. Hiện là mã phát triển, chưa có bản cài VXKey được nghiệm thu.

- [Bộ spec](docs/index.md)
- [Background driver và trạng thái kiểm chứng](docs/qa/background_driver_review.md)
- [Review tích hợp companion trước bridge](docs/qa/integration_review.md)
- [Bàn giao UI lịch sử cho Anti](docs/handoff/ui_handoff.md)
- [Prompt bàn giao](docs/handoff/anti_ui_prompt.md)
- [Kết quả engine và giới hạn kiểm chứng](docs/qa/engine_implementation_report.md)
- [Nguồn gốc upstream](upstream_provenance.md), [license](LICENSE), [notices](third_party_notices.md)

## Kiểm tra local

```sh
sh scripts/test_engine.sh
sh scripts/build_native_candidate.sh
sh scripts/build_desktop_candidate.sh
sh scripts/build_bridge_candidate.sh
cargo +stable check --manifest-path desktop/src-tauri/Cargo.toml --offline
node desktop/scripts/check_settings_ui.mjs
```

Native build tạo `build/native_candidate/VXKeyEngine.app`, desktop build tạo companion, bridge build nhúng Engine vào `build/desktop_candidate/VXKey.app` và chuẩn bị development pins. Không lệnh nào cài hoặc kích hoạt input source. Toolchain đang kiểm chứng là Rust stable 1.99.0, CLT SDK 27.0, Apple Silicon. Pin Rust 1.98.1 upstream chưa được kiểm chứng trên máy này. Frontend và Tauri companion host đã có. Desktop candidate là development bundle nhúng UI, có data root QA riêng, không phải bộ cài/production signing. Bridge có authenticated transport, config ACK và recovery latch; đã kiểm fixture/process và native metadata subset. Native typing cần quyền Trợ năng do user cấp; daily-driver/production identity vẫn còn gate trong background review. Cặp bridge development giữ safe-start marker, không tự install/activate; autostart OFF.
