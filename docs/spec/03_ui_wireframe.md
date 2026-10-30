# 03. UI wireframe và component specifications — VXKey

> **Skill:** `_spec3_ui_wireframe`  
> **Version:** 0.5.1 — **Ngày cập nhật:** 2026-10-30  
> **Trạng thái:** Đã soạn để review; chưa được user xác nhận specs ready.  
> **Đối tượng:** Dev, UI/UX, QA và AI Execution Agents.  
> **Nguồn scope:** [Tổng quan](00_system_overview.md); quyết định UI Tauri chung Mac/Windows của user.

## 1. Surface và bố cục

Một React codebase, native adapter riêng Mac/Win. Không tái sử dụng SwiftUI Settings khi chuyển UI. macOS giữ Swift/AppKit cho IMK và system integration; native About/menu/dialog vẫn dùng OS API qua host. Windows không sao chép traffic lights. Chưa phát hành Windows UI/backend placeholder.

| ID | Surface | Layout / hành vi |
|---|---|---|
| S01 | Tray panel | Custom 320 pt wide, cao theo nội dung tối đa work area trừ32; radius12, transparent native window; anchor tray, clamp màn hình; không titlebar |
| S02 | Settings | Shell672×560, sidebar192, header/footer56, body24; min theo nội dung chuẩn và co trong work area trừ32, body scroll; title “VXKey Settings” |
| S03 | Onboarding | Window560×520, body scroll; Back/Next/Để sau rõ; OS-native corner |
| S04 | App picker | Sheet gắn Settings; search tên, path và ID; Cancel không đổi config |
| S05 | History | Custom panel600×460, min480×320 nếu work area cho phép; search trên, list trái240, preview phải flex, footer status; màn nhỏ clamp và một cột list/preview chuyển qua lại |
| S06 | About native | Panel native riêng theo mẫu VXSearch, icon bundle/metadata/version thật/credits; không QR |
| S07 | Update notice | Góc dưới phải surface đang mở, right/bottom20, max384, radius12; owner không phụ thuộc panel |

Titlebar cửa sổ có header áp contract suite standard32 và geometry gate; menu popup không bị ép traffic/titlebar. OS-native menu/corner giữ OS vẽ. Không gắn vùng drag lên input/button; About/Settings menu⌘, dùng một router/single window. React vendor Settings canonical và catalog có manifest/hash, adapter giữ business state.

```text
Tray panel
  VXKey                         trạng thái hiệu lực
  Tiếng Việt [VI]    English [EN]
  Tắt VXKey cho ChatGPT          ← app đang dùng trước khi mở menu
  Quản lý ứng dụng…
  Lịch sử sao chép               phím tắt nếu có
  Settings…                     ⌘,
  About VXKey
  Thoát tiện ích VXKey
```

Bypass có action nhanh theo app ngay trong S01 theo mục9; Quản lý ứng dụng mở danh sách đầy đủ. Chọn VI không tự gỡ rule. Source ABC: label inactive, chọn VI/EN không tự đổi source; hướng dẫn chọn VXKey.

## 2. Settings mapping

Mọi key/capability phải có vi/en catalog cùng key, UI chỉ hiển thị locale đang chọn. Prefix `settings.*` trong bảng là app keys; copy General/Updates/About/Support lấy catalog canonical, không tự viết lại bản vendor.

| Tab / section | Key / capability | Control / default | Apply/persist | Catalog / QA |
|---|---|---|---|---|
| General / Theme | theme | Radio cards System/Dark/Light/Pure Black; system | Ngay + readback | common catalog; UI01/CFG01 |
| General / Language | ui_locale | Radio Tiếng Việt/English; theo OS lần đầu | Ngay toàn UI, không đổi VI/EN | common catalog; UI01 |
| Typing / Method | input_method, tone_style | Select telex/telex_advanced/vni; modern/traditional | Safe composition boundary | settings.typing.*; KEY01/02 |
| Typing / Safety | composition_mode | Preedit/direct verified; direct_verified có safety fallback | Direct cảnh báo giới hạn client | settings.composition.*; KEY01 |
| Typing / Behavior | smart_restore, eager_restore, spell_check, retone, auto_capitalize | Defaults theo00 | Safe boundary | settings.behavior.*; KEY02/05 |
| Applications / Memory | remember_per_app, language_memory | Checkbox true; list + Forget | Ngay, xóa memory riêng | settings.apps.memory.*; KEY04/06 |
| Applications / Policies | input_bypass, clipboard_blocked | Hai checkbox độc lập mỗi exact ID | Ngay sau quiesce nếu cần | settings.apps.policy.*; KEY03/04 |
| Macros / Rules | macros_enabled, macros_in_en, macro_smart_case, macro entries | On/on/on; editable table | Validate rồi persist | settings.macros.*; KEY05 |
| Shortcuts | toggle_hotkey, flip_hotkey, history_hotkey | Recorder; Mac Control+Option / Windows Control+Win; history Mac Control+Command+V, Windows Win+V với chọn provider; flip chưa gán | History global registration / toggle IME ack trước báo active | settings.shortcuts.*; UI01 |
| Clipboard / Capture | clipboard_enabled | Opt-in off, permission/status thật | Reset changeCount baseline | settings.clipboard.*; CLP01 |
| Clipboard / Storage | history_mode, limits | session; encrypted tùy milestone | Confirm persistence; không silent fallback | settings.history.*; CLP03 |
| System | launch_companion_at_login | Checkbox off trong giai đoạn chưa an toàn, OS status thật | OS ack, không fake enabled | settings.system.*; KEY08 |
| Updates | updater state | Version/read-only ở dev; actions thật ở production | Single controller | common + update.*; UPD01 |
| About | metadata/credits | Version build/repo đã xác minh | Read-only | common + about.description; UI01 |
| Support | canonical QR | Asset208, trắng, giữ aspect | Không network lấy QR | common catalog; UI01 |

General không chứa launch-at-login, typing, clipboard hay nghiệp vụ. Footer Reset Defaults trái / Close phải. Reset preferences chỉ theme và typing; giữ locale và toàn bộ data/policies cùng các field được liệt kê tại02 mục10. Macro draft có Save/Cancel riêng, không nút Save chung cho Settings. Macro enable trong EN chỉ có hiệu lực khi macros_enabled=true; disabled helper giải thích.

## 3. History interactions và states

Search focus khi mở, Up/Down chọn, Enter copy, Escape đóng; Cmd/Ctrl+C copy selected nếu search không đang chọn text, nếu có text selection thì giữ hành vi copy native của input. Delete entry qua nút/context menu và confirmation cho clear-all; không dùng Backspace xóa history khi đang gõ search. Preview plain text, không HTML/URL fetch. Truncate preview tối đa2000 Unicode scalars, copy luôn full stored text. List 20 rows đầu, virtualize phần còn lại. Search case-insensitive substring, phân biệt dấu, không tự trim/chuyển nội dung entry; kết quả mới debounce150ms, bỏ result revision cũ.

| State | Hiển thị / action |
|---|---|
| off | “Lịch sử đang tắt” + Bật; không lộ entry cũ nếu đã clear |
| monitoring + empty | “Chưa có nội dung sao chép mới” |
| paused | Banner “Đã tạm dừng”; dữ liệu cũ còn xem; Resume không backfill |
| denied | Hướng dẫn quyền của companion, retry thủ công; không prompt loop |
| locked | Panel đóng, không preview |
| store_error | Lỗi ngắn + Retry/Chẩn đoán; không báo empty hoặc lưu plaintext |
| no_results | “Không tìm thấy kết quả”; Clear search |
| copy_error | Giữ panel/selection, error inline |

## 4. Accessibility, theme, i18n

UI tokens lấy master theme; system font12 labels,11 helpers,14 headings; native alpha xuyên window/WebView/root, surface clip12/shadow ngoài. Theme cards chỉ hiện mode thực; pure_black phải có CSS thật trước expose. Focus ring, accessible label cho icon và radio, tooltip shortcut, keyboard xuyên tabs/list/modal. Không chỉ dùng màu biểu đạt state. Giảm animation theo OS reduce motion. Không OCR/voice feature phát sinh.

UI locale và language_mode độc lập: English UI vẫn gõ VI. Mọi errors/dialog/menu/tooltip/onboarding dịch vi/en; path, code và app names giữ nguyên. Không ép AppleLanguages=vi như upstream. Persist locale, reload đúng; catalog check cộng packaged runtime. Missing runtime key là lỗi, không dùng raw key làm nhãn. Repo/support URL chưa xác minh thì chưa render link giả.


## 5. Component hierarchy, props và wireframes

```text
VXKeyCompanion
├── NativeTrayAdapter → TrayPanel (S01)
├── SettingsWindow (S02)
│   ├── SharedSettingsShell: Header / Sidebar / Footer
│   ├── General / Typing / Applications / Macros / Shortcuts / Clipboard / System
│   └── SharedUpdates / SharedAbout / SharedSupport
├── OnboardingWindow (S03)
├── NativeAppPickerSheet (S04)
├── HistoryPanel (S05): Search / EntryList / TextPreview / StatusActions
├── NativeAboutPanel (S06)
└── SharedUpdateNotice (S07)
```

### S02 — Settings wireframe

```text
┌────────────────── VXKey Settings ────────────────────────────────┐
│ Chung             │ GIAO DIỆN & MÀU SẮC                          │
│ Bộ gõ             │ [ System       ] [ Dark          ]           │
│ Ứng dụng          │ [ Light        ] [ Pure Black    ]           │
│ Gõ tắt            │ ─────────────────────────────────            │
│ Phím tắt          │ NGÔN NGỮ                                     │
│ Lịch sử sao chép   │ (•) Tiếng Việt        ( ) English            │
│ Hệ thống          │                                               │
│ ────────────────  │         Body cuộn độc lập khi cần             │
│ Cập nhật          │                                               │
│ Giới thiệu        │                                               │
│ Ủng hộ tác giả    │                                               │
├───────────────────┴───────────────────────────────────────────────┤
│ Khôi phục mặc định                                      Đóng     │
└───────────────────────────────────────────────────────────────────┘
```

### S05 — History wireframe

```text
┌──────────────────────────────────────────────────────────────────┐
│ Tìm nội dung đã sao chép…                                  [×]  │
├──────────────────────────┬───────────────────────────────────────┤
│ Dòng1: excerpt           │ Plain-text preview                    │
│ Ứng dụng · thời gian     │ giữ whitespace và xuống dòng          │
│ ──────────────────────── │ Không render HTML hoặc tải URL        │
│ Dòng2: excerpt           │                                       │
│                          │                                       │
├──────────────────────────┴───────────────────────────────────────┤
│ Chỉ lưu trong phiên · 12/200      Tạm dừng      Sao chép ↵        │
└──────────────────────────────────────────────────────────────────┘
```

| Component | Props / input types | Callback | States / accessibility |
|---|---|---|---|
| `LanguageModeSelector` | effective_mode enum, target_token nullable, busy bool | set_language(vi/en, token) | active/disabled/stale_target; radio semantic, label locale |
| `AppPolicyRow` | app_id string, display_name string, memory vi/en/null, input_bypass bool, clipboard_blocked bool | patch one rule / forget | resolving/ready/missing_app/saving/error; checkbox nhãn độc lập |
| `HotkeyRecorder` | value accelerator/null, recording bool, registration_error nullable | validate_and_register/cancel | idle/recording/conflict/unsupported; Escape hủy recorder trước khi đóng window |
| `HistorySearch` | query string≤256 scalars, request_generation int | search/clear | idle/loading/no_results/error; focus khi mở |
| `HistoryEntryRow` | entry_id UUID, preview≤200 scalars, timestamp, selected bool | select/copy/delete | default/hover/selected/expired/disabled; listbox/option |
| `HistoryPreview` | selected text≤128KiB, full_byte_length int | copy | loading/ready/error/locked; preformatted plain text |
| `CaptureStatus` | state enum, history_mode, count, permission_status | pause/resume/retry | off/monitoring/paused/locked/denied/store_error |
| `PersistStatus` | config_revision, applied_revision, operation state | retry_if_safe | saving/saved/pending_engine/applied_not_saved/error |

List excerpt200 scalars và preview2000 scalars chỉ là presentation; full content chỉ fetch entry được chọn. Không dùng CSS truncation như dữ liệu copy. Native dialog picker mở trên đúng owner S02; Cancel/one-Escape trả focus về control gọi, không đóng cả Settings hoặc mất rule nháp. Clear-all confirmation có Cancel default, xác nhận ghi rõ chỉ history VXKey.

### Responsive/native adaptation

| Surface | Bình thường | Work area nhỏ / scale lớn |
|---|---|---|
| Settings |672×560, sidebar192 | Clamp work area−32; sidebar nội dung cuộn, body min-height0; không cắt footer; nếu work area không đủ minimum, layout hẹp sidebar icon+label thu gọn nhưng mọi tab accessible |
| History |600×460 split | Dưới600 logical px dùng list một cột; chọn entry mở preview có Back; giữ Search/Copy/Escape |
| Tray |320 wide, content height | Clamp x/y theo work area, không tràn dưới/ngoài monitor; nội dung cuộn |
| Onboarding |560×520 | Footer giữ visible, nội dung cuộn, không auto resize sang màn khác |

Mốc layout là logical px/pt, không nhân đôi theo Retina; trên Win dùng DPI adapter. Text scaling/hệ thống high contrast kiểm riêng; không dùng web mobile viewport375 như tiêu chí cho app desktop.

## 6. Menubar/tray và asset contract

### 6.1. Source và đích

Upstream Mac [FunputApp.swift](../../../xfunput/platforms/macos/Funput/FunputApp.swift) dùng SwiftUI MenuBarExtra và chữ VI/EN. VXKey đích chuyển tray owner sang Tauri companion, có native tray integration và React custom panel. Không giữ MenuBarExtra thứ hai trong IME sau migration. Windows reuse panel/copy nhưng tray rendering adapter riêng.

### 6.2. Bốn SVG nguồn và hai style

`menubar_serious_vi.svg` (V), `menubar_serious_en.svg` (E), `menubar_playful_vi.svg` (Ư) và `menubar_playful_en.svg` (W): cùng viewBox0 0 24 24, baseline/optical weight, đơn sắc/nền trong suốt, chữ path, không gradient/shadow/background squircle. Display đề xuất18×18pt; size cuối theo native QA. Không dùng app icon màu làm status icon.

SVG là master; Mac convert thành template asset/PNG18/36px cho logical18pt hoặc định dạng native pipeline đã chứng minh. Tauri tray adapter phải expose template semantics trên Mac nếu API/plugin thiếu thì bổ sung native bridge; không mặc định nạp SVG trực tiếp hoặc template tint tự hoạt động. Windows xuất raster icon phù hợp DPI/theme, kiểm taskbar/tray thật; không claim monochrome template Mac tự xử lý Windows.

Bốn SVG do user chuẩn bị, chưa được tạo/duyệt trong task viết spec. App icon PNG/ICNS/ICO hiện có dùng bundle/Dock/About, không lẫn với glyph titlebar hoặc status item.

### 6.3. State mapping

| effective_mode | Hiển thị | Accessible label vi / en |
|---|---|---|
| vi | V (serious) hoặc Ư (playful) | VXKey — Tiếng Việt / VXKey — Vietnamese |
| en | E (serious) hoặc W (playful) | VXKey — English |
| bypass | Native text BY, cùng khung | VXKey — Không xử lý phím / VXKey — Input bypassed |
| inactive | Native text VX | VXKey — Nguồn nhập khác / VXKey — Another input source |
| suspended | Native text VX, status tạm ngừng | VXKey — Đã tạm ngừng / VXKey — Input processing suspended |
| unavailable | Native text VX + tooltip lỗi | VXKey — Bộ gõ chưa sẵn sàng / VXKey — Input engine unavailable |

BY/VX là native label fallback, không cần thêm SVG ngoài4asset user chuẩn bị. Nếu platform tray không hỗ trợ text label, adapter render label thành raster cùng logical frame; visual QA bắt buộc. Bypass không giả EN. Khi snapshot stale hoặc source ABC không hiển thị VI đang live. Text fallback V/E hoặc Ư/W theo style lúc thiếu asset vẫn phải đúng state và báo asset issue trong QA.

Click mở panel, không toggle mode ngầm; VI/EN là action trong panel/hotkey riêng, target validation theo feature flow. Right-click context menu có Settings/About/Quit tiện ích. Một icon dù IME/companion restart; một primary instance. Giữ show_tray_icon như upstream; khi ẩn icon, launcher vẫn mở Settings để bật lại, không nhân process.

### 6.4. Acceptance

Q07/Q12: kích thước ổn định, đổi state không dịch menubar; light/dark, Retina, màn1x nếu support, tray Win DPI riêng. Native alpha/corners12/focus/Escape/click ngoài/anchor/clamp và accessibility. Convert asset + inspect AppKit logical size, screenshot artifact thật; header PNG/SVG browser không thay visual acceptance. Không thêm light/dark SVG khi template pipeline đã xử lý đúng.


## 7. Visual wireframe prompts cho UI Agent

Các prompt chỉ mô tả mockup; layout/token/capability ở trên là contract. Không yêu cầu ImageGen hoặc Figma trong task này.

**Settings:**
```text
Create a precise desktop wireframe for VXKey Settings, a Tauri application with native macOS window controls. Use a 672 by 560 logical-unit shell, a 192-unit left sidebar, fixed 56-unit header and footer, and a scrolling content region. Show General selected, with Theme cards followed by a Vietnamese/English language radio row. Keep input-method, application policy, macros, shortcuts and clipboard settings in separate business tabs. Match the VX canonical Settings component; cyan accent, system typography, compact 12-unit labels. Show one selected theme and visible keyboard focus. Do not place debug identifiers or architecture details in user labels.
```

**History:**
```text
Create a desktop clipboard-history wireframe for VXKey, 600 by 460 logical units. The custom floating panel has transparent outer corners, a clipped 12-unit-radius surface, a search field at the top, a 240-unit entry list on the left and a plain-text preview on the right. Include a session-only status, pause control and Copy action in the footer. Use multiline Vietnamese sample text and an emoji. Show an empty state and a permission-denied variant separately. Enter copies; it does not paste into another application.
```

**Tray:**
```text
Create a compact 320-unit-wide VXKey tray panel anchored below a monochrome VI status icon. Present Vietnamese and English mode choices, the target application and a separate input-bypass status, followed by clipboard history, Settings, About and Quit utility actions. Keep a 12-unit corner radius, a subtle border, readable focus outlines and no titlebar. Provide light and dark mockups using the VX theme tokens.
```

## 8. Shortcut UI states

Recorder phân biệt “Đổi VI/EN trong nguồn nhập VXKey” với “Mở history toàn hệ thống”. History báo OS registration conflict thật; toggle báo validation/IME apply pending, không nhãn “global active” khi chỉ IME-local. Không hai listener cùng xử lý tổ hợp. Shortcut trùng giữa hai action bị reject. UI không hứa phát hiện mọi shortcut của app khác; Cancel recorder không thay registration/config.

## 9. Quick action — Loại trừ app đang dùng

User tham chiếu menu MKey có mục “Tắt mkey cho ChatGPT” và yêu cầu áp dụng cơ chế thao tác ngay theo app hiện tại. VXKey đưa action này ngay sau lựa chọn VI/EN trong S01; không bắt mở Settings hoặc tìm app trong picker.

| App target / policy | Label vi | Label en | Action |
|---|---|---|---|
| App ngoài VXKey, bypass=false | Tắt VXKey cho {app_name} | Disable VXKey for {app_name} | input_bypass=true |
| App ngoài VXKey, bypass=true | Bật VXKey cho {app_name} | Enable VXKey for {app_name} | input_bypass=false |
| Target không xác định / internal app | Không xác định ứng dụng đang dùng | Current application unavailable | Disabled, không dùng app cũ |
| Đang lưu | Đang áp dụng… | Applying… | Disable click lặp |
| Target stale | Ứng dụng đang dùng đã thay đổi | The active application has changed | Không mutation; đóng/mở menu để lấy target mới |
| Persist error | Không lưu được thay đổi | Could not save the change | Giữ state đã commit, error inline, retry do user |

`app_name` lấy metadata để hiển thị; không match theo tên. Tooltip/help: “Bỏ qua xử lý phím trong ứng dụng này. Không thay đổi lịch sử sao chép.” Khi bật lại, khôi phục language memory/default có sẵn; không ép VI. “Tắt VXKey cho app” không quit companion/IME, không tắt VXKey toàn máy, không tự chặn clipboard.

Target là external app được chụp ngay trước lúc menu nhận focus, không phải app VXKey sau click. Label giữ nguyên tên target trong phiên menu; nếu user chuyển external app thì token invalid, không âm thầm đổi label rồi áp cho app khác. Quản lý ứng dụng vẫn mở Settings→Applications để xem/xóa rules và chỉnh clipboard policy độc lập.

Component `CurrentAppBypassAction`: props `{app_name:string|null,target_token:UUID|null,input_bypass:boolean,operation_state:idle|saving|pending_engine|error}`; callback `set_current_app_bypass(target_token, desired_value, expected_revision)`. Native menu adapter nếu có phải dùng cùng command/state/target rule như React panel.

## 10. Bổ sung Settings parity và dấu chấm hai Space

Typing có3mode Telex, Telex nâng cao, VNI; helper nâng cao ghi `w→ư` khi chưa có nguyên âm chính, `]→ư`, `[→ơ`; ví dụ `vwng→vưng`. Giữ eager restore (disabled khi smart_restore=false, không xóa preference), spell check, retone, autocap và extra onset master switch/từng chữ. Bật master extra onset chọn cả z/f/w/j; tắt clear; bỏ chọn chữ cuối thì master off. Gõ tắt giữ smart-case upstream. Shortcuts có thêm lật từ vừa gõ (IME-local, không global remap).

| Native macOS period state | Toggle VXKey | Effective VXKey transform | Helper |
|---|---|---|---|
| enabled | Disabled, thể hiện native đang xử lý | false | “macOS đang bật thêm dấu chấm bằng hai dấu cách. Tắt trong cài đặt macOS để dùng tùy chọn VXKey.” |
| disabled | Enabled, checked theo stored preference | Theo toggle và safe context | “Nhấn Space hai lần nhanh để chèn dấu chấm và một dấu cách.” |
| unknown/checking | Disabled tạm thời | false | “Chưa xác định được cài đặt macOS. Thử kiểm tra lại.” |
| Windows | Enabled | Theo toggle và adapter capability | Fresh install default bật |

Không đổi giá trị VXKey đã lưu khi khóa do OS; khi OS tắt thì toggle trở lại giá trị user trước đó. Đổi setting macOS khi VXKey đang mở phải refresh cả UI/backend, cancel pending double-space. Không tự thay setting OS. Khi UI toggle off, native macOS vẫn có thể xử lý nếu user bật native; không hiển thị lời hứa “tắt toàn hệ thống”. Locale EN: “Period with double-space”; native enabled helper “Managed by macOS. Turn it off in macOS settings to use VXKey.”

System bổ sung Hiện biểu tượng thanh menu/tray; toggle ẩn không quit engine/companion. Launcher là đường mở Settings để hiện lại.

S08 Conversion: window920×620, min760×520/clamp work area; Tauri layout header→source/target charset→input/output preview hoặc batch rows→ordered transform chips/options→progress/result actions. Empty có Paste/Choose Files/Drop; text có Copy/Save; batch có charset từng dòng, unreadable/skipped/failed counts và action chạy. Unknown charset không auto đoán thành công. Transforms uppercase/lowercase/no-diacritics/sentence/title, Undo/Clear và keep_d/flatten_caps. Native file dialogs owned bởi S08; không sửa originals/in-place. Không để byte encoding hoặc file path generic trở thành arbitrary shell command.

## 11. Settings icon style, shortcut modifier-only và recovery

System→Biểu tượng: radio “Nghiêm túc”/“Nhây” (EN “Serious”/“Playful”), có preview cặp V/E và Ư/W. Key ui.tray_icon_style, default serious; apply ngay trên icon live, persist/readback sau restart, không reset mode/memory/clipboard/shortcut. Đặt ở tab System, không thêm mục thứ3 vào General. Common frame/viewBox/padding của cả4asset; chữ Ư phải đủ khoảng cho dấu móc, không clip ở18pt. Theme/template/DPI pipeline như mục6; không yêu cầu user vẽ4bản light/dark.

Shortcut recorder toggle cho phép cả modifier-only và modifier+key; label Mac `⌃⌥`, Windows `Ctrl+Win`. Help: “Nhấn rồi thả cả tổ hợp để đổi Việt/Anh.” Recorder nhận đủ cả chu kỳ rồi mới lưu; Escape hủy, combo có phím khác không thành modifier-only. Preset mặc định giữ theo user; custom được validate theo adapter, không silently replace. Phím lật và history giữ key-chord; history dùng OS registration, modifier-only không đi RegisterHotKey.

S01 thêm “Tạm ngừng xử lý phím” và “Khôi phục nhập liệu…” bằng chuột. Suspended state label rõ: icon neutral VX, text “Đã tạm ngừng”, không báo VI/EN đang live. Nút Resume explicit chỉ chạy sau health check. Recovery window mở được từ launcher ngay cả khi WebView history/engine lỗi; cung cấp action native tối thiểu nếu renderer không khởi tạo, không chỉ nút nằm trong React đã treo. Mac: hướng dẫn/chọn nguồn thay thế OS bằng action user; Windows: stop owned input worker/unhook theo contract08, không kill app khác.

Startup mặc định off; helper giải thích đang ở giai đoạn kiểm chứng an toàn. OS may giữ source VXKey từ phiên trước dù companion autostart off: onboarding phải nói rõ cách chuyển ABC hoặc input source thay thế bằng chuột. Không coi checkbox off là đã ngăn IME tự nạp.

## 12. History shortcut theo platform

Mac: Control+Command+V (⌃⌘V), custom được. Windows: giữ Win+V, provider native Windows hoặc VXKey theo mục13; không map Ctrl+Win+V vì Windows11 dùng cho sound output/volume mixer. Recorder Windows báo xung đột đã biết nếu nhập Ctrl+Win+V; không giả register thành công hoặc disable native OS. Global history shortcut có phím V phải cancel candidate Ctrl+Win mode-toggle trên Windows, không đổi VI/EN sau khi mở history hoặc action OS.

## 13. Windows history provider selector

Windows Settings→Clipboard→“Mở lịch sử bằng Win+V”: radio “Clipboard Windows” / “Clipboard VXKey” (EN “Windows clipboard” / “VXKey clipboard”), default Windows. Hiện shortcut Win+V read-only tại selector; Mac không có control Windows này và vẫn có custom history shortcut. Provider native không intercept Win+V; provider VXKey mở S05 và suppress native popup đúng một action. Không gộp provider selection với capture consent toggle.

States: native / preparing_vxkey / vxkey_active / restoring_native / unavailable/error. Chỉ báo active khi native routing adapter ack thật. Nếu unsupported/conflict, giữ native hiệu lực và hiển thị lý do; không checkbox giả. Native mode mà native history OS chưa enabled thì giữ popup/hướng dẫn OS như vốn có, không tự bật OS setting. VXKey mode nhưng capture chưa consent thì mở S05 off/consent, không tự bắt đầu ghi history.

Đổi provider không xóa history bên nào, không import OS history hoặc tắt storage/sync của Windows. Helper nói rõ “Lựa chọn này chỉ đổi bảng mở bằng Win+V”; có action mở Clipboard settings OS nếu user muốn tự quản lý. Quit/crash/suspend/recovery phải release routing về native; reopen mặc định không giành lại khi safe-start marker đang bật.
