# VXKey

Bộ gõ tiếng Việt phát triển từ Funput: Rust engine + InputMethodKit trên macOS, UI đích Tauri 2 + React dùng chung Windows. Hiện là mã phát triển, chưa có bản cài VXKey được nghiệm thu.

- [Bộ spec](docs/index.md)
- [Bàn giao UI cho Anti](docs/handoff/ui_handoff.md)
- [Prompt bàn giao](docs/handoff/anti_ui_prompt.md)
- [Kết quả engine và giới hạn kiểm chứng](docs/qa/engine_implementation_report.md)
- [Nguồn gốc upstream](upstream_provenance.md), [license](LICENSE), [notices](third_party_notices.md)

## Kiểm tra local

```sh
sh scripts/test_engine.sh
sh scripts/build_native_candidate.sh
cargo +stable check --manifest-path desktop/src-tauri/Cargo.toml --offline
node desktop/scripts/check_settings_ui.mjs
```

Lệnh build chỉ tạo `build/native_candidate/VXKeyInput.app`; không cài hoặc kích hoạt input source. Toolchain đang kiểm chứng là Rust stable 1.99.0, CLT SDK 27.0, Apple Silicon. Pin Rust 1.98.1 upstream chưa được kiểm chứng trên máy này. Frontend chưa có package scaffold; Tauri hiện là backend library.
