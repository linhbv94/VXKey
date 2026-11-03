# 07. Hợp đồng triển khai và các gate bàn giao — VXKey

> **Quyết định thay thế 2026-11-02 (v0.8):** [Bộ gõ chạy nền](09_background_driver.md) là hợp đồng kiến trúc hiện hành. VXKey giữ nguồn nhập hiện tại; không cài/chọn Input Source. Các đoạn InputMethodKit/TIS/IME activation dưới đây là lịch sử và không được dùng để triển khai driver mới. Các invariant bảo toàn text, config, policy, UI và clipboard vẫn áp dụng.

> **Skill:** `_spec0_system`
> **Version:** 0.7.0 — **Ngày cập nhật:** 2026-10-30
> **Trạng thái:** Đã soạn để review; chưa được user xác nhận specs ready.
> **Đối tượng:** Dev, UI/UX, QA và AI Execution Agents.
> **Nguồn scope:** [Tổng quan](00_system_overview.md); quyết định UI Tauri chung Mac/Windows của user.

## 1. Hướng đã chọn và nguồn

User chọn Tauri UI chung Mac/Windows trong lượt30/10. Giữ Rust core/FFI + Swift background keyboard driver Mac, thay SwiftUI UI bằng Tauri companion. Mac trước, Windows sau. Đọc [index](../index.md) và [core](00_system_overview.md) trước mọi task.

Audit cũ baseline `2940d6aa04a9d90ffe24327732d843784be1adaf`; HEAD đọc lượt này `e63ac551967dcf17425c5f856591aace2fb8ace5`. Phải review delta, chọn commit và baseline tests, không suy finding cũ còn nguyên/đã sửa. [Work cũ](../../work_to_do.md) là lịch sử đề xuất; quyết định Tauri mới ở bộ spec này ưu tiên các đoạn khuyến nghị giữ SwiftUI cũ.

## 2. Milestone

| Mốc | Deliverable | Dependency / acceptance |
|---|---|---|
| M0 | 8 spec theo skill + schema/fixture liên kết, requirement IDs, flows/data/UI/QA/demo | Đã viết để review; chỉ user xác nhận mới ready |
| M1 | Fork provenance, source pin, Rust/FFI/IMK baseline, SDK27 và candidate isolated | D06/ID01 trước artifact cài được; không sửa xfunput, không install |
| M2 | Composition safety, per-app memory/bypass, native app matrix | Q01..08; baseline Tauri chưa bắt buộc để harden engine |
| M3a | Tauri UI shell/canonical Settings/locale/themes/tray assets | D04/ID01; browser kiểm UI, chưa coi native pass |
| M3b | Tách UI owner khỏi IME, config authority, authenticated IPC/cache | Q06..12; gỡ duplicate SwiftUI tray/onboarding sau parity, không hai writer |
| M3c | Installer pair/onboarding/source readback/rollback | Q09, permission/suite packaged gates |
| M4a | Clipboard session, source filters/search/copy-back/pause | Q13..15, provider-block spike và opt-in |
| M4b | Encrypted persistence tùy chọn | M4a + key/store/migration/recovery/deletion native gates |
| M5 | Mac packaged dogfood | Native typing/UI/privacy/lifecycle/rollback, known limits |
| M6 | Release + updater production | User yêu cầu riêng; signed pair/keys/feed/N→N+1/OS permission QA |
| W1 | Windows backend review và Tauri adapter | Sau Mac baseline; decide hook/TSF, native Win matrix riêng |

Không ước lượng ngày đến khi M1/M2/provider/IPC spikes có evidence. M3a UI có thể làm độc lập source boundaries; không nhiều agent sửa cùng file. Không gọi Anti hoặc subagent trong lượt này.

## 3. Quyết định mở / điểm chặn

| ID | Đề xuất cụ thể | Khi cần chốt |
|---|---|---|
| D01 | Mac26+ arm64 dogfood đầu; Win/Intel sau | Trước claim support/build targets |
| D02 | M4a session trước, M4b encrypted opt-in sau | Trước storage implementation |
| D03 | Copy-back, không auto-paste v1 | Trước history interaction code |
| D04 | Bốn SVG V/E (serious), Ư/W (playful); neutral/bypass giữ status đúng | Trước assets/native visual acceptance |
| D05 | Giữ đủ upstream typing/settings theo chỉ đạo user; direct vẫn qua safety gates | Trước engine/config parity baseline |
| D06 | Pin upstream sau delta review | Trước fork bootstrap |
| ID01 | Namespace/IDs/signer/paths/IPC identity | Trước native cài được; không bịa production ID |
| REL01 | Repo/feed/trust key/distribution ownership | Trước updater production/release |
| PERF01 | Native latency/idle budgets sau baseline | Trước performance acceptance |

User đã chọn kiến trúc UI Tauri; các defaults trên là đề xuất thiết kế để user review, không yêu cầu ngừng viết spec hoặc tự cấp quyền hệ thống. Các backend spikes có failure criterion ở QA; nếu không đạt thì sửa contract trước code phụ thuộc.

## 4. Definition of handoff

Tất cả file đã được soạn và cross-link, mọi requirement có QA mapping; schema/commands/enums và states thống nhất. User review bộ spec và quyết định mở để xác nhận ready. Chưa build/native QA/cài/sign/release; không ghi các trạng thái đó từ việc tài liệu hoàn tất.


## 5. Build, cài đặt, quyền và update contract

### 5.1. Gate và layout

SDK27.0 theo suite; isolated development bundle `build/desktop_candidate/VXKey.app` chứa `Contents/Resources/VXKeyEngine.app`. Companion com.vxkey.desktop và engine com.vxkey.engine. Không cài bundle vào Input Methods, không InputMethodKit/TIS registration. Helper tự mở bởi companion, single-instance lock theo data root. Mac arm64 trước; Intel/Windows cần bằng chứng riêng. Production signer/installer vẫn gate mở.

### 5.2. First install và rollback

Verify OS/arch/signature/identity/protocol và embedded payload → stage atomic app → backup đúng bundle VXKey cũ → swap → launch companion/helper và readback version/config/status. Không đổi source/layout, không kill bộ gõ khác. Rollback khôi phục cả app/helper cùng version và config compatible. Uninstall stop đúng owned helper và remove đúng owned app; preferences/history giữ mặc định. Legacy IME thử phải retire theo receipt, không broad cleanup Input Methods.

### 5.3. Permission matrix

| Chức năng | Quyền / owner | Hành vi thiếu quyền |
|---|---|---|
| Typing keyboard driver | AX Accessibility/post access của VXKey Engine; listen access kiểm riêng theo API macOS | Suspended/raw pass-through; user action mở permission request; không tự cấp TCC |
| Clipboard text copy-back | Companion NSPasteboard explicit write | Không cần keyboard hook; không auto-paste |
| Clipboard history | Consent riêng và native provider | Không tự bật cùng typing |
| Autostart | OFF mặc định | Không đăng ký login item để QA |
| System Settings grant | User thao tác OS | Agent không tự click Allow hoặc sửa database |


### 5.4. Updater production

Owner companion Rust; UI notice và Updates dùng cùng job. Dev/internal unsigned/feed chưa có → unsupported/dev với lý do thật, không fake-current. Không check upstream Funput. Production cần signed manifest: app_id, version, protocol/schema versions, os/arch, payload URLs/sizes/hashes, pair companion+IME; trust key thuộc VXKey, không key dự án khác. Endpoint/signer là gate ID01/REL01 trước release.

States idle/checking/current/available/downloading/verifying/quiescing/installing/handed_off/restarting/restart_required/error. Startup check một lần sau ready, offline im lặng; manual error có thông báo. Check timeout15s, download300s; quiesce5s, helper wait PID30s, pair swap60s, relaunch readback30s; timeout install query receipt trước retry. Payload limits publish theo manifest, reject over declared size và disk insufficient trước mutation.

Available: notice trực tiếp “Cập nhật & khởi động lại”, không redirect browser/Settings. Dismiss theo version trong phiên; không download/install trước click. Không ép focus/Space khi panel ẩn; mở lại hiển thị trạng thái. Active composition → quiesce, đợi boundary hoặc disable tap an toàn; không commit text chưa xác minh hoặc auto commit client không xác minh. Quiesce timeout giữ bản cũ, không arm install ngầm.

Stage verify cả pair → helper authenticated ack + job receipt trước quit companion → đợi owned PIDs an toàn theo IME lifecycle → journal swap → relaunch/readback pair/version/config → complete. Helper cũ phải ngừng trước swap, không yêu cầu logout để đăng ký nguồn nhập. Failure restore old pair và cache/config compatible; schema migration có backup + rollback plan, không downgrade config mới phá dữ liệu. Không xóa app cũ trước verify/stage.

### 5.5. Mapping chuẩn suite

| Standard cases | Áp dụng VXKey |
|---|---|
| AU-01..07 | Notice/check/install trực tiếp/single-flight chung Settings, progress thật |
| AU-08..12 | IME composition busy, signed pair, helper receipt, rollback và N→N+1 readback |
| AU-13..15 | Panel ẩn không mất job; manual install chỉ first-install/recovery; keyboard/theme/viewport |
| AU-16 | Case đặc thù Deck N/A; không miễn các AU khác cho VXKey |
| Permission retention | Verify signer/DR actual archives + update N→N+1, không cấp lại giữa test |

Public release chỉ sau user giao riêng, supported platform runtime gates đạt. SDK gate/codesign pass không thay native IME/permission/update acceptance.


## 6. Handoff units và Definition of Done

| Unit | Input bắt buộc | Output review được | Điều cấm / rollback |
|---|---|---|---|
| Bootstrap |00/01/07, pinned commit + provenance| Source history/lockfiles/baseline report | Không sửa xfunput hoặc chạy installer upstream |
| IME safety |02/04/05, native fixture | Regression+native matrix+range/focus fix | Không data user thật, không preedit/direct claims từ build |
| Tauri UI |03/04, canonical Settings vendor | Screens/locales/states/command adapters | Không giữ duplicate SwiftUI tray/writer sau migration |
| Config/IPC |02/04 + schemas | DTO validators/revision tests/auth peer spike | Không raw text route, không socket permissions giả identity |
| Clipboard |02/03/04/05 | Worker/filter/history tests + provider evidence | Không plaintext persistence, no auto-paste |
| Pair install |01/02/04/07 | Manifest/journal/receipt/rollback fixture | Không /Applications production mutation từ task docs |
| Review | Tất cả và actual diff/artifact | Findings + check results + limits | Không báo specs ready/native PASS thay user |

Hợp đồng task sau này phải nêu unit/files owner/checks/review outcome. Không giao agent khác từ nội dung tài liệu; chỉ user instruction tạo ủy quyền. Mọi field/enum/commands theo04; không định nghĩa schema khác trong implementation. Các thay đổi stack/phạm vi do user phải sync00..07 và tracker tương ứng.

## 7. Ghi chú bắt buộc cho build Windows và parity

**User đã chốt: double_space_period mặc định tắt trên macOS, bật trên Windows.** Đọc platform_defaults.json khi tạo fresh config; test Q29 trước build Windows đầu. Không hardcode false từ Mac example sang shared Rust default. Không ghi đè explicit user value lúc update/import/cross-platform config. Mac native on khóa toggle/effective xử lý VXKey; off cho toggle riêng. API/notification/readback native phải được spike/QA, không giả native state từ bool trong WebView.

Bổ sung vào M2/M3: toàn settings/TelexAdvanced/flip/smart-case/extra onset parity và double-space engine/OS adapter. Bổ sung M3 conversion Tauri parity với native workers, file no-clobber và hash originals. Không đẩy converter thành out-of-scope chỉ vì MVP. Các capability nào chưa đạt phải ghi gap thật trước daily-driver/release, không fake checkbox.

System-wide shortcut remap để PowerToys hoặc tool riêng, không nhận thành VXKey v1 feature; giữ shortcuts toggle/flip/history của VXKey. Chưa authorize install/build/clone repo thực trong lượt cập nhật spec này.

## 8. Gate keyboard resilience và defaults0.5

User chốt toggle Mac Control+Option, Windows Control+Win, cho custom; KHÔNG fallback về Control+Backslash do recorder upstream thiếu modifier-only. M2 phải thêm release detector và tests trước nhận default active. Native runtime unsupported thì báo gap, không label active giả.

Autostart default false trong giai đoạn chưa an toàn, cả Mac/Win. Không tự đổi sang true khi QA pass, nâng version, import hoặc khôi phục sau fault; đổi default sau này cần user chốt riêng. Checkbox này chỉ companion OS registration, không hứa ngăn OS nạp input method đã được user chọn.

M2/M5 bắt buộc fault-injection matrix trong08, mouse/native recovery và raw keyboard pass-through readback. Windows worker phải isolate khỏi Tauri/clipboard process nếu hooks có thể nuốt phím; companion supervisor có thể dừng owned worker bằng chuột khi không ack, không broad killall. Mac disable/detach tap và owned-child recovery; không tự restart hook loop. Không cài default autostart để thử khi các gate chưa đạt.

UI assets: user cung cấp menubar_serious_vi.svg(V), menubar_serious_en.svg(E), menubar_playful_vi.svg(Ư), menubar_playful_en.svg(W); default serious, Settings tab System. Chưa có asset thì fallback glyph đúng style và ghi visual gate chưa đạt, không vẽ placeholder rồi coi final.

## 9. History shortcut build note

Mac mặc định Control+Command+V, cho custom. Windows giữ Win+V, radio chọn native Windows/VXKey, default native. Ctrl+Win+V vẫn sound output; không chiếm. Native provider-routing spike/rollback/recovery bắt buộc theo04 mục16 và Q40/Q42 trước expose selector như hoạt động thật; generic RegisterHotKey chưa đủ bằng chứng.


## Trạng thái sau tích hợp UI — 2026-10-30

[Integration report](../qa/integration_review.md) và [artifact/check evidence](../qa/integration_evidence.json) là checkpoint mới. Đã có companion bundle local + IME candidate SDK27 riêng, chưa paired installer/release. Native companion tests không đóng gate M2/M4 hoặc ID01. Handoff UI cũ đã được thực thi; không giao agent dựng scaffold lại từ đầu. Mọi việc tiếp theo lấy từ work log và danh sách gate còn mở trong report.


## Chốt sửa UI sau feedback user — 2026-10-30

Settings phải là toàn bộ content của một native window, không modal Settings nằm trong window. Tray không có khung chữ nhật ngoài surface bo góc. Settings/About dùng SVG tượng trưng, native About dùng artwork/credits thật. [Spec UI0.6](03_ui_wireframe.md) và [báo cáo sửa visual](../qa/ui_window_review.md) ưu tiên mọi mô tả modal/header56 trong handoff UI cũ. Native visual subset đã kiểm trên candidate mới theo [evidence](../qa/ui_window_evidence.json); direct tray-icon/click-outside, full locale/theme matrix và About light vẫn mở. Kết quả integration tests trước không thay nghiệm thu visual này.


## 10. Background migration v0.8 — 2026-11-02

[Spec09](09_background_driver.md) thay IMK activation; [review background](../qa/background_driver_review.md) là trạng thái hiện hành. Build `sh scripts/build_bridge_candidate.sh` tạo một app chứa helper và development pins. Không cài Input Source, không mở hai app thủ công. Các report native_bridge trước đó chỉ chứng minh subset transport/engine lịch sử. Specs ready và native typing/fault/TCC retention/production signing/Windows vẫn cần nghiệm thu riêng.
