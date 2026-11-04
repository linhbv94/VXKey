# VXKey — background driver migration

Ngày: 2026-11-02. Owner: Codex. Không Anti/commit/push/release.

## stream_candidate gõ tay vẫn FAIL — chuyển sang probe contract

User báo `tiêesng vieejt` / `tieesng vieejt`. Trace seen5/rewrite1/verified0/failed1, unified reason `edit_verification_timeout` lúc16:03:16.665. Đã native Pause ACK; bản có quyền hiện tại vẫn stream_candidate, không tiếp tục gõ sai. Chuyển batch sang proxy chưa giải quyết native failure.

Chuẩn bị filter_candidate với probe độc lập Funput/batch: chỉ TextEdit fresh AX, z→nil, x→X trên event trả về, không post synthetic events. [Playbook đầy đủ](filter_probe_playbook.md), [31 fixture checks](background_edit_evidence.json), [test log](filter_probe_tests.log), [build](filter_pair_build.log), [artifacts](filter_pair_artifacts.json). Probe chỉ bật bằng private QA mode file + dev bridge env, mặc định code OFF. Đây là kiểm chứng tầng filter, không phải tuyên bố đã sửa tiếng Việt. Cần approval đúng candidate trước khi thay entry OS.

## stream_candidate đã mở — chờ gõ tay

User duyệt thay/bật riêng entry VXKey cho stream_candidate. Đã Quit bản cũ, thay qua Settings UI và đọc toggle ON. Native status AX/PostEvent/listen=true, Resume ACK. Đóng alert xác nhận, chờ user click foreground TextEdit Untitled6 thật và gõ tay `tieengs Vieetj`. Không rebuild sau grant; chưa có typing acceptance. [Artifacts](stream_pair_artifacts.json), [evidence](typing_native_evidence.json).

## Gõ tay đã tái hiện lỗi thật — trạng thái hiện hành

User gõ tay và báo `tieêsng Vieejt` / `tieengs Vieejt`. Trace của tap_resume_candidate xác nhận callback nhận5keydown, rewrite1, verified0 rồi driver_fault/failed1; unified log reason generic edit_or_prepare_failed lúc15:56:14.735. Như vậy **native typing FAIL thật**, không chỉ là giới hạn automation. Đã Pause ACK qua menu native; không để bản đang gõ sai tiếp tục chạy live.

Source chuyển cả hai rewrite paths từ postToPid sang `event.tapPostEvent(proxy)` trong callback, không lưu proxy. Header CoreGraphics SDK xác định event được chèn tại đúng điểm của tap và trước event trả về; chưa coi đây là atomic application ACK. Đồng thời bỏ pass-through phím mới mà vẫn giữ pending owned state: chưa verify trước key tiếp → fault/pass, không tiếp tục edit từ proof cũ. Verification timeout có reason riêng. Đây là bản vá theo evidence, chưa xác nhận hết lỗi trong native thực tế.

Bản kế tiếp riêng `build/stream_candidate/desktop_candidate/VXKey.app`; [build](stream_pair_build.log), [artifacts](stream_pair_artifacts.json), [28 Swift checks](background_edit_evidence.json), [test log](stream_edit_tests.log). Không ghi đè tap_resume_candidate có quyền. Trước native trial phải có approval đúng candidate nếu thay entry. [Evidence user + trace](typing_native_evidence.json). Cần user foreground TextEdit thật và gõ tay; CUA targeted keys chỉ kiểm khi chứng minh foreground đúng.

## Tiếp tục — tap_resume_candidate có quyền, cần foreground thật

User đã cho phép thay/bật đúng tap_resume_candidate. Agent đã Quit bản cũ rồi thay riêng entry VXKey qua Settings UI; AX/PostEvent/listen=true. Không rebuild. Native Resume→automated Telex→Pause ACK, trace failed0/fault=null/seen0/rewrite0. Lượt này không lặp tap_disabled_by_input; chưa đủ để kết luận hết lỗi lâu dài.

**Phát hiện test precondition không đạt:** CGGetEventTapList metadata cho helper23532 có enabled per-PID taps target8552=ChatGPT; TextEdit là PID510. CUA click và exposed Raise vẫn không đổi target. Công cụ gửi phím vào TextEdit nhưng foreground hệ điều hành vẫn là app khác. Vì vậy raw Telex trong test tự động không đủ kết luận driver/engine thất bại; không được coi là successful native typing. Các raw trials lịch sử cần cùng lưu ý foreground này, còn các fault đã ghi vẫn là fault thật.

Đã Resume lần hai trên cùng binary, đang chờ user click TextEdit Untitled6 thật và gõ tay `tieengs Vieetj`, không paste. Đây là bước cần thiết để phân biệt routing automation với hook runtime. Không xin đổi quyền/build mới. MKey giữ xác nhận user tạm ngừng; autostart OFF. [Evidence](typing_native_evidence.json).

## Checkpoint — user yêu cầu dừng task, 2026-11-02

**PAUSED BY USER. Chỉ tiếp tục khi user yêu cầu.** Bản đang mở/có quyền là `build/tap_candidate/desktop_candidate/VXKey.app`, driver đã native Pause ACK (alert xác nhận còn mở). MKey giữ trạng thái tạm ngừng theo xác nhận user; không thao tác MKey/layout/app khác. Không bật autostart, không Input Source, không commit/push/release hoặc chạy Anti.

User đã duyệt thay entry riêng VXKey cho tap_candidate; thực hiện qua Settings UI, toggle ON; AX/PostEvent/listen=true. Native Resume attached=true nhưng TextEdit Untitled6 vẫn raw `tieengs Vieetj`; trace seen0/rewrite0/failed1. Unified log xác nhận `2026-11-02 12:11:20.898 VXKeyEngine[97381] driver_fault reason=tap_disabled_by_input`. Generation guard chưa giải quyết failure này. **Không có Vietnamese typing PASS.**

Đã xác định một nguồn notification trong code: prepare tạo tap (mặc định enabled) rồi tự disable trước readiness; notification có thể được giao sau Resume, lúc running=true, và bị latch thành fault. Bản vá bỏ disable tại prepare (callbacks pass-through khi running=false), Pause/teardown invalidate port thay vì disable/reuse. Chưa khẳng định đây là nguyên nhân duy nhất hoặc native đã hết lỗi.

**Bản vá đã build riêng, chưa mở/chưa cấp quyền/chưa native test:** `build/tap_resume_candidate/desktop_candidate/VXKey.app`. [Build](tap_resume_pair_build.log), [hashes](tap_resume_pair_artifacts.json), [28 Swift checks](background_edit_evidence.json), [test log](tap_resume_tests.log). Đã copy config revision26 và recovery paused từ QA root của tap_candidate; giữ exact pair pins mới. Không ghi đè binary đang có quyền. Các log HIServices Connection invalid trong fixture là warning headless; không phải bằng chứng runtime pass.

**Khi tiếp tục:** đọc source diff và artifacts; xin approval đúng tap_resume_candidate nếu cần thay entry Accessibility, không tự áp approval của tap_candidate sang binary khác. Quit bản cũ trước khi thay quyền. Sau đó kiểm tra AX/PostEvent, Resume→focus TextEdit→gõ từng phím→Pause để flush counters; đọc unified reason nếu fault. Trace chỉ ghi khi signature trạng thái đổi nên counters ở dòng Resume không đại diện kết quả sau gõ; phải chốt ở Pause/fault. Không rebuild candidate sau khi cấp quyền. Chỉ báo gõ được khi readback thật có tiếng Việt. Các gate secure/focus/Undo/latency/production signing/Windows vẫn mở.

## key_candidate sau cấp quyền — trạng thái mới nhất

User cho phép agent thay/bật đúng entry VXKey. Đã thực hiện qua System Settings UI: gỡ entry VXKey cũ, thêm đúng key_candidate, toggle ON; AX/PostEvent/listen readback đều true. Không sửa MKey, layout, app khác hoặc TCC database. Candidate đã cấp quyền giữ nguyên byte.

Native Resume ACK/attached=true nhưng sau chuyển focus driver_fault/attached=false, trước keyDown đầu tiên: seen0/rewrite0/failed1. Thử lại với TextEdit đã chọn trước Resume vẫn fault, failed2. Native Pause ACK và UI responsive. **Chưa gõ được tiếng Việt.** [Evidence hiện hành](typing_native_evidence.json). Không được quy lỗi này cho thiếu quyền, hoặc tuyên bố tap per-PID đã giải quyết lỗi.

Source mới gắn callback với generation của port; callback cũ không được xử lý như port hiện hành. Thêm unified log reason chỉ gồm nhãn cố định tap_prepare_failed/tap_not_enabled/tap_timeout/tap_disabled_by_input/edit_or_prepare_failed, không ghi phím hay text. Đây là phòng vệ lifecycle và instrumentation, **chưa xác nhận là root-cause fix**. [28 Swift checks](background_edit_evidence.json), [log](tap_lifecycle_tests.log) PASS; gồm callback generation cũ không đếm phím/không fault. Fixture không attach/post phím thật. Đã build `build/tap_candidate/desktop_candidate/VXKey.app` riêng để phân biệt failure chính xác trước khi tiếp tục sửa ([build](tap_pair_build.log), [hashes](tap_pair_artifacts.json)); không ghi đè key_candidate đã được cấp quyền.

## Lịch sử typing_candidate và chuẩn bị key_candidate

User đã cấp quyền cho typing_candidate và giữ MKey tạm ngừng gõ Việt. Đã Resume/Pause thật, AX/PostEvent/listen=true, UI vẫn phản hồi sau Resume (không lặp deadlock). TextEdit Untitled6 nhập bằng từng phím `tieengs Vieetj`, readback vẫn là Telex thô, **FAIL**, không có Vietnamese typing acceptance. Trace ghi16keydown đều missing_target,0rewrite; không phải engine chưa có quyền. [Evidence](typing_native_evidence.json).

Đã chuẩn bị `build/key_candidate/desktop_candidate/VXKey.app`: dùng public CGEventTapCreateForPid để bound callback với đúng app, không đòi eventTargetUnixProcessID (trong case vừa chạy luôn thiếu); observer focus và AX worker xác minh focused application. Source vẫn pass-through nếu chưa chứng minh ownership/range. Đồng thời C-ABI busy được phân biệt với lỗi: giữ lease cũ còn hạn, không reset composer do một try-lock miss; hết lease live mới latch. Unknown/decode/NULL vẫn fail-closed. [Artifacts](key_pair_artifacts.json), [native SDK27](key_native_build.log), [desktop SDK27/typecheck/Settings](key_desktop_build.log), [Rust checks](key_rust_tests.log), [Swift checks](key_edit_tests.log), [real cache/bridge adapter](key_adapter_tests.log) PASS. Chưa test tap per-PID trên máy sau fix; không báo success từ build/fixture.

User sau đó đã cho phép, agent đã thay/bật đúng entry key_candidate; xem trạng thái mới nhất phía trên. Không rebuild key_candidate sau approval/grant. MKey đã có xác nhận tạm ngừng, không hỏi lại hoặc tự sửa. typing_candidate hiện paused, không bật lại bản còn guard lỗi. Các ghi chú phía dưới là lịch sử theo candidate, không đổi artifact cũ thành PASS.

## Fix deadlock/icon và gate typing hiện hành — 2026-11-02

User báo icon sai và chưa gõ được. Bản resume_candidate đã có AX/PostEvent=true, native Resume ACK và tap attached=true trong trace, nhưng companion sau đó deadlock. [Stack companion](resume_companion_sample.txt) chứng minh main thread chờ Service mutex trong sync_tray, còn async command giữ mutex đó khi set_title chờ main thread. [Stack helper](permission_resume_sample.txt) trước đó chứng minh run loop không bị treo; không được tiếp tục quy mọi failure cho thiếu quyền.

Đã sửa: snapshot try_lock/thả Service và cache locks trước native renderer, dispatch UI lên main thread; nativePeriod OS query ngoài Service lock. [4 regression tests](tray_lock_tests.log) PASS: reentrant renderer lấy lại được cả hai locks, busy Service/cache không block, disconnected giữ đúng SVG/mode nhưng mờ. Icon dùng đúng V/E hoặc Ư/W cả khi suspended, alpha45%/tooltip trạng thái; không text VX hardcode. Bốn SVG root/public giống nhau byte-for-byte; tray-icon adapter đặt logical18pt.

Native callback dùng annotated-session target PID để so cache, không query NSWorkspace trong callback; publish target mới trước prime context; modifier gesture dùng flags/physical bookkeeping, không query keyState OS trong callback. Thêm diagnostics chỉ là counters (không key/text) ở dev status. [26 Swift composer/edit/gesture/injection checks](background_edit_evidence.json) PASS; [real Swift cache + Rust bridge](typing_adapter_tests.log) PASS qua3vòng Resume/Pause với hook OS mock, có precondition focus-prime order/own-UI retention. Không thay native typing QA.

**Candidate hiện hành:** `build/typing_candidate/desktop_candidate/VXKey.app`, [hashes](typing_pair_artifacts.json), [native SDK27](typing_native_build.log), [desktop/typecheck/canonical Settings](typing_desktop_build.log), [Swift fixture build](typing_edit_tests.log). Native UI đã đổi playful→serious, revision24→25→26, vẫn connected và không treo. User đang được yêu cầu cấp quyền đúng candidate mới; MKey đã được user xác nhận tạm ngừng gõ Việt. Không rebuild binary sau khi user cấp quyền trong lượt này. Chưa có Vietnamese text readback; không báo typing PASS. Tài liệu dưới giữ baseline lịch sử.

Bản companion deadlock và helper đã được dừng đúng PID/path của task; không signal app khác. TextEdit `Untitled 6` là fixture task, phím từng ký tự đọc lại `raw ok` thành công khi driver ngừng; các tab khác không sửa. Quyền/tín nhiệm OS do user tự cấp, không reset/sửa TCC/keychain, không đổi layout hoặc tự thao tác MKey.

## Tiếp tục sau khi user cấp quyền — 2026-11-02

User đã bật Accessibility. Kiểm tra native và log TCC cho thấy PostEvent vẫn denied; UI cũ gộp hai quyền nên báo thiếu quyền không rõ. Đã sửa nút request để gọi thêm CGRequestPostEventAccess ngoài input run loop, tách AX/PostEvent readback. Bản vá build riêng, giữ nguyên candidate cũ đã được cấp quyền.

Candidate hiện hành: `build/permission_candidate/desktop_candidate/VXKey.app`; [artifact hashes](permission_pair_artifacts.json), [baseline trước sửa](background_permission_baseline.json), [native build](permission_native_build.log), [desktop build](permission_desktop_build.log), [Swift checks](permission_edit_tests.log). SDK27/build/canonical Settings/typecheck và18Swift checks PASS. User đã tự thêm/bật bản vá; sau restart, UI native xác nhận **Accessibility=true, PostEvent=true**, connected/applied24/24, vẫn suspended. Chưa Resume trước khi xác nhận MKey tạm ngừng xử lý tiếng Việt.

Bản dev ad-hoc có DR theo cdhash; không có bằng chứng giữ quyền khi rebuild. Không thay signer, nới requirement, sửa/reset TCC hoặc thu hồi quyền app khác. [AX API](https://developer.apple.com/documentation/applicationservices/1460720-axisprocesstrusted), [PostEvent request](https://developer.apple.com/documentation/coregraphics/cgrequestposteventaccess%28%29) là hai API khác nhau. Các phần dưới là baseline migration trước lượt cấp quyền này.

## Kết quả và phạm vi

Theo quyết định user, VXKey là bộ gõ chạy nền, giữ nguyên nguồn nhập hiện tại. [Spec09](../spec/09_background_driver.md) thay hướng IMK/Input Source; 00–08/API schema/README/index/work log/backlog đã đồng bộ. Không cần cài/chọn VXKey Input Source. Bản IME thử do task cài đã gỡ, receipt ở `build/native_activation/install_receipt.json`; không sửa MKey/layout/TCC.

Đã có Swift session event tap trên main run loop, cache config/foreground/policy/memory, real Rust Funput C ABI, AX worker chỉ đọc đúng owned range, proof/lease/secure/bypass guards, nonce/PID event ownership và UTF-16 chunk batch. Retone chỉ trong vùng còn sở hữu hoặc từ vừa xóa Space có proof; không scan/adopt văn bản tùy ý. Numpad source được giữ. Rewrite chưa được AX xác nhận sẽ suspend, không replay. Unsupported/stale context chuyển phím gốc.

Protocol2 đổi role/session sang engine; Security peer verification giữ nguyên độ chặt, development pair pin engine+companion. Bỏ TIS source-active/select/fallback khỏi runtime active. Embedded `VXKeyEngine.app` tự mở bởi companion, có lifetime lock và parent-death guard. Main-loop/bridge watchdog và owned-child supervisor không tự restart sau fault. Pause/Resume giữ durable latch. Autostart OFF.

System tab phân biệt connection, permission, driver attached, suspended và applied/saved revision. Bỏ lựa chọn preedit/marked text. Settings vẫn là một native window, không modal shell phụ; canonical component không sửa. Nút yêu cầu quyền chỉ route từ Settings đến authenticated native session; ACK không đồng nghĩa đã được cấp quyền.

## Bằng chứng

- [Evidence/checks](background_driver_evidence.json), [artifact hashes](native_pair_artifacts.json).
- [Native SDK27 build](background_native_build.log), [desktop build + canonical Settings integrity](background_desktop_build.log), [pair packaging](background_pair_build.log).
- [1197 Cargo PASS](background_all_cargo_tests.log), gồm50VXKey runtime (12bridge+38policy).
- [18 real Swift/Rust composer/edit checks](background_edit_evidence.json), [fixture build](background_edit_tests.log): Telex, macro dài, sentinel, scalar/grapheme rejection, retone/adoption, UTF-16 chunk/surrogate. Không event tap/permission mutation.
- [11 two-process bridge checks](background_process_tests.log): exact signed-peer pin, frames/validation/target, config ACK/memory, stall/crash/latch/Resume/restart. Không native typing.
- [2 frontend regression checks](background_frontend_tests.log).
- [Native System AX readback](background_system_ax.txt), [screenshot](screenshots/background_system.png): connected, applied24/24, permission chưa cấp, suspended. Quan sát helper child của companion; Quit kết thúc cả hai. Nguồn nhập readback giữ nguyên baseline; không chọn source trong migration.

## Gate còn mở — không báo daily-driver ready

Baseline ban đầu (đã được thay thế bởi kết quả cấp quyền/test ở đầu tài liệu): máy chưa cấp Accessibility/post access cho VXKey Engine. User phải dùng **Hệ thống → Thiết lập quyền xử lý phím…**, bật **VXKey** (macOS gộp quyền helper dưới app chính) trong **System Settings → Privacy & Security → Accessibility**, rồi explicit Resume. Agent không tự cấp quyền toàn hệ thống hoặc sửa TCC. Development ad-hoc identity chưa chứng minh giữ quyền qua rebuild/update.

Sau khi có quyền: user tạm ngưng xử lý tiếng Việt của MKey trước khi explicit Resume VXKey để tránh hai bộ gõ cùng sửa một phím; không tự stop/đổi setting MKey. TextEdit + WebView thực, fast typing, focus/selection/programmatic focus race, secure field, sentinel/emoji/combining, Undo và các fault trong spec08/09. AX cache/CGEvent posting chưa là atomic app ACK; không claim bảo toàn mọi app từ fixture. UI custom shortcut recorder, flip/history registration, period native edit, clipboard capture/encryption, production signing/updater/installer, Intel/Windows vẫn là các gate riêng; không lấy stored config làm runtime success.

## Chạy candidate

```sh
open build/desktop_candidate/VXKey.app
```

Chỉ mở app này; helper nằm bên trong. Không chạy `install_native_test.py install` hoặc register source — entry point đó đã bị khóa. Rebuild bằng `sh scripts/build_bridge_candidate.sh`; phải prepare lại exact pair pins khi đổi một component. App dùng QA data root riêng, giữ marker suspended, không tự bật login item. Báo cáo IME/native_bridge trước migration là lịch sử.

## Null-event probe - 2026-11-02

User: azx -> azX. Native Pause ACK. In-place mutation works; nil did not suppress z in this trial. Whole-session counters seen149/decisions10/fault0 are not isolated three-key counts.

New candidate: `build/null_candidate/desktop_candidate/VXKey.app`. Suppress now mutates original event.type=.null and returns that event, including consumed keyDown/keyUp, shortcut and both rewrite paths. [Artifacts](null_pair_artifacts.json), [build](null_pair_build.log), [32 fixture checks](background_edit_evidence.json), [log](null_filter_tests.log). Native .null behavior NOT YET VERIFIED.

After exact binary permission approval: Quit old app, replace only VXKey entry, open/Resume new app in filter mode. User foreground TextEdit azx must produce aX. Pause and read counters. If PASS: Quit, remove only private QA file `build/null_candidate/qa_integration_data/driver_probe_mode`, reopen SAME binary and Resume for manual Telex. No rebuild or new permission entry between modes. If FAIL: stop composer trial and investigate tap suppression further.
