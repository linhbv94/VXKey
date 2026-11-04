# VXKey — bàn giao lượt tiếp cho Anti

Ngày: 2026-11-04. Owner hiện tại: Codex; user dự định tự giao Anti. **File này không có nghĩa Codex đã gọi Anti.**

## Đọc trước và phạm vi

1. [AGENTS riêng](AGENTS.md), quy tắc global/workspace đang áp dụng.
2. [Work log — Track A](../_summary/work_log.md), [spec11](docs/spec/11_mac_driver_replan.md).
3. [Integration report](docs/qa/session_adapter_integration.md), [evidence](docs/qa/session_adapter_evidence.json), [artifacts](docs/qa/session_adapter_artifacts.json).

Mục tiêu lượt tới: xử lý LOG_011 flip, giữ bản vá LOG_010 và UX đã có trong source, sau đó tạo **một candidate Test mới** để QA. Giữ Funput Rust/C ABI, Tauri UI, nguồn nhập ABC và hướng session tap + return-original. Không viết lại engine hoặc quay về AX text proof bắt buộc.

Không commit/push/release/cài Applications hoặc triển khai Windows. Release1.0.1 vẫn dừng; Windows PLAN ONLY. Track dài hạn MỞ, chỉ user đóng. Working tree có nhiều thay đổi của các lượt/agent trước: đọc diff, không reset/clean/blanket-stage hoặc ghi đè phần không liên quan. Không sửa runtime đang chạy để thử chẩn đoán giữa chừng.

## Checkpoint đã kiểm lại lúc bàn giao

**Rollback đã chuẩn bị:** đọc [checkpoint_rollback.md](checkpoint_rollback.md). Snapshot source hiện tại (gồm dirty/untracked) và bundle Test đã được kiểm restore/hash tại `build/checkpoints/pre_anti_2026_11_04`. Giữ checkpoint chỉ đọc, không cleanup/overwrite; source hỏng thì restore có chọn lọc, không quay về HEAD hoặc reset sạch. Quyền OS không nằm trong backup.

- App đang chạy: `build/session_integrated_candidate/desktop_candidate/VXKey.app`, hiển thị VXKey Test, ID `com.vxkey.test.desktop`; helper nhúng ID `com.vxkey.test.engine`. Data riêng `build/si_data`.
- Process snapshot: companion55110/helper55122, không thấy MKey chạy. Đây là snapshot, phải kiểm lại trước native QA. Last bridge transition: attached=true/suspended=false/fault=null. Không coi log transition là heartbeat trực tiếp.
- Hash app/helper vẫn khớp artifacts. **Không rebuild/re-sign/ghi đè candidate này**, spike v6 đãgrant hoặc production. Dùng root/data/report revision mới.
- Autostart/clipboard/updater production OFF. Exclusion TextEdit thử đã gỡ. Config Test revision5 có memory TextEdit=vi, macro nháp `vxqa`, flip Ctrl+Option+F5. Flip này **Codex gán tạm để QA**, factory config vẫn `flip:null`.
- Build mặc định vẫn driver legacy; module mới chỉ chọn bằng `VXKEY_SESSION_ADAPTER=1`. Chưa promote/default switch hoặc đổi contract production09/10/KEY01.

## Đã đạt — giữ làm regression

| Scope | Bằng chứng |
|---|---|
| Telex TextEdit/Figma canvas nháp | User xác nhận `as af aa ngaan hangf casch cachs ddwojwc w vxqa ` đúng, macro dài/emoji đầy đủ |
| TextEdit boundary | Cmd+A→xóa→gõ ngay `ddwojwc casi` và Shift+Enter→`ddwojwc mwojwt` đúng |
| App exclusion | TextEdit giữ raw `ddwojwc vxqa `, không macro; Figma vẫn ra `được`. UI/helper2/2, sau gỡ rule3/3 |
| Modifier toggle/basic Backspace | User xác nhận Ctrl+Option VI→EN→VI đúng; `chaof`, Backspace, `s`→`chá` đúng. Chưa full retone matrix |
| Z | User báo gõ dấu rồi bỏ dấu bằng Z đang đúng; chưa có exact sequence/all-tone matrix |
| Illustrator | User báo đang gõ ổn; chưa xác nhận cùng prescribed corpus |
| Lifecycle | Pause tháo hook, Resume ACK bật lại; Cmd+Q từ active dừng app/helper, ps không orphan. Reopen cùng binary giữ quyền, nhưng gặp LOG_010 |

Chưa chốt daily-use, mọi app/layout, ChatGPT first-word, native VNI/advanced/retone đầy đủ, integrated sentinel/autocomplete/undo/rapid-focus, parent crash/sleep-wake hoặc updater N→N+1. Không suy các phần này từ số tests.

## Thứ tự công việc và độ khó

### A1 — LOG_010: giữ bản vá có sẵn, đưa vào candidate mới — nhẹ

Đã tái hiện: lúc mở lại, PermissionCache chưa có observation, ready=false bị gửi thành permission_required ở tick đầu. Sau AX=true vẫn còn recovery latch, phải Resume. Đây là lỗi phân biệt unknown/denied, không phải bằng chứng grant bị mất.

**Source đã sửa, chưa nằm trong bundle đang chạy:**

- [permission_cache.swift](native/macos_background/permission_cache.swift): observed/denied; unknown giữ ready=false nhưng denied=false.
- [session driver](native/macos_session/session_event_driver.swift): permissionDenied từ cache.
- [runtime settings](native/macos_background/runtime_settings.swift): stream chỉ báo permission_required sau quan sát thực. Legacy branch giữ nguyên.

Regression hiện tại: [104Swift checks](docs/qa/session_adapter_startup_fixture.log), [6bridge gates](docs/qa/session_adapter_startup_control.log), hook/quyền trong bridge fixture là mock. Không bỏ gate quyền thật hoặc bỏ Pause/fault để làm startup xanh.

### A2 — LOG_011: sửa delivery của flip trên special key — khó hơn, cần review kỹ

User nhấn phím thử Ctrl+Option+F5 thấy menu gợi ý “pas”; chưa có exact event trace/final text. Source defect rõ: flip branch xóa modifiers rồi StreamDelivery trả event vẫn mang virtual keycode F5. Apple mô tả [NSTextView.complete](https://developer.apple.com/documentation/appkit/nstextview/complete%28_%3A%29) mặc định gọi bằng F5, phù hợp triệu chứng. Không kết luận core Funput flip sai từ riêng menu này.

Source chính: [session_event_driver.swift](native/macos_session/session_event_driver.swift), [stream_delivery.swift](native/macos_session/stream_delivery.swift), [stream_composition.swift](native/macos_session/stream_composition.swift), [fixture](native/tests/session_adapter/main.swift).

Yêu cầu giải pháp:

- Phân biệt trigger phím văn bản và trigger special key. Khi flip được xử lý, editor không được nhận nhầm lệnh completion/navigation/shortcut.
- Nếu đổi keycode/event, phải giải quyết keyDown/keyUp và autorepeat, cùng reset/Pause/focus/Quit giữa lúc giữ phím. Không chỉ đổi keycode của keyDown rồi bỏ qua pairing.
- Chuẩn bị đầy đủ delivery trước deletion; lỗi chuẩn bị không được xóa trước rồi mất output. Không sửa lấn prefix/suffix/selection ở tài liệu nháp.
- Không thêm AX proof/IPC/sleep hoặc I/O vào đường gõ thường. Giữ route normal typing đã native PASS; không dùng app whitelist, sửa setting TextEdit/Figma/macOS, đổi binding để che lỗi F5, hoặc cắt feature rồi báo parity.
- Chốt hypothesis nhỏ, kiểm riêng delivery special-key nếu cần, sau đó native flip với engine thật. Fixture Unicode/core không thay client render.
- Có thể tham vấn Codex chỉ đọc theo global AGENTS khi user giao Anti chủ trì; câu hỏi cụ thể, không chuỗi gọi agent, giữ giới hạn/bộ đếm hiện hành. Nếu chưa tìm được delivery đúng, báo blocker của flip riêng; không tuyên bố đã sửa vì mất menu.

### A3 — Gộp UX source có sẵn, kiểm và build candidate mới — trung bình

Đã có source-only, chưa trong frozen bundle:

- [permission_setup.tsx](desktop/src/components/permission_setup.tsx): không tự đánh Step3 hoàn thành chỉ vì hook active; hướng dẫn external editor đúng thực tế.
- [apps_tab.tsx](desktop/src/components/apps_tab.tsx): nhãn xóa rule nói rõ hành động/app ID.
- Frontend đã build PASS. Không redesign toàn bộ Settings trong lượt engine này.

Shortcut custom parsing có trong config/driver nhưng Settings chỉ hiển thị binding. UI tùy chỉnh vẫn là nợ riêng, chưa được tính full parity; ưu tiên candidate engine trước, không âm thầm claim UI custom đã xong.

## Checks và build

Chỉ chạy lại checks liên quan sau sửa; logs mới, không ghi đè evidence native cũ:

```sh
sh scripts/test_session_adapter.sh
VXKEY_SESSION_ADAPTER=1 python3 scripts/test_background_bridge.py
cargo +stable test -p vxkey-runtime --offline
node scripts/test_native_permissions.mjs
npm run build --prefix desktop
cargo +stable check --manifest-path desktop/src-tauri/Cargo.toml --features custom-protocol --offline
```

CGEvent RAM fixture cần WindowServer; bridge fixture cần Unix socket. Dùng cơ chế quyền tool hợp lệ nếu sandbox chặn; không thay tests bằng mock thêm để né lỗi. Không phát keyboard events vào tài liệu thật để chạy tests.

Sau khi A1/A2/source checks đạt, xem lại các script trước build:

- [build_native_candidate.sh](scripts/build_native_candidate.sh): bắt buộc env `VXKEY_SESSION_ADAPTER=1` và root mới.
- [build_desktop_candidate.sh](scripts/build_desktop_candidate.sh): env `VXKEY_TEST_BUILD=1` để tách identity/updater.
- [prepare_bridge_candidate.py](scripts/prepare_bridge_candidate.py): cùng root, test flag, QA data root ngắn riêng, report artifacts mới. Socket path tối đa103bytes.

Gợi ý root `build/session_followup_candidate`, data `build/sf_data`; **kiểm chưa tồn tại**, nếu trùng chọn revision khác. Seed config từ example, không copy clipboard/history/user text; autostartOFF, clipboardOFF, startup paused. SDK27 theo chuẩn workspace; ad-hoc/pins chỉ Test, không là retention proof production.

Quyền OS chỉ đúng app Test/scope đã được user giao; không reset TCC toàn máy, đổi production entry hoặc keychain/trust. Touch ID/mật khẩu user tự nhập tại OS. Không chạy hai bộ đang xử lý tiếng Việt cùng lúc. Kiểm tool có thật trước khi nhận làm native QA; phím do automation gửi thẳng TextEdit trước đây không đi qua session tap, nên cần physical typing cho acceptance.

## Ba gate nghiệm thu trước bàn giao lại

1. **Startup:** unknown→grant không latch giả; observed denial vẫn chặn; Pause/fault thật không tự Resume. Candidate mới mở lại nhận quyền đúng, không phải chỉ source PASS.
2. **Flip:** `pass`→flip ra raw `pass`, không completion/menu, không phím thừa hoặc mất sentinel; thử nhấn/giữ/nhả special key và đổi focus/Pause. Core test, event fixture, native output báo riêng.
3. **Regression:** prescribed TextEdit/Figma corpus + macro + first-word/reset, exclusion, Ctrl+Option và basic Backspace vẫn đúng; Quit không orphan. Nợ matrix còn lại ghi rõ, không tự promote/release.

Output: diff/check logs, artifact+hash mới, native outcomes/gaps, cập nhật report/evidence/coordination và **Track A trong work log**. Chỉ user được đóng track. Không dùng handoff file mới để reset giới hạn CLI các task trước; lượt bàn giao này Codex không gọi CLIagent.
