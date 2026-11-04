# VXKey — review độc lập và tích hợp sau UI

Ngày: 2026-10-30. **Cập nhật sau M2: đọc [bridge/recovery review](native_bridge_review.md) trước; bảng M2 bên dưới là gap tại checkpoint trước, không phải mô tả source hiện tại.** Chủ trì: Codex. Không gọi lại Anti/agent khác. Đây là kết quả source/build/fixture và một số case native companion; không phải nghiệm thu bộ gõ dùng hằng ngày.

## Kết quả

UI Anti đã tạo được scaffold và các màn hình. Review độc lập phát hiện thêm lỗi dữ liệu, quyền cửa sổ và vòng đời app; đã sửa các mục dưới đây. Có companion development bundle nhúng frontend tại `build/desktop_candidate/VXKey.app`, IME candidate riêng tại `build/native_candidate/VXKeyInput.app`. Chưa paired installer, chưa ký nghiệm thu production, chưa cài hoặc kích hoạt input source, chưa release.

### Các lỗi đã xử lý

| Vấn đề | Thay đổi và kiểm chứng |
|---|---|
| Companion và IME đọc hai data root khác nhau | [paths.rs](../../crates/vxkey-runtime/src/paths.rs) dùng Application Support/VXKey trên Mac, VXKEY_DATA_DIR tuyệt đối cho fixture; không fallback ghi vào cwd. IME giữ cùng root |
| Hai process có thể ghi đè config/revision | OS file lock giữ suốt đời ConfigStore, directory mới0700/config lock0600. Process thứ hai bị config_in_use; smoke thực sự chạy hai process và kiểm lock được giải phóng sau exit |
| Macro chỉ tải50 nhưng Save thay cả từ điển | [macro loader](../../desktop/src/services/macro_loader.ts) tải toàn bộ cùng revision, tối đa1000; lỗi/mismatch không trả tập thiếu. Native fixture123→thêm/lưu124→restart vẫn đủ124 |
| Macro draft có thể bị bỏ khi đổi tab/Close/Quit | [Settings](../../desktop/src/components/settings_view.tsx) và [Macros](../../desktop/src/components/macros_tab.tsx) có Save/Discard/Stay, giữ nháp khi conflict/save fail, không trim replacement. Native Close và menu Quit/Cmd+Q đã test; Save rồi Quit có file readback và lock release |
| Quit AppKit bypass chốt draft | Native QA bắt lỗi predefined terminate: thoát thẳng. Đã thay bằng custom Quit menu cùng accelerator Cmd+Q, dispatch qua Settings guard, chỉ final confirmed action mới gọi app.exit. Đã test lại thật |
| Footer Reset tự patch một phần không confirmation | Host mint token CSPRNG UUID one-use/revision-bound/monotonic30s; command reset chỉ Theme+Typing, giữ dữ liệu/locale/policies/macros/hotkeys/clipboard/system. Native Cancel giữ nguyên revision4; Confirm về System revision5, còn124macros/autostart OFF |
| UI cho limits vượt backend | Clipboard controls1..200 entries và1..7days; backend vẫn validate |
| Browser preview giả chuyển mã | Đã bỏ mô phỏng converter làm sai charset/transform order; browser có nhãn Preview/RAM và unsupported cho native actions |
| Các cửa sổ cùng hiện lúc launch | Chỉ Settings visible; mỗi window có URL surface riêng, native label là authority |
| Điều hướng fallback đổi React surface sai quyền | Mở/hide/quit qua host helpers allowlist. Lỗi native hiện rõ, không render Settings bên trong History/Tray để né authority |
| Settings có thể xin history bodies | Service dùng allowlist tường minh; test Settings→history/conversion bị permission_denied. JS capabilities chỉ event listen/unlisten; không core:default, shell hoặc arbitrary paths/URLs |
| Mac native period luôn unknown | Đọc public NSSpellChecker API trước snapshot/mutation, lock enable khi native on/unknown, giữ stored preference. Native UI đã quan sát control disabled khi native OS đang enabled; không đổi OS setting |
| Clipboard copy dựa navigator WebView không chắc chắn | Host explicit NSPasteboard write cho Conversion, history copy kiểm store revision tại backend trước write. Chưa capture/read clipboard; copy native chưa chạy test ghi/đọc clipboard người dùng, không đánh PASS chức năng capture |
| History response có thể cũ hoặc giữ detail sai sau query | Generation guard cho list/detail, invalidate selection/detail khi tải lại, lỗi copy hiển thị rõ; không silent success |
| UI theme/copy sai nghĩa | Pure Black giữ dark variant; shortcut hiển thị saved binding; Smart restore/Eager/Retone mô tả theo upstream thật, bỏ lời hứa “tương thích tuyệt đối” |
| Native đổi config giữa từ | Hoãn syncSettings khi composer còn buffer, reset modifier gesture khi deactivate; build candidate PASS, chưa typing app matrix |
| Tray chưa có lifecycle native/asset contract | Host có tray click-open, click-outside hide, clamp monitor, dock reopen, native menu/About/Edit/Settings. Bốn SVG user được raster bằng script deterministic thành RGBA+hash manifest. Khi IME chưa kết nối, native tray dùng neutral VX, không dùng saved VI/EN giả live state |
| Request IDs lặp lại sau restart | Trace IDs và reset tokens dùng OS CSPRNG UUID; không đặt counter vào UUID rồi tái sử dụng giữa process |

## Checks và bằng chứng

- Cargo core/engine/config/convert/FFI/runtime: **1185 passed, 0 failed**, trong đó **38 tests VXKey runtime**. [Log](engine_cargo_tests.log).
- Frontend regression: **2 tests PASS** cho pagination hơn50 và lỗi trang/revision không sinh tập dữ liệu thiếu. Lệnh `node --experimental-strip-types --test desktop/tests/macro_loader.test.ts`.
- `npm run build --prefix desktop`: canonical vendor integrity + TypeScript + Vite PASS.
- `cargo +stable check --manifest-path desktop/src-tauri/Cargo.toml --offline`: host binary/library PASS.
- `sh scripts/test_engine.sh`: engine suites, process-restart persistence, competing-writer smoke, actual Swift DirectPresenter fixture, Settings vendor gate PASS. [Smoke](backend_smoke.json).
- `sh scripts/build_native_candidate.sh`: IME candidate SDK27/arm64, Rust FFI static PASS; không install/activate.
- `sh scripts/build_desktop_candidate.sh`: local development companion bundle SDK27/arm64, frontend assets nhúng, QA data root trong build; [build log](integration_build.log). Không coi debug bundle là release package.
- Native feature cases được chạy trên candidate trong cùng lượt sửa; sau đổi trace ID CSPRNG đã rebuild và smoke final launch/snapshot/clean Quit, không lặp toàn bộ visual matrix. Native CUA: Settings load từ `tauri://localhost`,123macros tải đủ; thêm/lưu/restart thành124; Close mở draft guard; menu Quit/Cmd+Q guard + Save/readback + exit release lock; theme Light thực sự đổi; Reset Cancel/Confirm bảo toàn124macros; native period-on khóa control. Đây là subset native companion, không phải full visual/native matrix.
- Native About đã quan sát là panel AppKit riêng; chưa nghiệm thu toàn bộ suite logical sizing/credits. Native tray placement/click/negative monitors và Dock Quit/logout còn cần QA, không suy từ source.
- [Evidence/fingerprints](integration_evidence.json), [tray asset hashes](../../desktop/src-tauri/assets/tray_asset_manifest.json). Screenshot trong UI report của Anti là browser preview cũ, không thay các case native trên.
- Rust dùng +stable1.99.0, CLT SDK27.0. Pin upstream1.98.1 chưa chạy; clippy chưa cài. Không tự cài Xcode/toolchain hoặc đổi trust/TCC.

Lượt khởi chạy đầu qua CUA dùng default root và tạo config defaults tại Application Support/VXKey; đó không phải process fixture đã truyền env từ shell. Sau đó bundle có LSEnvironment fixture root rõ ràng; toàn bộ thao tác thêm macro/theme/reset dùng `build/qa_integration_data`, không chỉnh config mặc định đó. Fixture cuối revision5,124macros. Không cài input method hoặc thay bộ gõ đang dùng.

## Việc còn mở — không được tự ghi hết nợ/full acceptance

| Gate | Còn thiếu | Điều kiện đóng |
|---|---|---|
| M2 native driver integration | Authenticated IME↔companion transport, trust identity/DR, handshake/session/ack, per-app mode/memory writeback, actual active source/current-app token | Implement đúng04 mục5/9, fixture hai process và native source đã xác minh; file cache poll chưa thay IPC |
| Recovery / KEY safety | Watchdog gắn actual main event loop, safe-start marker lifecycle, suspend/resume ack, source fallback/mouse path, deadlock/partial injection faults | Native controlled typing/fault matrix theo08; model unit tests chưa đóng gate này |
| macOS double-space | Detector đã nối companion; native edit/UTF-16 ownership/undo và effect ack chưa nối | Gõ trong app matrix, native on/off interlock thực tế, không hai bộ cùng thay Space |
| Global shortcuts / mode routing | Custom registration UI/OS ack, modifier release/focus/sleep/wake native matrix, history hotkey | Không báo “active” từ preference; không giả native registration success |
| Clipboard | OS metadata/provenance adapter, consent receipt, before-read filters, copy ownership, capture worker, encrypted persistence/key management | Fixture trước read + native consent/source markers/races; không plaintext fallback hoặc suy source từ foreground |
| Feature parity còn lại | Native import/export dialogs + preview tokens; file/batch conversion/session Undo; encrypted history; launcher/autostart/updater production | Implement từng typed wrapper và safety gate; không thêm nút fake success |
| Distribution / ID01 | Production namespace/signer/trust baseline, paired packaging/migration/install/rollback | Chốt identity và QA đúng artifact; không tự lấy ad-hoc candidate làm trust proof |
| QA platform | Full native IME app matrix/latency, Intel và Windows | Chạy riêng trên đúng target; Windows vẫn backlog theo chỉ đạo Mac-first |

Autostart vẫn OFF. Browser mocks và compiler không chứng minh bàn phím đã an toàn. Normal close/menu Quit guard không bảo đảm giữ draft khi Force Quit/crash; Dock Quit/logout chưa được nghiệm thu.

## Điểm bắt đầu cho lượt tiếp theo

Đọc work log tổng, report này, spec04/07/08 và [source runtime](../../crates/vxkey-runtime/src/lib.rs). Tiếp tục M2 authenticated native bridge/recovery trước daily-driver activation. Không giao Anti dựng UI lại từ đầu hoặc quay về SwiftUI. Các docs handoff cũ là lịch sử; không phải lệnh tự gọi CLI. Phần UI review/companion có evidence, nhưng track VXKey chưa đóng vì driver/platform gates còn mở.
