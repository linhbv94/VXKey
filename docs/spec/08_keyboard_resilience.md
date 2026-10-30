# 08. Brainstorm và hợp đồng phục hồi nhập liệu — VXKey

> **Skills:** `_strat0_system`, `_strat1_problem`, `_strat2_solution`, `_strat3_evaluation`, `_strat4_validation`; kết nối `_spec2_feature_flow` và `_spec5_qa_acceptance`.  
> **Version:** 0.5.1 — **Ngày cập nhật:** 2026-10-30  
> **Trạng thái:** Contract đề xuất đã soạn theo yêu cầu user; chưa implementation/fault-injection/native QA.  
> **Scope:** Backend keyboard macOS/Windows, UI companion/recovery, startup. Không can thiệp bàn phím/OS hiện tại trong lượt này.

## 1. Problem framing

**Vấn đề thực sự:** User mất khả năng gõ hoặc dùng shortcut sau lỗi bộ gõ, nhưng chính keyboard shortcut có thể không còn dùng được để tắt bộ gõ. Mục tiêu là giảm phạm vi lỗi, không chặn input khi không chắc, và luôn có nhiều đường thoát độc lập đã test.

**Hiện trạng:** VXKey chưa có native binary. Mac upstream dùng IMK per-client controllers; Windows có low-level hook và SendInput. Tauri UI chưa được tách khỏi upstream. Không có bằng chứng máy user từng liệt do đúng Funput hoặc shortcut Ctrl+Option/Ctrl+Win; không suy lời cảnh báo thành finding đã tái hiện.

**Mong muốn:** Khi xử lý nhập liệu lỗi, ngừng biến đổi phím theo phạm vi lỗi, giữ đường keyboard nguyên bản khi backend còn forward được; mouse/native recovery và nguồn nhập thay thế không phụ thuộc composition. Không hứa recovery tuyệt đối nếu OS/input driver hoặc hardware đang lỗi.

**Stakeholders:** User cần không gián đoạn công việc; IME/backend chịu trách nhiệm input; companion/recovery chịu trách nhiệm status và đường thoát; QA chứng minh bằng fault injection đúng artifact. Không thu nội dung gõ để chẩn đoán.

**Giả định cần kiểm chứng:** Client cho đủ thông tin để fallback an toàn; native OS source selector vẫn dùng được khi IME treo; Windows hook/worker được dừng đúng ownership; mouse/recovery surface độc lập thread/process lỗi. Đây là runtime hypotheses, không kết luận từ build.

## 2. Evidence và giới hạn platform

- [Mac controller](../../../xfunput/platforms/macos/Funput/IME/FunputInputController.swift) hiện chỉ recognizedEvents keyDown và handle keyDown, nên modifier-only cần thêm handling. [KeyCombo](../../../xfunput/platforms/macos/Funput/Model/KeyCombo.swift) hiện lưu main key + modifiers; không coi recorder đó đã hỗ trợ combo mới.
- [Windows hotkey rules](../../../xfunput/platforms/windows/src/background/hotkey/rules.rs) đã phân biệt modifier-only và release watcher; có thể reuse sau review actual event path. Cần track cả left/right physically thay vì gộp sai release.
- [Windows injection](../../../xfunput/platforms/windows/src/background/inject/events.rs) raw_send hiện bỏ return value SendInput. Đây là rủi ro source đã đọc, không native repro ở Windows và chưa có bản vá trong lượt spec này.
- [Microsoft LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc): hook phải trả nhanh và forward events không xử lý; timeout có thể khiến OS gỡ hook im lặng. Gỡ hook thường làm mất chức năng bộ gõ, không tự chứng minh toàn bàn phím bị khóa. Không dùng giới hạn timeout của OS làm cơ chế safety duy nhất.
- [Apple IMK recognizedEvents](https://developer.apple.com/documentation/inputmethodkit/imkstatesetting) cho nhận event masks; native delivery flagsChanged phải test trên artifact/OS/app matrix. [VoiceOver](https://www.apple.com/uk/voiceover/info/guide/_1124.html) có dùng Control+Option. User đã dùng hai defaults ổn định; giữ defaults, không tắt VoiceOver hoặc đổi key âm thầm để đạt test.

## 3. Cause-and-effect: “liệt” có thể là gì?

| Giả thuyết | Dấu hiệu cần phân biệt | Cơ chế có thể xảy ra | Response cần thiết |
|---|---|---|---|
| Client/session IME treo | Một app/field không gõ; OS khác vẫn nhận | Sync document read hoặc callback client không trả | Isolate session, fallback/preedit/alternate source, không giữ lock chung |
| IME input event loop deadlock | Nhiều app dùng VXKey không gõ, UI companion có thể vẫn mở | Mutex reentry, FFI panic boundary, synchronous IPC/DB | Không IPC/DB trên key callback; main-loop health, recovery source path |
| Hook swallow sai | Phím bị mất trong nhiều app Windows | Hook return handled nhưng replacement chưa vào app | Validate plan/result; no swallowing theo assumption |
| SendInput zero/partial/UIPI | Một phần chữ mất/duplicate hoặc target elevated không nhận | Injection không complete nhưng original bị consume | Detect counts/error, stop composing; không replay cả batch mù |
| Modifier state bị kẹt | Nhiều phím thành shortcut như đang giữ Ctrl/Win | Down/up imbalance, self-injection recursion, focus đổi | Reset state nội bộ theo OS physical snapshot, không thả phím thật thay user |
| Shortcut fire sớm | Ctrl+Win+phím khác vô tình đổi VI/EN | Toggle ngay khi modifier thứ2 xuống | Release-only gesture, cancel khi có key thứ3 |
| Hook/event tap timeout | Tiếng Việt ngừng, keyboard vẫn plain | Callback chậm bị OS bỏ qua/gỡ | Báo degraded, không reattach loop vô hạn |
| Nhiều tool can thiệp | Chỉ lỗi khi chạy remapper/IME khác/RDP | Duplicate interception hoặc injected events đi vòng | Own-event tagging, coexistence matrix, không “kill other IME” |
| Lỗi ngoài VXKey | Chuột/OS/native keyboard hoặc thiết bị cũng bất thường | Hardware/OS/remote connection | Không đổ mặc định cho engine; OS/hardware troubleshooting ngoài claim |

Chỉ một trường hợp “không gõ được” chưa xác định được lớp lỗi. Diagnostic chỉ aggregate latency/error counts/source status/component version, không log keys/text/window titles/password.

## 4. Mở rộng solution space

- **A — Vận hành/UX:** Mặc định autostart off, giữ alternate OS source có thể chọn bằng chuột, hướng dẫn recovery rõ từ onboarding/launcher.
- **B — Xử lý bên trong backend:** Callback bounded/nonblocking, verified replacement trước consume, circuit breaker theo session, reset modifier state, no blind retry.
- **C — Kiến trúc/failure domains:** Native input process tách Tauri/WebView/clipboard; companion supervision và launcher recovery có native control path.
- **D — Safety profile:** Fault ghi safe-start marker và ngừng processing cho tới user Resume; không auto restart can thiệp vào input đang hỏng.

Các hướng có thể kết hợp, không yêu cầu chọn đúng một hướng.

## 5. Evaluation matrix

| Hướng | User impact / lợi ích | Cost và maintenance | Rủi ro còn lại / trade-off |
|---|---|---|---|
| A | Đường thoát dễ hiểu, chi phí ít | Thấp; onboarding và OS-specific steps | Không chữa được callback treo, user vẫn cần mouse/source chooser |
| B | Giảm lỗi trước khi ảnh hưởng user | Vừa; tập trung hot path/native tests | Không thể bọc timeout là hủy được client call/injection partial |
| C | UI/clipboard lỗi không kéo chết bộ gõ và ngược lại | Cao hơn: versioned IPC/process ownership/packaging | Supervisor cũng có thể chết; phải test OS recovery khi cả2lỗi |
| D | Lỗi lặp không tự khởi động lại ngay vào vòng lỗi | Vừa; marker/recovery state migration | User phải explicit Resume, có thể mất tiện lợi tạm thời |

Business value là độ tin cậy/khả năng dogfood; không thêm telemetry để đo user text. Native fault fixtures cung cấp measurement và pass/fail. Khuyến nghị kết hợp A+B+C+D, giữ complexity của supervisor tối thiểu đủ kill/recover owned worker, không làm một hệ thống orchestration agent.

## 6. Contract bắt buộc KEY10

### 6.1. Phòng lỗi tại input path

1. Key handler không chờ Tauri/IPC, clipboard/provider, disk/config fsync, network, updater hoặc UI rendering. Dùng immutable validated cache và single-thread/concurrency ownership rõ.
2. Windows hook dùng dedicated native worker/event loop, không WebView/clipboard process/thread. Unhandled/unsupported/bypass input forward ngay; nCode<0 luôn theo OS hook contract. Không giữ modifier xuống để “đợi xem user bấm gì”.
3. Chỉ consume event khi mutation được xử lý theo contract có result rõ. SendInput count=0: không coi success; partial: outcome_unknown, dừng transform/quarantine session, không gửi lại cả batch hoặc phím original theo phỏng đoán. Native implementation phải quyết định compensation theo actual accepted events và test, không hứa atomic SendInput.
4. Self-injection có marker/process/session bounds để không chạy lại qua composer; untrusted producer không được giả marker thành quyền bypass input policy. Không blanket suppress injected events của accessibility/remapper khác khi chưa hiểu source.
5. Mac unsafe range/client/focus → không destructive write; fallback/preedit hoặc pass-through session mới. Callback synchronous client đã treo không tự biến thành cancelable bằng timer; recovery external process/source path vẫn bắt buộc.
6. Fault liên quan correctness data-loss/partial injection chuyển suspended ngay. Latency suspicion đo main-event-loop heartbeat; missed3heartbeat1s thì degraded/unresponsive, chưa coi đã recover. Không log event content.

### 6.2. Circuit breaker và safe start

- Circuit breaker ban đầu theo session/client; mất tiến triển input-loop hoặc unknown injection ownership có thể suspend cả backend.
- Suspension ngừng engine transforms, macros, retone, double-space và injection; đi raw pass-through khi callback còn hoạt động. VI/EN toggle không auto resume.
- Ghi recovery_state.json sau fault; launcher/autostart/restart không attach lại hook hoặc bật processing âm thầm. Explicit Resume làm native health check và tạo session generation mới; fail thì giữ safe mode.
- Không auto kill/restart/reselect OS source vô hạn. Giới hạn tối đa một stop/cleanup của owned worker mỗi incident, không tự relaunch processing cho tới user Resume.

### 6.3. Mouse/native recovery độc lập

- S01 có Tạm ngừng xử lý phím/Khôi phục nhập liệu; launcher mở recovery dù tray ẩn. Native minimal recovery surface không phụ thuộc WebView còn responsive.
- Windows: companion/control path ngoài hook thread gửi suspend request; timeout2s thì báo unresponsive, verify process handle/PID generation/bundle ownership rồi stop đúng input worker do VXKey tạo khi user yêu cầu recovery. OS xác nhận exit/unhook trước báo recovered; không dựa vào lệnh spawn/kill đã gửi. Không đòi admin mặc định, không kill IME/remapper khác.
- Khi companion cũng chết: user vẫn có OS Task Manager/alternate input path để stop đúng VXKey component; kiểm bằng mouse trên test machine. Không claim UI riêng vẫn sống khi process chứa nó đã chết.
- Mac: recovery nêu/chọn alternate source do user xác nhận bằng mouse, ưu tiên ABC hoặc source fallback đã xác minh tại onboarding. Chỉ đổi source sau action user; không tự reset danh sách nguồn nhập. Nếu IME không ack, không báo “đã ngừng” từ bool UI. Sau source chuyển thật có thể stop đúng owned IME để giải phóng, theo lifecycle/ownership gates; không broad killall.
- OS input source selector còn hoạt động là giả định phải test. Nếu OS/input stack cũng treo, có hướng OS recovery/logout/restart như last resort; không cam kết mọi incident tránh được restart.

### 6.4. Startup policy

Companion autostart mặc định OFF cho cả platform khi chưa đạt an toàn. Gate PASS không tự bật; user phải chốt default change riêng. Import/update không tự bật registration hoặc xóa safe-start marker. Source VXKey có thể được OS nhớ/nạp lại độc lập checkbox autostart; onboarding giải thích và test reboot/sign-in case. Không đặt task test autostart trên app production của user.

## 7. Validation plan và fault matrix

Hypothesis: nonblocking path + independent recovery khôi phục keyboard plain input trong phạm vi VXKey khi một component lỗi. Method: native fixture/fault injection trên account hoặc VM cô lập, có mouse/alternate source được kiểm trước, không trên tài liệu user thật.

| Fault / test | Kết quả bắt buộc | Evidence |
|---|---|---|
| Kill/hang WebView/history worker | Input engine còn gõ; recovery/native launcher mở được | Native readback, component/PID/versions |
| Disconnect companion IPC | Engine dùng cache, no per-key wait, UI không báo live giả | Timing/heartbeat và typing fixture |
| Hang native input callback | Supervisor phát hiện, recovery bằng mouse/source khác có outcome thật | Main-loop heartbeat, stopwatch từ click tới readback |
| Windows SendInput=0/partial/UIPI | Không false-success/blind replay; suspension và plain input recovery | Injected fixture return counts và actual text |
| Lost key-up/sleep/wake/focus switch | Không kẹt Ctrl/Alt/Win nội bộ; chord state reset | Native shortcut + ordinary-key fixture |
| Self-injection recursion | Không event storm, no unbounded queue | Aggregate event counts/CPU, không text log |
| Ctrl+Option hoặc Ctrl+Win bấm/thả | Đổi mode đúng một lần, đúng default cả left/right | Q35 |
| Combo cộng phím thứ3 | OS/app nhận shortcut, không toggle VI/EN thừa | Q35, Desktop/arrow/VoiceOver fixture |
| Fault rồi restart/login | Safe-start còn suspended, autostart default off | Marker/OS registration/source readback |
| UI+engine cùng lỗi | OS alternate-source/Task Manager mouse path dùng được hoặc ghi failed | Native run, không suy từ unit test |
| Tray hidden và serious/playful đổi | Launcher phục hồi entry, recovery state không bị glyph che | Q36/Q37 |

Targets: request suspend healthy backend ack≤2s; dead/unresponsive worker path phải có final outcome/error≤5s sau mouse recovery action trong controlled Windows fixture. Đây là acceptance mục tiêu chưa đo, không lời hứa OS phổ quát. Mac source-switch UI/readback và hồi phục gõ phải đo riêng, nếu không đáp ứng thì blocked daily-driver; không đánh PASS chỉ từ supervisor detect3s. Full OS/hardware failure ngoài guarantee nhưng phải có hướng last resort.

Success criteria: fault không làm silent processing restart, data ngoài vùng fixture nguyên vẹn, plain typing khôi phục qua ít nhất một đường độc lập đã chứng minh. Nếu false-selection/partial injection chưa giải quyết hoặc không có recovery path, giữ autostart off và chưa dogfood chính/release.

## 8. Decision record và việc tiếp

Problem: mất input có thể làm mất luôn shortcut tắt bộ gõ. Decision: A+B+C+D với input process isolation, safe-start và mouse/native/OS escape; giữ2modifier-only defaults theo user. Trade-off: thêm lifecycle/health tests/explicit Resume, đổi lấy hạn chế phạm vi và khả năng phục hồi. Unknowns: native flagsChanged delivery, Start/VoiceOver/coexistence, IME source escape khi hang, SendInput partial compensation và timings. Next: implement từng backend theo02/04/07, fault fixtures và Q35..Q39, giữ status not_run đến có binary/artifact.

## 9. Scoped clipboard routing không được phá input safety

User chọn native Windows/VXKey cho Win+V. Adapter này chỉ active khi explicit provider VXKey, routing health ready; không gắn vào mọi keys hoặc bộ remap tổng quát. Recovery/suspend/safe-start phải release Win+V interception; test crash owner/hook timeout để không mất cả native và VXKey clipboard entry. Chọn provider không thay input source hoặc hard bypass; callback không đợi history WebView/data decryption. Native routing implementation còn spike, chưa có evidence runtime.
