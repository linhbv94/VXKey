# VXKey — kết quả triển khai engine và bàn giao

Ngày: 2026-10-30. Phạm vi: fork/source, engine foundation, native candidate build, adapter backend cho UI. Chưa phải bản dùng hằng ngày hoặc full spec acceptance.

## Đã triển khai

| Thành phần | Bằng chứng / hành vi |
|---|---|
| Upstream Rust | Giữ history/base Funput e63ac551; core/engine/conversion/FFI dùng mã thật |
| VXKey runtime | [Source](../../crates/vxkey-runtime/src/lib.rs), [tests](../../crates/vxkey-runtime/tests/policies.rs) |
| Config | Typed strict fields, semantic validation, quota, revision decimal uint64; temp create_new/0600 + sync + rename trên Mac; corrupt file không bị ghi đè |
| Composition config | Đợi boundary trước đổi engine config, kể cả clear/backspace; không cắt composition đang gõ |
| Input policy | Hard bypass/secure/suspend trước engine, memory tách clipboard policy, target app/pid/generation |
| Shortcut model | Modifier-only nhả hết mới fire; theo dõi hai bên trái/phải; phím khác hoặc modifier thừa hủy; modifier order không né collision |
| Recovery model | Mặc định suspended, heartbeat sequence từ actual loop; partial injection không replay; không auto resume |
| Period model | Native on/unknown khóa; ownership/caret/revision/time checks trước quyết định replace |
| History model | RAM text-only, filter metadata trước payload closure, recheck changeCount sau đọc, quota/TTL/dedup/delete/search; không ghi plaintext persistence |
| Backend service | Persist preference/policy/macros, history RAM API, text conversion thật; unsupported/stale_target trả lỗi rõ |
| Tauri library | Native window label quyết định Surface, không nhận quyền từ JS; mutex poison trả unavailable |
| Native candidate | InputMethodKit + Rust static FFI, không SwiftUI, bundle/connection VXKey riêng, cache đọc bounded ngoài hot path, secure-input bypass |
| Direct rewrite fix | Không cộng selected suffix vào rewrite range; có selection thì drop, bảo toàn văn bản ngoài ownership |

Config commit point là rename. Nếu directory fsync sau rename lỗi, memory vẫn chuyển sang config đã commit và trả `durability_uncertain`; caller refresh snapshot, không giả chắc chắn durability. Chưa fault-inject directory fsync failure. Single-writer/lock đa process và Windows atomic replacement chưa nghiệm thu.

## Checks đã chạy

- `sh scripts/test_engine.sh`: Cargo test core/engine/config/convert/FFI/runtime offline; [log](engine_cargo_tests.log), isolated JSONL smoke + restart readback, Swift composition fixture, canonical Settings integrity.
- VXKey runtime: 34 tests, gồm real Engine parity, boundary/cancel/backspace config, physical modifier release, bypass/secure/suspend, filter-before-read/post-read race, stale target, injection failure, invalid/corrupt config, revision/IO failures, surface authority và converter order.
- Tổng suite Cargo: 1181 passed, 0 failed, 0 ignored; bao gồm upstream tests và doc tests. Số đếm không phải số native scenarios.
- [Backend smoke evidence](backend_smoke.json): thật sự chạy executable với thư mục tạm, save → restart → readback, converter, native unsupported, unknown-field rejection, autostart OFF.
- `sh scripts/test_native_composition.sh`: compile actual DirectPresenter/CompositionPresenter với fake text surface Foundation. False-selection suffix, Unicode UTF-16, stale caret, collapsed rewrite và range ownership PASS. Đây không phải chạy IMK trong app thật và không phải toàn bộ XCTest upstream.
- `cargo +stable check --manifest-path desktop/src-tauri/Cargo.toml --offline`: PASS cho backend library; chưa có frontend/app context nên chưa build full Tauri bundle.
- `sh scripts/build_native_candidate.sh`: PASS, SDK 27.0, arm64 deployment target macOS 26.0, FFI static; `otool -L` không có dependency libfunput_ffi.dylib hoặc build path. [Artifact evidence](engine_implementation_evidence.json).
- `node desktop/scripts/check_settings_ui.mjs`: vendor source/CSS/copy integrity PASS; chưa visual QA.
- `cargo +stable clippy ...`: KHÔNG chạy được vì toolchain chưa cài clippy; không tự tải toolchain/component. Rust upstream pin 1.98.1 không có local; đã dùng `+stable` = 1.99.0. Không coi là pinned-toolchain PASS.
- Checkout nguồn `xfunput` còn sạch; không chỉnh app khác. Chưa commit/push/release.

## Các gap còn phải làm trước dùng thật

1. Native↔companion authenticated IPC, session/revision ack, single writer, target snapshot/token trước focus, mode/language memory writeback và ownership handshake. Cache poll 500ms hiện chỉ là prototype; backend chủ động báo unavailable/disconnected.
2. Native watchdog/controller integration + recovery marker lifecycle + fault injection thật. Các Rust models chưa gắn thành watchdog bảo đảm phục hồi bàn phím. Secure Event Input check không thay toàn bộ secure-field/platform matrix.
3. Native double-space detector/notification, actual document-range ownership/undo và lock sync. Chưa bật transform trên OS.
4. OS clipboard metadata/provenance adapter, capture consent, hotkey/copy routing; encrypted persistence/keychain và crash recovery chưa có. Launch hiện không monitor clipboard.
5. Global custom shortcuts/Windows Win+V provider, launcher/autostart/status/native menu/About/update APIs chưa nối. Custom key-code decode native mới subset, unsupported toggle làm suspend thay vì âm thầm fallback. Không quảng cáo full custom shortcut.
6. Native candidate buộc preedit dù stored composition_mode=direct_verified. Native direct rewrite đã sửa nhưng còn cần matrix app thật, focus/caret/selection/emoji/terminal/browser/Electron và delivery failure. Candidate không phải production profile.
7. Frontend Tauri/React chưa viết; không có tray/history/settings window chạy thật. User sẽ tự giao Anti theo [handoff](../handoff/ui_handoff.md).
8. Signer/namespace release baseline, install/migration/source switch, permission retention và native QA chưa làm. Windows/Intel không được build hoặc QA.

Không cài bundle, không đăng ký/kích hoạt input source, không sửa TCC/keychain, không ghi autostart, không thay bộ gõ user đang dùng. Candidate artifact chỉ ở build local; chưa ký nghiệm thu production.

## Anti và phạm vi bàn giao

Hai lượt trước không có UI source vì vướng quyền headless; exit 0 không được tính hoàn thành. User đã yêu cầu dừng Anti và tự giao UI sau. Lượt bổ sung cho phép trước đó chưa khởi chạy. Không còn chạy CLI Anti trong task này; session-only settings, không thêm global grants.

[UI handoff chi tiết](../handoff/ui_handoff.md) gồm file ownership, scaffold, backend invocation/authority, command/capability matrix, defaults, trạng thái unavailable và checklist. [Prompt cho user tự gọi Anti](../handoff/anti_ui_prompt.md). Bộ spec vẫn là mục tiêu, chưa gắn nhãn specs ready/full implementation.
