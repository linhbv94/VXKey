# Session adapter — bản tích hợp cách ly

Ngày: 2026-11-04. Owner: Codex. Track dài hạn **MỞ, chỉ user đóng**.

User giao triển khai sau pilot v6. Giữ Funput Rust, ABC và Tauri UI. Không gọi agent; Windows PLAN ONLY; release1.0.1 vẫn dừng. Không thay bundle production hoặc spike v6 đã cấp quyền.

## Phần đã triển khai

- [Session driver](../../native/macos_session/session_event_driver.swift): một session tap, Unicode trên keyDown vật lý gốc, chỉ tổng hợp Backspace hữu hạn. Config/target/policy đọc từ cache; không đọc AX text hoặc chờ IPC trong callback. Loại trừ app và Secure Input đi trước toggle/macro/composer. Reset khi focus/mouse/scroll/layout/navigation/Cmd shortcut/repeat thay đổi.
- [Composition](../../native/macos_session/stream_composition.swift): ACTION_SEND và ACTION_RESTORE, Telex/VNI/advanced, restore tiếng Anh, macro VI/EN, retone/flip, word-space đã trực tiếp quan sát, auto-capitalization. Event history tối đa128scalar, không nhận lại text từ document. [Delivery](../../native/macos_session/stream_delivery.swift) dựng đủ deletion/payload và round-trip Unicode trên bản sao trước mutation; tối đa8192UTF16, không cắt payload ở20unit.
- [Run policy](../../native/macos_session/run_policy.swift): không dùng lease500ms. Không được khởi động offline bằng config chưa xác thực. Sau khi đã được host xác thực/Resume, mất control giữ trạng thái/config cuối; Pause và fault thật vẫn chặn. Config mới chỉ áp dụng ở boundary.
- [Bridge](../../crates/vxkey-runtime/src/bridge/local.rs) thêm role `engine_stream`, cùng peer trust/version/session checks; bỏ fault giả bridge_disconnected riêng role này, giữ Pause/permission/engine/tap/main-loop faults và incident ACK. Pending control bị bỏ khi disconnect, không replay. Legacy giữ cách xử lý cũ.
- [Companion owner](../../desktop/src-tauri/src/lib.rs): watchdog không kill helper stream còn sống vì telemetry stale; child exit thật vẫn latch. Quit/update vẫn reap đúng child; helper main giữ parent-death check và per-data-root lock. Chưa có native fault-injection đầy đủ.
- [Permission state dùng chung](../../desktop/src/services/native_permissions.ts): capability `native_session_stream` do peer đã xác thực báo; stream yêu cầu AX, legacy giữ gate cũ. Áp dụng onboarding/Settings/tray/ô test và backend core test. Stream request chỉ mở Trợ năng; không gọi request Listen/Post. Preflight hai dịch vụ vẫn là diagnostics. Tap tạo được/typing đúng là gate độc lập.

## Build và bằng chứng

Candidate: `build/session_integrated_candidate/desktop_candidate/VXKey.app`, hiển thị VXKey Test; IDs `com.vxkey.test.desktop` và `com.vxkey.test.engine`. Data riêng `build/si_data`; autostartOFF, clipboardOFF, updater productionOFF, khởi động suspended. Config từ example, thêm macro nháp `vxqa` và flip Ctrl+Option+F5 phục vụ QA; không copy history/clipboard/user text.

[Artifacts/hash](session_adapter_artifacts.json) · [evidence hiện hành](session_adapter_evidence.json) · [baseline trước build](session_adapter_baseline.json).

| Gate | Kết quả và giới hạn |
|---|---|
| Swift core/composition/delivery/run policy | [96checks PASS](session_adapter_fixture.log) tại build; source mới [104checks PASS](session_adapter_startup_fixture.log) gồm unknown/granted/denied permission observation; event RAM, không hook/post phím. Telex/restore/VNI/numpad/advanced/retone/flip/macro dài VI/EN/sentinel/reset; policy disconnect/Pause/fault/Resume. |
| Rust runtime | [59tests PASS](session_adapter_runtime_tests.log):4unit,16bridge,39policy; có role/version negotiation, AX-only attached và backend test gate. |
| UI permission matrix | [145assertions PASS](session_adapter_ui_permissions.log), [runner](../../scripts/test_native_permissions.mjs); legacy/stream/unknown × connected/AX/listen/post. |
| Hai process Swift/Rust thật | [5gates PASS](session_adapter_control.log) tại build; source mới [6gates PASS](session_adapter_startup_control.log) thêm startup pending permission không latch, [runner](../../scripts/test_background_bridge.py) với `VXKEY_SESSION_ADAPTER=1`:3vòng Resume/Pause, host exit>lease vẫn attached, reconnect cùng helper, Pause không tự Resume. **OS hook/permission mock**, không phải gõ thật. |
| Legacy source | [Compile fixture default không flag PASS](session_adapter_legacy_compile.log); không chạy hook hoặc thay bundle legacy. |
| Frontend/backend | [Frontend](session_adapter_frontend.log), [backend cargo check](session_adapter_backend.log) PASS. |
| Bundle | SDK27 app/helper, strict ad-hoc/pins PASS; [native](session_adapter_native_build.log), [desktop](session_adapter_desktop_build.log), [pair](session_adapter_pair_build.log). Không phải signer/permission-retention production. |
| Native UI/permission | Đã mở candidate; onboarding chỉ1quyền, nút request mở đúng Accessibility. User tự Touch ID; entry Test cũ đã được xác minh qua Show in Finder và user xóa riêng. Add exact companion/helper; app nhận AX và hiện “Đã đủ quyền · Chờ kích hoạt”. User đã xác nhận MKey tạm ngừng gõ Việt. Test đã Resume ACK applied, attached=true/suspended=false/fault=null; UI hiện “Đang hoạt động · Sẵn sàng gõ”. User xác nhận physical corpus Telex, macro dài/emoji cả TextEdit/Figma và chữ đầu sau Cmd+A→xóa/Shift+Enter trên TextEdit đều đúng. Scope này PASS; chưa full parity/daily use. Current pair báo AX/listen/post=true (các grant Test có sẵn); **AX-only native proof thuộc spike v6**, không gán cho cặp bundle mới. Không xin thêm Listen/Post trong lượt này. |

## Giới hạn / gate tiếp

Module được chọn bằng `VXKEY_SESSION_ADAPTER=1` trong [build script](../../scripts/build_native_candidate.sh). **Default build vẫn dùng driver legacy.** Spec09/10/KEY01 chưa được thay bằng contract event-history. Đây là integration candidate, chưa full parity hoặc promotion.

History hữu hạn không phải document ownership đã xác minh. Client có thể tự sửa text/caret/autocomplete mà không tạo sự kiện đã quan sát; prefix/suffix trial một app không bảo đảm mọi editor. Long packet round-trip trong RAM không chứng minh client nhận đủ Unicode; flip trên function-key và retone trên Backspace cần gõ tay thật. Chưa QA native đầy đủ uppercase/layout/undo/autocomplete/secure input/terminal, mode memory, parent crash/sleep-wake hoặc fault injection. Per-app exclusion và Pause/Resume/Quit đã native PASS trong phạm vi bên dưới. Không đóng LOG_005/007/008/009 và không gọi kết quả fixture là sửa dứt điểm fade/Figma.

Tiếp tục trên **cùng binary đã cấp quyền**, không rebuild/re-sign:

1. ĐÃ ĐẠT: MKey tạm ngừng theo user; Start của Test ACK applied, actual attached=true/fault=null và UI active. Không rebuild binary đãgrant.
2. ĐÃ ĐẠT theo user: TextEdit/Figma canvas nháp corpus `as af aa ngaan hangf casch cachs ddwojwc w vxqa ` và macro đủ dài/emoji; TextEdit Cmd+A→delete→`ddwojwc casi`, Shift+Enter→`ddwojwc mwojwt` đúng. Sentinel/selection trên integrated binary còn gate riêng (spike v6 từng PASS).
3. Exclusion TextEdit trước macro đã PASS và đã gỡ rule thử. Modifier toggle/basic Backspace đã PASS; Z user báo đúng. Flip F5 gặp LOG_011; VNI/advanced/mode-memory/sentinel matrix còn mở.
4. Pause/Resume/Quit native đã PASS; reopen giữ quyền nhưng lộ LOG_010 false startup latch, bản source sửa cần candidate revision mới. Parent crash/sleep-wake còn mở. MKey trạng thái theo xác nhận user, không tự bật lại Vietnamese khi Test active.
5. Chỉ sau native acceptance mới quyết định promote/default và đồng bộ các spec liên quan. Release/signing/updater N→N+1 là gate riêng.

## Follow-up UX chỉ ở source sau khi freeze candidate

Quan sát native activation: Step3 gõ thử đang bị đánh dấu xong chỉ vì hook active; active card còn chỉ tới “ô bên dưới” dù test native đã hướng dẫn dùng external editor. Đã sửa source [permission_setup.tsx](../../desktop/src/components/permission_setup.tsx): Step3 là bước hiện tại, hướng dẫn TextEdit/app thật. **Chưa nằm trong binary đãgrant**, không rebuild chỉ để sửa copy giữa native trial. Hook/readiness activation không thay kết quả physical QA.

## Native QA bổ sung — 2026-11-04

- User xác nhận TextEdit giữ nguyên `ddwojwc vxqa ` khi input_bypass=true, không bung macro; Figma vẫn ra `được`. UI/helper applied/saved revision2/2. Đã gỡ riêng rule TextEdit qua UI, revision3/3, policy không còn trong test config; không đổi clipboard policy hoặc app ngoài.
- User báo thêm Illustrator đang gõ ổn; đây là phản hồi sử dụng, chưa phải xác nhận cùng prescribed corpus/edge-case matrix.
- Pause thật: ACK applied, attached=false/suspended=true/fault=null; Resume ACK applied, attached=true/suspended=false/fault=null. Quit bằng Cmd+Q từ active; ps xác nhận cả parent51610/helper51620 đã thoát và không còn process candidate. Không gọi đây là parent-crash/fault-injection proof.
- Mở lại cùng binary, AX/listen/post vẫn true, không cấp lại quyền. **Phát hiện startup false-latch:** status đầu dùng cache chưa có observation, AX=false/fault=permission_required; sau đó AX=true/fault=null nhưng marker permission_required giữ Pause. Resume thủ công đã bật lại. Không phải bằng chứng mất grant, không coi reopen hoàn hảo hoặc updater retention đã đạt.

### Sửa startup chỉ ở source

[PermissionCache](../../native/macos_background/permission_cache.swift) thêm observed/denied: unknown không cho chạy nhưng cũng không bịa permission_required. Session driver/runtime chỉ báo fault quyền sau quan sát thực; observed denial vẫn latch.104Swiftchecks và6gates bridge2process với hook mock PASS, gồm pending→grant không tạo marker; chưa build hoặc native accept binary mới. **Candidate đang chạy vẫn là binary cũ**, không re-sign/rebuild tại path đãgrant.

### Gate tiếp / UX còn nợ

Physical Control+Option toggle VI→EN→VI và `chaof`→Backspace→`s`→`chá` PASS theo user. Đây là basic Backspace scope, chưa đủ full retone matrix. User báo thêm bỏ dấu bằng Z hoạt động đúng; chưa có exact keystrokes hoặc all-tone matrix. Flip F5 chưa PASS (LOG_011 bên dưới). VNI/advanced và phần matrix còn lại chưa nativeaccept. Shortcut custom parsing đã có ở engine/config nhưng Settings hiện chỉ hiển thị binding; UI customization còn nợ, không claim full feature parity.

Nút xóa app rule đang có accessible name chung “Thao tác” dù click là xóa ngay. Đã sửa source title/aria-label thành hành động xóa + app ID, cùng follow-up onboarding; chưa nằm trong frozen binary.

## LOG_011 — Flip trên phím F5 làm hiện word completion

User hỏi Ctrl+Option+F5 dùng làm gì và báo sau khi nhấn xuất hiện list gợi ý cho “pas”; hai ca toggle/basic Backspace còn lại đúng. Binding này **do Codex gán tạm riêng test config**, không phải factory default: [config example](../spec/config_example.json) vẫn `flip: null`. Không coi user đã yêu cầu dùng F5 làm default.

Source defect đã kiểm: [flip branch](../../native/macos_session/session_event_driver.swift) gọi composition.flip(), xóa modifier flags rồi deliver; [StreamDelivery](../../native/macos_session/stream_delivery.swift) đặt Unicode nhưng giữ nguyên virtual keycode F5. Vì vậy đường chuyển shortcut thành text vẫn có thể kích hoạt hành động phím đặc biệt ở client. Apple xác nhận NSTextView [complete(_:)](https://developer.apple.com/documentation/appkit/nstextview/complete(_:)) mặc định được gọi bằng F5; phù hợp menu user báo. Chưa có exact native event trace để kết luận mọi chi tiết Fn/matching hoặc final output; không ghi core flip FAIL từ riêng UI completion.

Trạng thái: **MỞ, flip native chưa nghiệm thu**. Không sửa core từ phỏng đoán, không đổi setting TextEdit/macOS để né. Cần sửa delivery cho shortcut nguồn không phải phím văn bản, giữ pairing keyDown/keyUp và preflight trước deletion, rồi native test trên revision mới. Không được lấy fixture core flip/Unicode RAM làm bằng chứng client đã nhận text. Candidate đãgrant giữ nguyên; tạm không dùng phím F5 thử nghiệm. Không giới hạn app hoặc ảnh hưởng đường gõ thường để chữa shortcut này.
