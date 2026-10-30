# 07. Hợp đồng triển khai và các gate bàn giao — VXKey

> **Skill:** `_spec0_system`  
> **Version:** 0.5.1 — **Ngày cập nhật:** 2026-10-30  
> **Trạng thái:** Đã soạn để review; chưa được user xác nhận specs ready.  
> **Đối tượng:** Dev, UI/UX, QA và AI Execution Agents.  
> **Nguồn scope:** [Tổng quan](00_system_overview.md); quyết định UI Tauri chung Mac/Windows của user.

## 1. Hướng đã chọn và nguồn

User chọn Tauri UI chung Mac/Windows trong lượt30/10. Giữ Rust core/FFI + Swift InputMethodKit backend Mac, thay SwiftUI UI bằng Tauri companion. Mac trước, Windows sau. Đọc [index](../index.md) và [core](00_system_overview.md) trước mọi task.

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

Spec không cho phép install/release trong lượt viết tài liệu. M1 tạo artifact isolated; build Mac pin SDK27.0 theo suite, toolchain Rust/Xcode pin trong manifest và CI, không đổi global xcode-select. Minimum OS đề xuất26+ arm64 dogfood, Intel/Windows sau evidence riêng. Review upstream delta trước pin fork, giữ history/provenance/MIT/notices và không sửa xfunput.

Companion Tauri là app launcher người dùng mở; IME native là bundle riêng dưới per-user Library/Input Methods. Companion proposal path per-user Applications/VXKey.app, chưa tự thay /Applications; helper/IME nested payload versioned trong distribution manifest. Installer không dựa tên file để xác nhận ownership: verify bundle ID/signer/manifest/path canonical, reject symlink traversal/unknown existing bundle. Không dùng script Funput killall/delete prefs.

### 5.2. First install và rollback

Validate artifact OS/arch/identity/hash/signature → stage cùng volume đích → verify payload/versions → capture current VXKey paths/receipts/source status → backup bản VXKey cũ → coordinated swap → register/discover IME bằng API hỗ trợ → hướng dẫn user Add Input Source. Không tự thay source. Fail ở stage giữ nguyên bản cũ; fail giữa hai component dùng journal phục hồi cả pair cùng version. Receipt chỉ success khi companion mở đúng version và IME handshake tương thích; nếu cần logout/relogin thì báo pending_activation, không success runtime.

ABC/Funput giữ nguyên; hướng dẫn shortcut OS chuyển ABC trước disable/uninstall. Uninstall yêu cầu user bỏ nguồn VXKey đang active; verify không active rồi stop đúng owned processes, remove owned bundles/receipts. Preferences/history mặc định giữ, xóa opt-in riêng. Không killall theo display name, reset cfprefsd toàn máy hoặc TCC. Interrupt lúc uninstall có journal và retry idempotent.

### 5.3. Permission matrix

| Capability | Owner | Khi hỏi / từ chối |
|---|---|---|
| Đăng ký/chọn IME | macOS input-source flow | Hướng dẫn OS; chưa active vẫn dùng Settings/history |
| Clipboard read | Companion adapter | Chỉ sau opt-in; denied pause + hướng dẫn; kiểm grant/revoke runtime |
| Hotkey registration | Companion / native IME action rõ | Chưa gán mặc định; conflict không lưu giả; không xin quyền event tap mặc định |
| Launch at login | Companion | OS registration status/ack thật; denied hiện lỗi |
| Secure store | Companion, M4b | Key không có thì pause, không lưu plaintext |
| Auto-paste/Accessibility | Ngoài scope | Không hiện control/xin quyền |

Nguồn nhập do OS quản lý, không được giả thành checkbox quyền đã cấp. Manifest signer/DR/capability lưu trước build cài được. Bản local/CI/release identity tương thích; ad-hoc không chứng minh permission retention. Không tự mua chứng chỉ, đổi trust/keychain/TCC hoặc upload secrets.

### 5.4. Updater production

Owner companion Rust; UI notice và Updates dùng cùng job. Dev/internal unsigned/feed chưa có → unsupported/dev với lý do thật, không fake-current. Không check upstream Funput. Production cần signed manifest: app_id, version, protocol/schema versions, os/arch, payload URLs/sizes/hashes, pair companion+IME; trust key thuộc VXKey, không key dự án khác. Endpoint/signer là gate ID01/REL01 trước release.

States idle/checking/current/available/downloading/verifying/quiescing/installing/handed_off/restarting/restart_required/error. Startup check một lần sau ready, offline im lặng; manual error có thông báo. Check timeout15s, download300s; quiesce5s, helper wait PID30s, pair swap60s, relaunch readback30s; timeout install query receipt trước retry. Payload limits publish theo manifest, reject over declared size và disk insufficient trước mutation.

Available: notice trực tiếp “Cập nhật & khởi động lại”, không redirect browser/Settings. Dismiss theo version trong phiên; không download/install trước click. Không ép focus/Space khi panel ẩn; mở lại hiển thị trạng thái. Active composition → quiesce, hướng dẫn user hoàn tất/chuyển ABC khi cần; không kill IMK đang dùng hoặc auto commit client không xác minh. Quiesce timeout giữ bản cũ, không arm install ngầm.

Stage verify cả pair → helper authenticated ack + job receipt trước quit companion → đợi owned PIDs an toàn theo IME lifecycle → journal swap → relaunch/readback pair/version/config → complete. Không thể replace IME an toàn trong session hiện tại → restart_required/logout_required ghi rõ trước mutation, không claim seamless restart. Failure restore old pair và cache/config compatible; schema migration có backup + rollback plan, không downgrade config mới phá dữ liệu. Không xóa app cũ trước verify/stage.

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

M2/M5 bắt buộc fault-injection matrix trong08, mouse/native recovery và OS alternate-source readback. Windows worker phải isolate khỏi Tauri/clipboard process nếu hooks có thể nuốt phím; companion supervisor có thể dừng owned worker bằng chuột khi không ack, không broad killall. Mac ưu tiên OS input-source escape; không tự restart IME loop. Không cài default autostart để thử khi các gate chưa đạt.

UI assets: user cung cấp menubar_serious_vi.svg(V), menubar_serious_en.svg(E), menubar_playful_vi.svg(Ư), menubar_playful_en.svg(W); default serious, Settings tab System. Chưa có asset thì fallback glyph đúng style và ghi visual gate chưa đạt, không vẽ placeholder rồi coi final.

## 9. History shortcut build note

Mac mặc định Control+Command+V, cho custom. Windows giữ Win+V, radio chọn native Windows/VXKey, default native. Ctrl+Win+V vẫn sound output; không chiếm. Native provider-routing spike/rollback/recovery bắt buộc theo04 mục16 và Q40/Q42 trước expose selector như hoạt động thật; generic RegisterHotKey chưa đủ bằng chứng.
