# 05. QA acceptance và validation rules — VXKey

> **Skill:** `_spec5_qa_acceptance`  
> **Version:** 0.5.1 — **Ngày cập nhật:** 2026-10-30  
> **Trạng thái:** Đã soạn để review; chưa được user xác nhận specs ready.  
> **Đối tượng:** Dev, UI/UX, QA và AI Execution Agents.  
> **Nguồn scope:** [Tổng quan](00_system_overview.md); quyết định UI Tauri chung Mac/Windows của user.

## 1. Quy tắc bằng chứng

Đây là test specification, chưa có test run PASS. Mỗi result: case_id, requirement_id, commit, toolchain/SDK, artifact SHA256, OS/arch/app versions, fixture, steps, expected/actual, status(pass/fail/blocked/not_run), evidence path và limitation. Fixture logs không có text user thật/keys; screenshots chỉ dữ liệu demo. Source/fixture/browser/native/package/update là các mức riêng, không suy platform khác.

## 2. Traceability và edge matrix

| Requirement | Case ID | Positive / negative / boundary |
|---|---|---|
| KEY01 | Q01 | Prefix/suffix, false/stale/noncollapsed selection; UTF-16 emoji/composing marks; reject direct khi chưa verify |
| KEY02 | Q02 | Focus/source đổi giữa dấu/backspace/Undo, app đóng, config queued; không write client mới |
| KEY03 | Q03 | Bypass cả VI/EN/macro/retone; bật giữa composition, toggle không gỡ rule |
| KEY04 | Q04 | Memory/bypass/clipboard block độc lập; exact ID, unknown app, duplicate display names |
| KEY05 | Q05 | EN macro off giữ literal; bật explicit đúng expansion, enabled=false thắng in_en=true |
| KEY06 | Q06 | Mở Settings/history rồi đổi app, pending stale target; helper không có memory entry |
| KEY07 | Q07 | VI/EN/bypass/ABC/disconnected/incompatible; mất3 heartbeat không hiện VI cũ |
| KEY08 | Q08 | Kill companion/worker fixture, provider block, reconnect; typing tiếp tục và không disk IPC sync trên phím |
| KEY09 | Q09 | Install conflict/symlink/disk full/interrupted swap/uninstall; không thay Funput/ABC/data khác |
| CFG01 | Q10 | Readback restart, failed fsync, revision conflict/import malformed/oversize/new schema/Cancel/reset scope |
| IPC01 | Q11 | Wrong UID/signer/pipe peer, oversized/malformed/replay/out-of-order/version mismatch, untrusted WebView command |
| UI01 | Q12 | UI vi/en/theme/reload/shortcuts/accessibility/titlebar/menu/Settings/focus/scale |
| CLP01 | Q13 | Off/pause/lock/grant/revoke baseline; marker/source/file/image/remote/mixed filters trước payload |
| CLP02 | Q14 | Exact multiline/emoji/spaces, rapid count changes/self token, external copy race/write error; không auto paste |
| CLP03 | Q15 | 0/1/200/201 entries, 128KiB và +1byte,10MiB, TTL/clock rollback, key/DB corrupt/delete/clear |
| UPD01 | Q16 | Tamper/wrong arch, simultaneous CTA, busy/quiesce timeout/ack/helper timeout, old→new pair/receipt/rollback |

## 3. Gherkin contract tối thiểu

```gherkin
Feature: An toàn composition
  Scenario: Q01 Selection giả không xóa suffix
    Given fixture có "goKEEP_THIS_SUFFIX" và caret thực sau "go"
    And client khai selection giả bao phủ suffix
    When engine đổi dấu trong session direct
    Then suffix còn nguyên và không có write ngoài vùng được xác minh

  Scenario: Q02 Focus mới không nhận buffer cũ
    Given app A đang composing và app B chưa có composition
    When focus chuyển A sang B trước callback cũ
    Then callback cũ không insert hoặc replace vào B

  Scenario: Q03 Bypass thắng macro và toggle
    Given app A bị hard bypass và macro "vn" được bật cho EN
    When user chọn VI rồi gõ "vn " trong A
    Then A nhận input nguyên bản và rule bypass còn nguyên

  Scenario: Q04 Các policy độc lập
    Given A có memory EN và không chặn clipboard
    When user bật input bypass cho A
    Then memory EN và clipboard policy không đổi

  Scenario: Q05 English giữ literal khi macro đã tắt
    Given mode EN và user đã tắt macro
    When user gõ "vn "
    Then text vẫn là "vn "

  Scenario: Q06 Target thay đổi lúc menu mở
    Given menu mở từ app A với focus_generation 10
    When focus sang B rồi UI gửi toggle generation 10
    Then trả stale_target và không sửa memory của A hoặc B

  Scenario: Q07 Snapshot cũ
    Given tray đang hiện VI và IME connection mất
    When không có heartbeat trong 3 giây
    Then tray báo unavailable thay vì VI live

  Scenario: Q08 Companion dừng
    Given IME đang gõ fixture và config cache hợp lệ
    When companion dừng đột ngột
    Then IME vẫn xử lý input từ cache và không chờ reconnect trên phím

Feature: Data và IPC
  Scenario: Q09 Cài pair bị gián đoạn
    Given bản VXKey cũ và Funput cùng tồn tại
    When install lỗi sau swap component đầu
    Then journal phục hồi pair cũ và Funput không thay đổi

  Scenario: Q10 Config conflict
    Given config revision 8
    When UI patch với expected_revision 7
    Then trả revision_conflict và file config giữ nguyên

  Scenario: Q11 Peer không hợp lệ
    Given socket chỉ dành cùng user và identity đã pin
    When peer không khớp yêu cầu xác thực gửi config_snapshot
    Then từ chối trước apply và không log payload

  Scenario: Q12 Locale khác chế độ gõ
    Given UI English và language_mode VI
    When đổi UI sang Tiếng Việt rồi restart companion
    Then UI giữ Tiếng Việt và language_mode vẫn VI

Feature: Clipboard
  Scenario: Q13 Resume không backfill
    Given capture paused và user copy nội dung fixture X
    When user resume
    Then X không được materialize hoặc thêm history

  Scenario: Q14 Copy-back bị clipboard mới chen vào
    Given user chọn entry A
    When clipboard version đổi trước final pre-write check
    Then trả clipboard_changed và không ghi A

  Scenario: Q15 Không fallback plaintext
    Given encrypted history bật và key không truy cập được
    When worker nhận copy event mới
    Then capture pause với key_unavailable và không ghi plaintext

Feature: Update
  Scenario: Q16 Composition chưa quiesce
    Given update payload đã verify và IME chưa ack quiesce
    When deadline 5 giây hết
    Then không replace bundle và giữ bản cũ hoạt động
```

Mỗi scenario mở rộng thành tests cho toàn bộ variants trong matrix; một scenario tối thiểu không đủ kết luận toàn requirement đạt.

## 4. Native matrix bắt buộc

Mac đầu: đúng supported OS/arm64, TextEdit/Notes/Safari/Chrome/VS Code/Cursor/terminal với version ghi thật. Mỗi app: Telex/TelexAdvanced/VNI, modern/traditional, VI/EN, macro on/off, bypass, chọn/caret/emoji/backspace/Undo, focus/source-switch/autocomplete. Secure fields/lock/sleep/wake/reopen/multi-window kiểm riêng. Direct opt-in chỉ cho client/field đã đạt; app không đạt dùng preedit và limitation.

Windows sau: native backend hook/TSF quyết định riêng; kiểm SendInput partial/zero/UIPI, AltGr/layout/dead keys, elevated target, browser/editor/terminal/game nếu claim; không mặc định run-as-admin. UI common không chứng minh IME support. Intel/OS cũ không có artifact/runtime thì not_run, không ghi PASS từ universal build.

## 5. Suite standards mapping và performance

ST-01..10 toàn bộ Settings; BI-01..12 vi/en; AM-01..14 app menu/About/Settings trên companion; TB-01..12 cho cửa sổ có titlebar, N/A với tray/history không titlebar; AU mappings theo install spec. QA icon native light/dark/Retina/1x, alpha/corners/focus/click ngoài/Escape, clamp cạnh màn hình. Hash vendor/catalog/browser screenshot không thay packaged native.

M1 đo baseline hot-path p50/p95/p99 và idle CPU/RSS, tách IME/companion/worker. Fixture regression mục tiêu p95 không tăng quá10% ở cùng setup, kết quả noisy phải rerun có kiểm soát; ngưỡng latency tuyệt đối và idle budgets chốt sau baseline, không invent số PASS. Provider blocking không làm IME main bị stall; check static call path cộng runtime stress. Chưa baseline thì performance gate blocked.

## 6. Gates

M0: links/IDs/schema/examples/cross-spec kiểm xong, user review riêng trước ready. M1: pinned source/toolchain/SDK27/artifact/tests. M2: Q01..08 và native typing không có mất chữ. M3: Q09..12 và suite UI gates. M4a: Q13..15 session variants + privacy/provider isolation. M4b: thêm key/encrypted store recovery/deletion/restart. M5: packaged first-install/uninstall/rollback/network privacy + native matrix. M6: Q16/AU/permission N→N+1/public artifact/authorization riêng. Failure data-loss/privacy/identity hoặc chưa kiểm các gate bắt buộc chặn milestone, không đổi wording để cho qua.


## 7. Feature-to-contract coverage

| Feature | Flow | Screens | Commands / native types | Test cases |
|---|---|---|---|---|
| F01 | FL01 | S03 | get_snapshot, status | Q07/Q09/Q12/Q17 |
| F02 | FL02/03 | S01/S02 | set_language, config_snapshot | Q01/Q02/Q05/Q07/Q18 |
| F03 | FL04/FL04a | S02/S04 | set_app_policy, forget_app_language, set_current_app_bypass | Q03/Q04/Q06/Q19/Q25/Q26/Q27 |
| F04 | FL05 | S02 | patch_config/import/export/reset/hotkey | Q10/Q12/Q20/Q21 |
| F05 | FL06/07 | S02/S05 | capture/history commands | Q13/Q14/Q15/Q22 |
| F06 | FL08 | S02/S05 | set_history_mode, secure store | Q15/Q23 |
| F07 | FL09 | S03/S07 | update_action/quiesce/helper receipt | Q09/Q16/Q24 |
| F08 | FL11 | S08 | Conversion session native API wrapper (04 mục13) | Q28/Q34 |
| F09 | FL10 | S02 | patch_config + native period runtime | Q29..Q33 |
| F10 | FL13 | S01/native recovery | set_input_suspended + supervisor/OS readback | Q37/Q38/Q39 |
| F11 | FL14 | S01/S02 | patch_config.ui.tray_icon_style | Q36 |

## 8. Gherkin cho mọi user story còn thiếu

```gherkin
Feature: F01 Onboarding
  Scenario: Q17 Bỏ qua chọn nguồn nhập
    Given companion hợp lệ và OS đang dùng ABC
    When user chọn Để sau trong bước Add Input Source
    Then onboarding không báo engine ready giả
    And Settings vẫn mở được và history chỉ chạy khi có consent riêng

Feature: F02 Typing và đổi phương thức
  Scenario: Q18 Đổi input method trong composition
    Given IME đang composing trong client fixture hợp lệ
    When user đổi Telex sang VNI qua Settings
    Then file config lưu revision mới và UI hiện pending_engine
    And engine chỉ apply ở safe boundary
    And không reprocess text cũ bằng method mới

Feature: F03 App picker
  Scenario: Q19 Cancel không tạo rule
    Given user mở app picker từ Applications
    When user chọn app rồi bấm Cancel
    Then không có policy mới hoặc revision mới
    And focus trả về nút mở picker trong Settings

Feature: F04 Configuration
  Scenario: Q20 Import consent không được bật từ file
    Given file import hợp schema có clipboard enabled và launch_companion_at_login true
    When user preview rồi commit replace_preferences
    Then các quyền và capture không được tự bật
    And preview và result ghi rõ các field bị giữ theo local policy

  Scenario: Q21 Hotkey registration conflict
    Given tổ hợp user chọn đang bị OS hoặc app khác giữ
    When register_hotkey cho action history không được OS chấp nhận
    Then không persist tổ hợp đó như active
    And tổ hợp cũ vẫn hoạt động nếu đã được đăng ký hợp lệ

Feature: F05 Clipboard tìm kiếm và sao chép
  Scenario: Q22 Không có kết quả
    Given history có entry hợp lệ và capture paused
    When user tìm query không khớp
    Then panel hiện no_results và action xóa query
    And không xóa history hoặc tự resume capture

Feature: F06 Persistence
  Scenario: Q23 Restart encrypted history
    Given user đã opt-in encrypted mode và có entry fixture đủ điều kiện
    When companion restart rồi mở history khi unlocked và key hợp lệ
    Then entry được giải mã trong runtime và copy giữ text exact
    And không có text hoặc preview plaintext trong DB hoặc index

Feature: F07 Update
  Scenario: Q24 Helper chưa xác nhận relaunch
    Given helper đã swap pair sau payload verification
    When không có version readback trong deadline
    Then receipt không báo completed
    And UI hoặc recovery ghi restart_required hoặc lỗi phù hợp
    And không tự chạy installer thứ hai
```

## 9. Validation matrix frontend và authority

| Field / action | Frontend validation | Host/IME authoritative validation | Error / expected result |
|---|---|---|---|
| Revision counter | Decimal string; không parse JS number | uint64 range + current expected match | invalid_argument/revision_conflict |
| Target token | Present cho mode action | Same IME session/PID/ID/generation/source | stale_target |
| Macro trigger |1..32scalars, không whitespace | UTF-8/scalar/duplicate/engine boundary contract | invalid_argument; config nguyên |
| Macro replacement |1..8192bytes display count | Byte limit, valid text, no side effects | invalid_argument |
| App policy ID | Chọn từ picker/explicit confirm | Exact metadata ID, không internal surfaces | invalid_argument |
| Import | Filter file picker, preview |1MiB, schema, duplicate JSON keys, semantic uniqueness, token expiry | invalid_argument/unsupported |
| History query |≤256scalars, debounce150ms | Same limit + revision/lock | invalid_argument/locked |
| History entry | UUID shape | Exists/unexpired/unlocked/current revision | not_found/entry_expired/locked |
| History text limit | Không trust preview length |131072 UTF-8 bytes exact | Skip capture, không truncate |
| Consent | Show text/version/action | Backend action-bound receipt, state/access check | permission_denied |
| Hotkey | Recorder valid accelerator | OS registration + rollback if persistence fails | busy/unsupported/persist_failed |
| Update | Job state disables repeated clicks | Signer/target/ownership/quiesce/receipt | signature_invalid/busy/timeout |

## 10. Negative/boundary cases bổ sung

- Q17–Q24 đều phải có actual result/evidence như Q01–Q16, không được ghi PASS từ spec.
- Counter:0,1,9007199254740993,18446744073709551615 và overflow; không precision loss qua JS.
- Macro:0/1/32/33scalars, combining marks, replacement8192/8193bytes, duplicate UUID/trigger, enabled/in_en combinations.
- Import:0byte/1MiB/1MiB+1, invalid UTF-8, duplicate JSON keys, new schema, single-use token expired/cancel/file changed sau preview.
- UI permission denied→grant→revoke; lock từ monitoring và từ paused; unlock paused không tự capture.
- Socket same user nhưng wrong signer, forged session/source target; body fields/raw text forbidden trên native protocol.
- Copy write success rồi clipboard đổi trước readback: outcome_unknown không auto replay/restore.

## 11. QA handoff checklist

- [ ] Tạo fixture/config/data root cô lập, chuẩn bị app matrix và rollback ABC.
- [ ] Validate schema examples và rejection fixtures, đủ commands/window capabilities.
- [ ] Feature F01..F07 đều có scenarios và edge variants theo milestone thực hiện.
- [ ] UI tree/props/states/keyboard/picker Cancel/Open/panel Escape, vi/en/theme/scale đúng artifact.
- [ ] Mỗi state trong02 được đi vào/thoát ít nhất một test; transition ngoài bảng reject hoặc no-op có log metadata.
- [ ] Native IME/data-safety/privacy/installer/permission/update tests đúng mức bằng chứng, không suy từ browser.
- [ ] Ghi not_run/blocked riêng, không dùng aggregate PASS che thiếu support matrix.

## 12. Acceptance — Current-app exclusion

```gherkin
Feature: F03 Loại trừ app đang dùng từ menu
  Scenario: Q25 Một thao tác loại trừ app hiện tại
    Given user đang ở ChatGPT fixture với exact app ID đã xác minh và chưa bypass
    And clipboard policy và memory của app đã có giá trị fixture
    When user mở tray rồi chọn "Tắt VXKey cho ChatGPT"
    Then input_bypass của đúng app được lưu và áp tại safe boundary
    And clipboard policy và language memory không đổi
    And Settings Applications và sau restart hiển thị cùng rule
    And menu mở lại trong app đó hiện "Bật VXKey cho ChatGPT"

  Scenario: Q26 Bật lại không ép tiếng Việt
    Given app A đang bypass và có memory EN
    When user chọn "Bật VXKey cho A" trong tray
    Then hard bypass được gỡ và memory vẫn EN
    And app B cùng clipboard settings không đổi

  Scenario: Q27 Focus đổi trước click không loại trừ nhầm app
    Given menu được mở từ app A và có token của A
    When external focus chuyển sang app B trước command click tới host
    Then command trả stale_target
    And không đổi input_bypass của A hoặc B hoặc companion
```

Edge variants: app không có ID, internal surface, target PID thoát/relaunch, hai app cùng display name, mở menu khi ABC active, IME disconnected, bật bypass giữa composition, double-click/duplicate request, disk write failure/revision conflict, source đổi nhưng external app không đổi. QA phải chứng minh target lấy trước UI activation; không chỉ render label đúng. Q25..Q27 map KEY03/KEY04/KEY06 + CFG01; không native QA trong lượt sửa spec.

## 13. Parity và double-space acceptance

| Requirement | Cases | Coverage |
|---|---|---|
| TYP01 | Q28 | Inventory upstream→VXKey,3methods, eager/spell/extra-onsets/smart-case/flip/tray visibility/converter; defaults và combinations |
| TYP02 | Q29..Q33 | Platform defaults, native interlock/live refresh, safe rewrite/Undo, bypass/EN/context, persisted choice |

```gherkin
Feature: F02 Upstream typing parity
  Scenario: Q28 Giữ Telex nâng cao và tùy chọn có sẵn
    Given user chọn Telex nâng cao trong cấu hình fixture
    When gõ "vwng" rồi boundary
    Then text là "vưng" theo upstream engine behavior
    And UI/schema/import có eager restore, spell check, extra onsets, smart-case và flip shortcut

Feature: F09 Dấu chấm bằng hai dấu cách
  Scenario: Q29 Default theo platform chỉ khi khởi tạo
    Given chưa có giá trị double_space_period trong fresh config
    When loader tạo profile macOS hoặc Windows
    Then macOS lưu false và Windows lưu true
    And restart không tính lại đè giá trị user đã lưu

  Scenario: Q30 Native macOS bật khóa VXKey
    Given macOS báo native period substitution enabled
    When mở Settings bộ gõ
    Then toggle VXKey disabled với helper native đang xử lý
    And effective VXKey transform false dù preference trước đó true

  Scenario: Q31 Native đổi khi đang chạy
    Given user bật double-space trong VXKey khi native macOS đang tắt
    When user bật native trong System Settings
    Then UI và IME runtime khóa VXKey và clear pending Space token
    And không sửa preference user hoặc tạo thêm dấu chấm

  Scenario: Q32 Hai Space đúng ngữ cảnh
    Given VXKey active và không bypass và native period không sở hữu thao tác
    And tính năng đã bật và text fixture là "xin chàoKEEP" với caret trước KEEP
    When user nhấn Space hai lần cách nhau dưới 500ms
    Then text là "xin chào. KEEP" và suffix KEEP nguyên vẹn
    And Undo transaction phục hồi "xin chào  KEEP"

  Scenario: Q33 Không xử lý trong ngữ cảnh không an toàn
    Given app bypass hoặc secure hoặc caret đổi hoặc native state unknown
    When có hai Space
    Then VXKey không rewrite hoặc inject dấu chấm
```

Edges: distinct keys/held-repeat;500ms và501ms; paste hai Space;3Spaces; punctuation/newline/tab/indentation/selection/emoji+combining marks; source/focus đổi; macro expansion và EN; false selection/stale range; native enabled/disabled trước launch, trong session và sau wake; Unknown property/read failure; app custom substitutions và OS update. Windows test physical keyboard riêng, không suy từ touch keyboard/native Mac. Import/update explicit false trên Windows vẫn false; explicit true trên Mac vẫn preserved dù OS lock. Test hide-tray→launcher recovery, all3input methods, converter charset/casing order/Undo, batch duplicate paths/partial failure/original hashes.

```gherkin
Feature: F08 Converter parity
  Scenario: Q34 Conversion không sửa tệp nguồn
    Given fixture originals và existing output có hash đã ghi
    When user chọn batch charset và transforms rồi chạy convert
    Then outputs mới có kết quả đúng pipeline upstream
    And hash originals cùng existing output không đổi
    And report written/skipped/failed khớp actual readback
```

## 14. Shortcut, icon style, startup và keyboard recovery

| Requirement | Cases | Traceability |
|---|---|---|
| UI01 / KEY07 | Q35/Q36 | FL12/14, S01/S02, ShortcutBinding/ui.tray_icon_style |
| KEY10 / KEY08 | Q37/Q39 | FL13, recovery native/OS paths,08fault matrix |
| KEY09 / KEY10 | Q38 | Platform startup defaults/safe-start marker |

```gherkin
Feature: Shortcuts và presentation
  Scenario: Q35 Default modifier-only và custom
    Given fresh Mac shortcut là Control+Option và fresh Windows shortcut là Control+Win
    When user nhấn rồi thả đúng hai modifier mà không thêm phím khác
    Then VI/EN đổi đúng một lần
    And custom được lưu và restore sau restart
    And khi thêm phím thứ3 thì shortcut OS nhận input và mode không toggle

  Scenario: Q36 Style nhây không đổi engine state
    Given mode VI và style serious hiện glyph V
    When user chọn style playful trong System
    Then tray hiện Ư và engine vẫn VI
    And khi mode EN thì serious hiện E và playful hiện W
    And label accessibility vẫn là Tiếng Việt hoặc English theo mode

Feature: Keyboard resilience
  Scenario: Q37 Recovery bằng chuột khi processing không phản hồi
    Given input worker fixture unresponsive nhưng native recovery path còn hoạt động
    When user chọn Khôi phục nhập liệu bằng chuột
    Then supervisor hoặc OS source escape hoàn tất đúng owned component
    And gõ fixture bằng source nguyên bản được readback trước báo recovered
    And không kill component app khác hoặc auto resume processing

  Scenario: Q38 Autostart vẫn tắt trong giai đoạn chưa an toàn
    Given fresh config trên Mac hoặc Windows
    When launcher khởi tạo và sau đó cập nhật version
    Then launch_companion_at_login mặc định false
    And không tự register autostart sau check PASS hoặc import
    And safe-start marker sau fault không bị xóa

  Scenario: Q39 UI và backend cùng lỗi
    Given companion và input backend fixture đều không phản hồi
    When user dùng OS alternate input hoặc native process recovery bằng mouse
    Then kiểm readback plain typing và ghi outcome thật
    And không dùng emergency keyboard shortcut làm bằng chứng duy nhất
```

Q35variants: cả2press/release orders,left/right/both physical sides, autorepeat,3rd modifier,key-after-armed,release missing,sleep/RDP/AltGr/remapper/VoiceOver. Q36variants:4SVG/theme/DPI/18pt Ư không clip,missing assets fallback đúng glyph, restart và serious default; bypass/inactive/suspended không bị style giả VI/EN. Q37..39 dùng đầy đủ08fault matrix, deadline và ownership evidence, không gây lỗi trên input stack production.

## 15. Q40 — History shortcut và OS coexistence

```gherkin
Feature: History shortcut theo platform
  Scenario: Q40 Không cướp Ctrl+Win+V của Windows
    Given fresh Mac profile và fresh Windows profile
    When đọc defaults history shortcut
    Then Mac là Control+Command+V và Windows là Win+V với provider default native
    And Windows Win+V vẫn mở clipboard native
    And Windows Ctrl+Win+V không bị VXKey chặn và vẫn mở sound output
    And mode toggle Ctrl+Win không fire khi có phím V đi kèm
```

Custom Windows history: validate OS registration/conflicts; khi failed giữ last valid registration/config. Mac Control+Command+V phải mở đúng1history panel khi companion ready và không biến thành paste text. Clipboard consent off vẫn không tự bật capture do nhấn shortcut.

## 16. Q41/Q42 — Mac Control+V và Windows provider selection

```gherkin
Feature: Clipboard shortcut coexistence
  Scenario: Q41 Kiểm Control+V không tự đổi default Mac
    Given Mac history shortcut đang là Control+Command+V
    When user chưa lưu custom shortcut khác
    Then Control+V không bị VXKey chiếm làm history
    And trong Apple text fixture nó giữ hành vi navigation theo app

  Scenario: Q42 Win+V chọn native hoặc VXKey
    Given Windows provider default native và OS history ở fixture state đã biết
    When user bấm Win+V
    Then route không bị VXKey intercept và OS nhận shortcut
    When user explicit chọn provider VXKey và routing adapter ack thành công
    And user bấm Win+V
    Then chỉ một VXKey panel mở và không native popup song song
    And capture không tự bật nếu chưa consent
    When user chọn native hoặc VXKey quit/crash hoặc input safety suspend
    Then kiểm routing cleanup và Win+V về native bằng actual OS readback
```

Variants: native history on/off, OS build versions, active source ABC/khác, input bypass app, clipboard capture off/denied/locked, renderer unavailable, adapter callback hang, registration conflict, failed persist, owner crash giữa keypress, left/right Win, extra modifiers, remappers/RDP. Ctrl+Win+V luôn giữ sound output, không mode toggle. Native clipboard DB/sync settings không đổi; không chấp nhận popup giả hoặc disable OS history làm bằng chứng chặn Win+V thành công.
