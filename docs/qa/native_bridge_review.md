# VXKey — bridge IME/companion và phục hồi nhập liệu

Ngày: 2026-10-30. Owner: Codex. Phạm vi user giao: bước1 (nối engine/companion) và bước2 (ngừng xử lý khi lỗi, safe start); Mac trước. Không gọi Anti. **Đã có implementation và fixture/native-process subset; chưa nghiệm thu gõ trong document hoặc daily-driver.**

## Quyết định triển khai

Bài toán là truyền cấu hình/trạng thái thật qua ranh giới process mà key callback không phụ thuộc WebView, disk hay companion response. File polling cũ thiếu identity/ACK; XPC cần thêm lifecycle/service packaging; chọn Unix socket có audit-token/code-signature checks, native immutable cache và supervisor độc lập. Clipboard không nằm trong transport hoặc input callback.

**Safety profile hiện tại:** mất bridge hoặc lỗi main loop → ngừng biến đổi, giữ marker qua restart, chỉ explicit Resume sau ACK mới bỏ marker. Điều này thay thế đề xuất cũ cho phép tiếp tục gõ từ offline cache khi companion chết. Đổi lấy việc phải mở companion/Resume để gõ tiếng Việt trở lại; close Settings chỉ hide window, không quit companion. Khi callback còn chạy, unsupported/suspended input trả false để OS chuyển tiếp. Không hứa recovery tuyệt đối khi OS/client/driver treo.

## Source chính

- [Protocol/projection](../../crates/vxkey-runtime/src/bridge.rs): framed JSON2MiB, version/session UUID, directional sequence, strict serde DTO/unknown-field rejection; native projection không UI/history/clipboard/history-hotkey. Validate config và physical codes Mac trước apply; null toggle là tắt shortcut, không tự khôi phục default.
- [Transport/supervisor/latch](../../crates/vxkey-runtime/src/bridge/local.rs): runtime directory0700/socket0600, single listener lock, bounded wire deadlines, queue8, control expiry2s, reconnect1/2/4/8/16/30s. Client connection TTL độc lập main timer; main ticks không kéo dài kết nối khi worker không còn nhận response.
- [Peer trust](../../crates/vxkey-runtime/src/bridge/trust.rs), [public macOS shim](../../crates/vxkey-runtime/native/macos_trust.c): same UID + LOCAL_PEERTOKEN → kSecGuestAttributeAudit → SecCodeCheckValidity. Hai đầu xác minh peer trước handshake/data, không lookup identity qua PID. Production requirement có Apple anchor + bundle ID + cùng Team ID đã verify own identity. Chưa chạy bằng production signer; default fail closed khi không có trust.
- Development fixture dùng opt-in `VXKEY_BRIDGE_DEV_TRUST=1`, private `bridge_pair.json` CDHash pins, own-code check và peer check. Chỉ debug hoặc feature `development-bridge`; candidate FFI dùng feature `vxkey-native-dev`. Ad-hoc pins không phải production identity/TCC retention. Rebuild một binary thì phải chuẩn bị lại pair, không sửa pin khi process đang chạy.
- [Native FFI](../../crates/funput-ffi/src/vxkey_native.rs): bridge startup/poll/free/source-active/fault flag; poll trên main timer100ms, try_lock, không socket/file wait trên main/input path. [ABI guard](../../crates/funput-ffi/src/abi/guard.rs) có fault flag riêng cho VXKey; engine panic giữ bypass tới process restart. Native panic hook không in payload có thể chứa input text.
- [AppSettings](../../native/macos/runtime_settings.swift): bỏ config-file polling/fallback cũ; chỉ nhận validated snapshot. Hoãn toàn bộ config/macro table khi còn composing; ACK sau cache + active engine sync. Active IMK client/PID/session/focus generation là target; giữ original external target khi UI VXKey lấy focus, invalidate khi sang app khác. Main-loop lease hết hạn thì pass-through trước khi timer có cơ hội chạy lại.
- [Controller](../../native/macos/input_controller.swift): xử lý riêng trên monitored main thread; unexpected off-main delivery trả false, không sync-dispatch. Hard bypass/suspension trước Telex/VNI/macros/retone/toggle; finalize phần preedit đã hiển thị khi còn an toàn, không macro expansion trong cleanup. Candidate vẫn ép preedit, chưa bật direct document writes.
- [Service](../../crates/vxkey-runtime/src/service.rs): publish live runtime/applied revision, per-app memory writeback có base revision/exact active target; quick exclusion token bound IME session/focus, giữ clipboard flag. Source khác VXKey chưa có OS-frontmost token adapter, nên quick exclusion hiện chỉ có native target khi IME active. Không nhận current-app identity từ label WebView.
- [Tauri host](../../desktop/src-tauri/src/lib.rs): native commands validate surface/fields/session; request/ACK chạy worker ngoài service mutex và UI event loop. Native app menu Pause/Resume + NSAlert không phụ thuộc JavaScript; tray cũng có Pause/Resume. App-owned supervisor thử chọn ABC tối đa một lần mỗi incident khi current source đúng VXKey, không enable source mới, kill/relaunch process hoặc đổi TCC. **Nhánh chọn ABC thực sự chưa test vì VXKey chưa được activate.**
- [System tab](../../desktop/src/components/system_tab.tsx): connection/source/processing/applied-vs-saved metadata vi/en; không coi saved config là runtime success. Native tray glyph chỉ V/E/Ư/W khi live active/non-bypassed, còn lại neutral VX. Autostart giữ OFF/unavailable.

## Kiểm chứng

1. [Cargo regression](native_bridge_cargo_tests.log): core/engine/config/convert/FFI/runtime, gồm native panic guard và bridge policy tests. Tổng cuối được ghi trong [summary](native_bridge_summary.json), không cộng process cases thành unit tests.
2. [Hai process có ký/pin](native_bridge_evidence.json), [log](native_bridge_process_tests.log): hai chiều từ chối same-UID peer giả trước data; mismatched CDHash; handshake/live mode; deferred apply/ACK; stale target; memory writeback; freeze main ticks trong khi Rust worker còn sống; durable latch/reconnect/no auto-resume; explicit same-session Resume; companion crash; restart config/marker. Mode/target là fixture, không actual foreground typing.
3. [Swift adapter thật](swift_bridge_evidence.json), [log](swift_bridge_tests.log): AppSettings/Timer và Rust FFI composer thật; chỉ controller stub để không đăng ký IMK. Config/macro apply và VNI hoãn khi composer có buffer; block chính Swift RunLoop → supervisor phát hiện; explicit Resume có ACK. Không document edits.
4. [Native composition fixture](native_bridge_composition_tests.log) và2macro-loader frontend regression PASS. Fixture không thay stale-range/Undo/client matrix.
5. [Build pair](native_bridge_build.log), [artifact pins/hash](native_pair_artifacts.json): SDK27, Apple Silicon, TypeScript/vendor/Vite/Tauri/Swift build, codesign strict trên ad-hoc development pair. Không installer/release.

### Native app pair đã quan sát

Hai app được mở từ build folders, IME không được install/register/select. CUA gọi headless getApp timeout nhưng process thực sự chạy; không lặp lệnh đó. Companion native UI xác nhận:

- [Connected vi](native_bridge_native/pair_connected_vi.jpg) và [AX](native_bridge_native/pair_connected_vi.txt): ready, source inactive, applied/saved20/20.
- Đổi tone style từ modern sang traditional qua Settings: [21/21](native_bridge_native/pair_revision_21_ack.jpg), [AX](native_bridge_native/pair_revision_21_ack.txt). Restore modern quan sát21/22pending → [22/22](native_bridge_native/pair_revision_22_restored.txt).
- Native menu Suspend có [ACK](native_bridge_native/native_suspend_ack.txt), [marker readback](native_bridge_native/suspend_marker.json), [runtime suspended](native_bridge_native/pair_suspended_vi.jpg); Resume có [NSAlert](native_bridge_native/native_resume_ack.jpg) + [AX](native_bridge_native/native_resume_ack.txt), marker được xóa sau ACK và status về ready.
- [English status](native_bridge_native/pair_connected_en.jpg), [English native suspension](native_bridge_native/native_suspend_en.jpg), [AX](native_bridge_native/native_suspend_en.txt). Cuối lượt trả vi/system/modern,124macros, autostartfalse; giữ explicit user_suspended marker khi đóng QA.

Fingerprint đúng pair đã quan sát nằm trong [observed artifacts](native_bridge_native/observed_pair_artifacts.json). Sau native quan sát có hardening TTL/bridge_stale latch, required nullable command field, supported physical keys, deactivated-controller guards và main-thread guards; pair cuối được rebuild và fixture regression chạy lại. Không gán screenshot của pair cũ cho SHA mới hoặc ghi final pair runtime acceptance. [Summary](native_bridge_summary.json) tách hai mức.

## Lệnh tái hiện

```sh
cargo +stable test -p funput-core -p funput-engine -p funput-config -p funput-convert -p funput-ffi -p vxkey-runtime --features funput-ffi/convert,funput-ffi/vxkey-native --offline
cargo +stable build -p vxkey-runtime --bin vxkey_bridge_fixture --offline
sh scripts/build_bridge_adapter_fixture.sh
python3 scripts/test_native_bridge.py
python3 scripts/test_swift_bridge_adapter.py
sh scripts/build_bridge_candidate.sh
```

Socket fixture cần quyền bind Unix socket của môi trường chạy; sandbox hiện chặn bind, lượt test đã được chạy với approval đúng script. Không thêm grants toàn máy hoặc đổi TCC/keychain. Scripts chỉ ghi build fixture/docs QA và dọn child do mình tạo; không sử dụng data root thật.

Candidate UI: `build/desktop_candidate/VXKey.app`; input process: `build/native_candidate/VXKeyInput.app`. Hai app dùng `build/qa_integration_data`, marker hiện giữ suspended. Mở Settings không tự đăng ký/chọn input source. Không dùng ad-hoc này để cài đè identity production, không copy vào /Applications hoặc ~/Library/Input Methods trong lượt này.

## Gate còn mở và bước tiếp theo

- Controlled install/register/activation + typing trong TextEdit/AppKit, browser, Electron, secure fields; native session/PID/client/focus races, modifier-only release, macro/retone, input latency, selection/UTF16/Undo.
- Fault injection khi actual IMK client đang gõ/treo; chứng minh main-thread delivery, ABC selection/readback, menu recovery khi WebView hỏng, source chuyển bằng chuột, sleep/wake. Timer fixture không thay các case đó.
- Production Team/DR/anchor, process identity qua update, paired installer/quiesce/rollback; pin ad-hoc fixture chưa đóng ID01.
- OS-frontmost/panel-session tokens khi source ABC, local memory conflicts/durable outcomes, custom hotkey recorder/registration/history ownership, period replacement edit/Undo; clipboard capture/consent/provenance/encryption chưa làm.
- Windows/Intel chưa compile/run. Disk/fsync failure/hang, native IMK synchronous client hangs và real event latency còn phải fault-test; không tuyên bố mọi I/O hoặc recovery branch đã bounded trên OS.

M2 đạt implementation + process/adapter/native metadata subset; **M2/KEY10 production/native typing acceptance vẫn mở**. Không release/commit/push, không kích hoạt IME, không đổi bộ gõ hiện tại.

Tham chiếu primary: [Apple SecCodeCopyGuestWithAttributes](https://developer.apple.com/documentation/security/seccodecopyguestwithattributes(_:_:_:_:)), [Apple audit-token key](https://developer.apple.com/documentation/security/ksecguestattributeaudit); LOCAL_PEERTOKEN/TIS APIs đối chiếu SDK27 public headers. Binding audit token tránh PID lookup/reuse; không suy ra production trust từ socket permissions.
