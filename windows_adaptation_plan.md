# VXKey — kế hoạch thích ứng Windows

Version: 1.0.0 · Ngày: 2026-11-02
Trạng thái: **PLAN ONLY — CHƯA ĐƯỢC TRIỂN KHAI**
Chỉ bắt đầu khi user ra lệnh rõ ràng. Việc viết plan không cho phép agent tự code, prototype, build, gọi agent, cài app, thay OS setting hoặc phát hành Windows.

[Track dài hạn](maintenance_track.md) · [Work log](../_summary/work_log.md) · [Log lỗi gõ](typing_issue_log.md) · [Danh sách mốc có cấu trúc](docs/tasks/windows_adaptation_plan.json)

## 1. Mục tiêu và nguyên tắc ưu tiên

Sớm có candidate dùng thử cho cả macOS và Windows, rồi cải thiện bằng sử dụng thực tế. Windows là một luồng phát triển trong cùng track dài hạn VXKey, không phải việc để chờ macOS hoàn thiện toàn bộ rồi mới bắt đầu. **Đây là thứ tự đề xuất sau khi user cho triển khai, không phải quyền tự khởi chạy.**

Ưu tiên: gõ đúng → không làm bàn phím/app bị mắc kẹt → phục hồi rõ ràng → tray/Settings đồng nhất → tính năng phụ → installer/updater. Không đặt ngày giao cố định khi chưa có môi trường Windows và chưa đo phần tái sử dụng. Track dài hạn không mặc định effort/model cao.

## 2. Hiện trạng đã đọc từ repo

| Phần | Có gì thực tế | Cách sử dụng trong plan |
| --- | --- | --- |
| Rust core/engine/C ABI | Funput core và engine đang dùng trên Mac | Dùng chung thuật toán, tránh viết engine Việt thứ hai |
| Windows upstream | [Cargo](platforms/windows/Cargo.toml), [hook](platforms/windows/src/background/hook/keyboard/mod.rs), [keydown](platforms/windows/src/background/hook/keyboard/keydown.rs), [injection](platforms/windows/src/background/inject.rs), [foreground](platforms/windows/src/background/hook/foreground/mod.rs) | Tách phần native cần thiết và review assumptions; không coi code upstream là đã đạt QA VXKey |
| UI Windows upstream | Slint và tray riêng, package funput-windows ngoài workspace root | Không bê nguyên shell Slint sang VXKey; chỉ tái sử dụng phần native phù hợp |
| UI VXKey | Tauri/React Settings/tray đã có ở [desktop](desktop/) | Dùng cùng component/config/copy, bổ sung adapter Windows |
| Bridge VXKey | Protocol/status chung, local transport hiện dựa Unix/macOS; non-mac có nhánh unsupported | Cần transport và native lifecycle Windows thật; chưa phải chỉ đổi target rồi build |
| Windows defaults | [config_example_windows.json](docs/spec/config_example_windows.json) đã có | Giữ các quyết định user, kiểm parser/runtime/UI readback khi triển khai |
| TSF | [RFC upstream](docs/features/windows-tsf-mode.md) ghi chưa triển khai | Không phải backend VXKey hiện có; không chọn làm đường găng ban đầu |
| QA hiện hành | Mac đã gõ được nhưng còn log lỗi w và từ đầu câu | Không suy Windows PASS từ Mac hoặc từ fixture Rust |

**Chưa có:** candidate Windows VXKey đã kiểm chứng, bằng chứng chạy hook Windows, checklist tương thích app/integrity level hoặc môi trường Windows QA được xác nhận cho task này.

## 3. Kiến trúc đề xuất — để triển khai sau khi được giao

### Phần dùng chung

- Rust Funput core/engine, macro/edit-plan semantics, policy loại trừ app, config schema/migration và corpus hồi quy.
- UI Tauri/React, bản dịch, Settings, mode V/E và Ư/W, lỗi/cảnh báo theo capability thực.
- State contract: app mode, suspended/fault, config revision/ACK, session/generation, control deadline. Không đưa từng phím hay nội dung đang gõ qua IPC/WebView.
- Corpus chung phải chạy trên core, native Mac và native Win; mỗi track có kết quả độc lập.
- Không copy Swift AX code hoặc ngưỡng8key/120ms sang Windows như thông số đã được chứng minh. Policy có thể chia sẻ sau khi invariants rõ; chưa cần refactor lớn toàn repo chỉ để khởi động Windows.

### Phần native Windows

- Đề xuất companion Tauri + helper Rust VXKeyEngine.exe do companion quản lý. Hook/engine chạy ngoài UI thread, helper có message loop riêng.
- Tận dụng WH_KEYBOARD_LL, SendInput, Unicode UTF-16, key mapping, modifier/hotkey và foreground tracking upstream sau review.
- Không có Input Source/profile mới trong phạm vi prototype hook; không TSF DLL, service, driver kernel hoặc tự nâng admin.
- Lưu cache target/mode/config để callback không đợi UIA/COM, disk, network, clipboard, mutex UI hoặc IPC response.
- Snapshot guard trên worker; định nghĩa coverage cho UI Automation/TextPattern hoặc adapter cụ thể sau capability spike. Không hứa mọi editor đều hỗ trợ ownership proof như nhau.
- Delivery phải kiểm tra đầy đủ số input được SendInput nhận, tag injection riêng của phiên và trạng thái modifier. Không coi API success là document ACK; không replay mù sau batch partial.
- Trước injection phải kiểm target/generation phù hợp; vẫn ghi rõ race do focus đổi giữa kiểm tra và SendInput. Thiếu proof/capability chuyển nguyên phím theo policy, không xóa vùng tùy ý.
- App đích quyền cao hơn, secure desktop/UAC/lock screen: bypass và báo capability phù hợp. Không mặc định chạy admin để né giới hạn.
- Companion Quit/crash chỉ quản lý đúng child của mình; thiết kế Job Object/lifetime handle và cleanup để không còn orphan hook. Không tự respawn sau fault trong prototype.

### Bridge Windows

Đề xuất named pipe local với ACL theo user/session, từ chối remote, deadline và frame giới hạn. ACL một mình không đủ xác thực peer: xác minh PID/process identity/parent lifetime và session secret qua kênh khởi tạo riêng, không đặt secret trên command line/log. Production signer/package identity và dev trust cần contract riêng Windows; không giả lập audit-token/Security.framework của Mac bằng kiểm path đơn lẻ.

Dùng schema status chung với capability Windows thật: target_supported, hook_state, delivery_state, suspended/fault. Các trường này mới là đề xuất cần chuẩn hóa ở W2; chưa phải field đã có trong protocol. Không hiện UI đòi Accessibility kiểu macOS trên Windows.

## 4. Defaults user đã chốt

| Hạng mục | macOS | Windows |
| --- | --- | --- |
| Toggle Việt/Anh | Ctrl+Option | Ctrl+Win |
| Custom shortcut | Có | Có |
| Modifier-only gesture | Kích hoạt khi nhả hết, phím thứ ba hủy gesture | Cùng hành vi; test Start menu, Win shortcut, keyUp thật |
| Clipboard history | Ctrl+Cmd+V | Win+V; native provider mặc định, user chọn native hoặc VXKey |
| Double-space → dấu chấm + space | OFF mặc định; khóa khi native Mac đã bật hoặc chưa đọc được trạng thái | ON mặc định |
| Launch at login | OFF trong giai đoạn kiểm độ ổn định | OFF trong giai đoạn kiểm độ ổn định |
| Icon | serious V/E; playful Ư/W | Cùng bốn SVG, raster phù hợp DPI/tray |
| Kiểu đặt dấu | Giữ setting hiện tại theo user | Schema/ý nghĩa cùng Mac; không tự đổi vì ví dụ minh họa |
| Quy tắc gõ | w lẻ→ư, W→Ư, thw→thư, vwng→vưng; casch/cachs→cách; kéo dài chữ đúng | Cùng corpus bắt buộc |

Win+V thuộc native clipboard flow của Windows. Provider native: VXKey không chiếm chord này. Provider VXKey chỉ nối khi prototype chứng minh không mở cả hai panel/không giữ modifier; nếu hệ thống không cho chiếm đáng tin thì báo rõ và cho custom shortcut. Không tự tắt Windows clipboard history bằng registry.

Không xây công cụ remap shortcut hệ thống/chụp màn hình trong phạm vi này.

## 5. Các mốc thực hiện — tất cả đang PLANNED

| Mốc | Đầu ra cụ thể | Phụ thuộc | Gate để qua mốc |
| --- | --- | --- | --- |
| S0 — corpus chung | Ca input/expected cho w, đặt dấu linh hoạt, kéo dài chữ, Backspace, macro, tiếng Anh, numpad, câu dài; lỗi user có ID | User cho bắt đầu phần tương ứng | Core test và expected có nguồn; không tự suy từ chuỗi paste |
| W0 — inventory và contract | Map module tái sử dụng, dependency UI cần bỏ, bundle IDs/data paths, state/error contract, target Windows dự kiến | Lệnh triển khai Windows | Không còn nhầm Funput/Slint/TSF với VXKey/Tauri |
| W1 — native hook spike | Harness riêng: pass-through, suppress, Unicode, modifier, injection-tag, Pause/Quit/crash; không cần full Settings | W0; môi trường Windows QA | Native receipt đủ input/output/OS/build; không chỉ cross-compile |
| W2 — helper + runtime bridge | Named pipe authenticated, cache config, target generation, policy, watchdog/child lifetime, isolated data root | W1 | Deny peer sai/timeout/stale config/crash; không có per-key IPC |
| W3 — candidate gõ tối thiểu | Engine thật + Tauri tray/Pause/Resume + Settings thiết yếu, Telex/VNI/macro/policy | S0, W2 | Notepad + editor web câu dài; không lẫn phím gốc, không keyUp mắc kẹt; non-admin scope rõ |
| W4 — parity dùng thử | Clipboard provider, double-space, custom shortcut, app exclusion nhanh, theme/locale/DPI | W3 | Save/restart/readback và native behavior, không chỉ stored config |
| W5 — stability + package | Corpus Mac/Win cùng expected; stress/fault/compatibility; portable/installer/updater contract | W3; phần W4 tương ứng đã QA | Mỗi platform có receipt riêng; signer/updater/permission gates trước release |
| W6 — sử dụng dài hạn | Log lỗi thực tế theo version/app/platform, hồi quy mỗi sửa | Candidate mỗi platform | Không tự đóng track; chỉ user quyết định đóng |

Thứ tự ưu tiên sau khi được giao: **W0 → W1 → W2 → W3** để có Windows candidate sớm; S0 và sửa lỗi Mac có thể chạy thành hai luồng công việc, không cần chờ W4/W5. “Song song” là cách tổ chức backlog, không tự cho phép spawn agent.

Đề xuất chứng minh Windows11 x64 trước; Windows10/x64, ARM64 và coverage x86 app được ghi thành gate riêng, không coi đã hỗ trợ. Khi W0 bắt đầu mới xác nhận baseline sản phẩm và thiết bị QA hiện có; hiện chưa chốt thay user.

## 6. Gói QA bắt buộc

| Nhóm | Cases | Bằng chứng |
| --- | --- | --- |
| Core parity | w/W/thw/vwng/thwr/nhwng/chwx, casch/cachs, tone placement, kéo dài chữ | Corpus input/expected, version core; không native claim |
| Chặn/phát phím | Suppress một phím, Unicode đơn/chuỗi/surrogate, zero/partial delivery, injected-tag | Native Windows result; không dùng giả định nil/.null từ Mac |
| Tốc độ | Slow/normal/fast, rollover, autorepeat, câu dài, CPU/UI bận | Input sequence/timing fixture + user physical typing; không bắt user gõ chậm |
| Focus/ownership | Mouse/select/Tab/Enter, programmatic caret/focus, đổi app/PID reuse, sentinel prefix/suffix | Text trước/sau và Undo; không log tài liệu người dùng ngoài fixture |
| Modifier/layout | Ctrl/Win release, phím thứ ba, AltGr, CapsLock/Shift, dead keys, layout đang dùng, numpad VNI | Không mất keyUp, không gây Start menu ngoài ý muốn, không tự đổi layout |
| Quyền và app | Standard vs elevated, UAC/secure desktop, password fields, editor unsupported | Bypass/fault đúng phạm vi, không admin mặc định |
| Lifetime | Pause/Resume, helper/UI crash, sleep/wake, restart, hook bị mất | Gõ nguyên bản vẫn hoạt động; không orphan helper/hook |
| Clipboard | Native vs VXKey Win+V, text-only, deny sensitive/unsupported formats, retention | Provider chỉ một nơi, clipboard ngoài hot path |
| UI/package | Settings là cả native window, About/tray, theme/VI/EN, DPI100/150/200%, update | Native screenshots/actions + package receipts riêng |

Starter app matrix: Notepad, Word, VSCode, Chromium/WebView2, Firefox, Windows Terminal/console và password field fixture. Trạng thái từng app ban đầu **NOT_TESTED**, không hứa coverage đồng đều. Biến thể unsupported phải được ghi rõ.

## 7. Acceptance tiêu biểu

```gherkin
Feature: Windows native background typing

  Scenario: Gõ linh hoạt ở tốc độ thường
    Given VXKey có hook healthy trên Windows và editor thuộc coverage đã kiểm
    And mode Việt và target proof còn hợp lệ
    When user gõ "casch cachs" rồi một câu dài ở tốc độ thường
    Then hai từ đầu đều là "cách"
    And phần còn lại không mất chuyển dấu do một checkpoint AX hoặc UIA đang chờ

  Scenario: W là nguyên âm mặc định
    Given mode Telex Việt
    When user gõ riêng "w", "W", "thw", "vwng"
    Then các kết quả tương ứng là "ư", "Ư", "thư", "vưng"

  Scenario: Delivery không đầy đủ
    Given batch thay thế đã được chuẩn bị cho một target
    When SendInput nhận ít event hơn dự kiến
    Then driver không replay mù batch vào caret có thể đã đổi
    And trạng thái delivery failure hiển thị đúng
    And các phím vật lý tiếp theo không bị giữ vô hạn

  Scenario: Clipboard provider native
    Given windows_history_provider là "native"
    When user nhấn Win+V
    Then VXKey không chiếm shortcut hoặc mở panel VXKey cùng lúc

  Scenario: Callback hoặc helper lỗi
    Given helper đang xử lý gõ
    When helper dừng hoặc lease hết hạn
    Then không còn rewrite mới từ phiên đó
    And cleanup không signal tiến trình không thuộc VXKey
    And không tự bật lại sau fault trong prototype
```

## 8. Handoff contract và chứng cứ

Mỗi task triển khai sau này phải trả: file sửa, tests thực chạy, exact candidate/version/hash, OS/architecture/app matrix, known gaps và đường link work log/log lỗi. Schema/report không chứa raw user keystrokes, document content hoặc clipboard data ngoài fixture chủ động.

Windows và Mac có status riêng: source/fixture/build/native_trial/dogfood/release. Mốc Windows không tự PASS vì Mac đã PASS. Specs chưa được user đánh dấu ready thì vẫn không tự đổi milestone đó.

## 9. Nguồn kỹ thuật đã đối chiếu

- Microsoft [LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc): callback có message loop, cần nhanh; timeout có thể khiến hook bị gỡ im lặng; không dựa GetAsyncKeyState để đo trạng thái mới của chính event trong callback.
- Microsoft [SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput): giới hạn UIPI, số events được chèn, batch ordering và modifier hiện hữu. Thành công chèn input không phải ACK document.
- Microsoft [Named pipe security](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights): default ACL không thay security descriptor và peer authentication cụ thể.
- Microsoft [Clipboard format listener](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-addclipboardformatlistener): WM_CLIPBOARDUPDATE/listener, không đưa polling clipboard vào callback phím.

Các lựa chọn helper/pipe/UIA/các mốc là thiết kế đề xuất của VXKey, không phải khẳng định Microsoft đã chứng nhận cách triển khai hoặc repo đã làm xong.

## 10. Điều kiện bắt đầu

**Chờ user nói triển khai.** Không tự bắt đầu S0/W0/W1 hay phần code Windows nào sau khi viết tài liệu này. Track VXKey vẫn MỞ DÀI HẠN — CHỈ USER ĐƯỢC ĐÓNG.
