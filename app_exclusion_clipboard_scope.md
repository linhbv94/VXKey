# VXKey Mac — Loại trừ app và scope Clipboard History

Ngày: 2026-10-27. **Audit + đề xuất scope, chưa implementation.** User muốn tích hợp lịch sử clipboard vào VXKey; Mac trước, Windows backlog. Source vẫn [xfunput](../../xfunput) HEAD `2940d6aa04a9d90ffe24327732d843784be1adaf`, đích VXKey trống. [Audit nền](audit_2026_10_27.md), [work to do](work_to_do.md), [evidence lượt này](app_exclusion_evidence_2026_10_27.json).

## 1. Funput hiện “loại trừ app” như thế nào?

**Không còn hard exclusion list trên Mac.** Cơ chế hiện tại là nhớ VI/EN theo từng bundle ID. [AppLanguageMemory.swift](../../xfunput/platforms/macos/Funput/Model/AppLanguageMemory.swift) có persisted map `appLanguageMemory: [String: Bool]` trong UserDefaults và FFI handle chỉ lookup/ghi memory, không tự làm disk IO.

| Tình huống | Behavior hiện tại |
|---|---|
| User bấm hotkey VI/EN trong app A | Lưu true/false cho bundle ID A, tồn tại qua restart |
| Quay lại app có entry | `activateServer → applyPerAppDefault → resolveVietnamese` khôi phục entry |
| App chưa có entry | Trả nil/UNKNOWN; giữ trạng thái VI/EN toàn cục đang có, không tự mặc định VI |
| Chọn VI/EN từ menu/Settings Funput | `setPending`; lựa chọn bind vào app tiếp theo resolve, pending chỉ session |
| `excludedApps` cũ | Migrate một lần thành map false (EN), xóa key cũ; entry mới đã tồn tại thắng |
| Import config | Merge app chưa có entry, giữ lựa chọn local nếu đã có; không ghi đè mọi rule |
| Manual toggle VI trong app từng EN | Đổi pin sang true; không có hard lock “app này tuyệt đối bỏ qua” |

Sources: [Swift memory](../../xfunput/platforms/macos/Funput/Model/AppLanguageMemory.swift), [controller focus/toggle](../../xfunput/platforms/macos/Funput/IME/FunputInputController+Composition.swift), [FFI focus](../../xfunput/crates/funput-ffi/src/app_language/focus.rs), [Mac tests](../../xfunput/platforms/macos/FunputTests/AppLanguageMemoryTests.swift). Focus lấy `NSWorkspace.frontmostApplication.bundleIdentifier`; không phải domain/tab/window. App khác app name nhưng cùng bundle ID dùng cùng policy. Khi popup lấy focus, không mặc định frontmost là app user định tác động.

Comment tại activateServer còn mô tả exclusion list/default VI cũ; **source resolve thực tế mới là truth**. Không thấy UI quản lý hard blacklist/list map hoặc switch tắt memory trên Mac trong các panes đã đọc; không suy từ Windows `appLanguageMemoryEnabled` sang Mac. Rust FFI có forget/clear nhưng Swift memory/UI chưa expose thành chức năng quản lý danh sách.

## 2. EN không phải “Funput hoàn toàn không can thiệp”

[Controller](../../xfunput/platforms/macos/Funput/IME/FunputInputController.swift) vẫn xử lý toggle trước khi check VI enabled. Khi EN, nó gọi [handleEnglish](../../xfunput/platforms/macos/Funput/IME/FunputInputController+English.swift). Nếu có bảng gõ tắt hợp lệ và `shortcutsEnabled && shortcutsInEnglish`, engine vẫn giữ raw word/composition rồi bung macro; hai switches này mặc định true trong [registeredDefaults](../../xfunput/platforms/macos/Funput/Model/AppSettingsKeys.swift).

Probe actual Rust engine: VI disabled + macro `vn → Việt Nam`, gõ `vn ` vẫn ra `Việt Nam `. Tắt English macros thì giữ `vn `. 10 tests của actual FFI app_language module chạy đạt bằng isolated rustc harness; không là full Cargo/XCTest/GUI pass. Chưa input injection vào app thật.

Hệ quả: chọn EN để né lỗi direct composition chưa đủ nếu English macros đang hoạt động. Các app terminal/editor/game nhạy cảm có thể cần **hard bypass**, không chỉ pin EN.

## 3. Policy nên thêm cho VXKey

Ba khái niệm độc lập, không dùng chung boolean:

| Policy | Ý nghĩa |
|---|---|
| Nhớ VI/EN theo app | Tiện ích ghi nhớ lựa chọn; user vẫn đổi được |
| Không xử lý phím trong app | Hard bypass: không dấu/macro/retone/autocap/direct rewrite; trả input về app |
| Không ghi clipboard từ app | Capture filter riêng, không ảnh hưởng khả năng gõ tiếng Việt |

Hard input bypass ưu tiên hơn per-app memory/pending VI/EN; toggle VI không âm thầm gỡ rule hard bypass. Match exact bundle ID (có picker/app path để user xác nhận), không substring app name. Có per-app xóa rule/forget memory và explanation. Secure fields giữ OS protections, không ép event tap qua secure input.

Check bypass tại entry point trước sync/engine/composition của client mới. Khi chuyển app phải commit/clear đúng client cũ và reset session, không commit text cũ vào client bị loại trừ. IME/companion/launcher của VXKey phải được nhận diện như internal surfaces: mở Clipboard/Search/Settings không tạo pin hoặc consume pending sai cho chính helper. Snapshot app trước mở panel nếu thao tác cần quay lại, không dùng frontmost sau panel như original target.

Clipboard blocklist không tự được lấy từ hard input list: user có thể muốn copy code từ app EN nhưng không muốn lưu mật khẩu; cũng có thể muốn gõ VI trong app bị chặn history. UI có thể cùng trang “Ứng dụng” với hai cột độc lập và không auto tick thay user.

## 4. Funput có Clipboard History chưa?

**Chưa.** Mac chỉ có read/write clipboard thủ công trong [ConvertPlatform](../../xfunput/platforms/macos/Funput/Convert/Platform/ConvertPlatform.swift): đọc `.string`, clear/setString cho công cụ chuyển mã. Không có monitor/changeCount/history DB/search/hotkey toàn hệ thống cho history. Các clipboard-related UI này không phải nền history đã hoàn thành.

Repo [vxclipboard](../../vxclipboard) hiện chỉ có icon assets, chưa có engine có thể tái sử dụng. Backlog cũ Tauri/arboard là ý tưởng, không code; user yêu cầu tích hợp vào VXKey thay vì tự mở thêm app độc lập. Có thể giữ tên VXClipboard cho backlog riêng sau này, chưa xóa/rename repo/assets.

Mac 26 đã có clipboard history qua Spotlight. VXKey chỉ đáng thêm nếu đem lại hotkey/panel chung, retention/search/privacy control phù hợp nhu cầu; không cần reverse-engineer database của Spotlight hoặc import lịch sử hệ thống. [Apple hướng dẫn clipboard history](https://support.apple.com/en-ie/guide/mac-help/mchl40d5b86b/26/mac/26).

## 5. Scope v1 khuyến nghị: text-only, copy-back trước

### Nhận vào

- Unicode plain text, code, URL dưới dạng text và multiline; giữ nguyên whitespace/newlines/nội dung khi copy lại. Bỏ formatting HTML/RTF chỉ khi source cung cấp plain-text representation hợp lệ; không render HTML, không fetch URL/preview từ mạng.
- Theo dõi **copy events mới sau khi user bật** tính năng; không lấy clipboard có sẵn lúc startup/enable làm entry đầu hoặc backfill khi resume.
- Popup từ tray/hotkey global tùy chỉnh; keyboard search/select, preview rút gọn, copy lại nguyên nội dung, delete một mục/clear history, pause/resume. Chưa cần pin/tag/AI/translation.
- Đề xuất limits ban đầu: 200 entries, 7 ngày, tối đa 128 KiB UTF-8/item và 10 MiB tổng plaintext; đạt giới hạn thì prune oldest, không tự cắt nội dung rồi lưu như đủ. Đây là defaults đề xuất, chưa user chốt.
- Local-only; có thể xem lịch sử sau restart nếu bật persistence. Persistent entries mã hóa bằng CryptoKit/Keychain, không để text/preview/search-index plaintext trong SQLite/log. Với chỉ 200 mục, search memory khi unlock tránh thêm encrypted full-text-search infrastructure. Keychain không sẵn sàng thì pause hoặc session-only theo UI rõ ràng, không fallback lưu plaintext.
- Clipboard capture mặc định chưa bật cho tới consent/onboarding. Tính năng lỗi/tắt/không có quyền không làm bộ gõ ngừng hoạt động.

### Chưa nhận vào v1

- Ảnh/screenshot/thumbnail/OCR, HTML/RTF round-trip, files/folders/security-scoped bookmarks, cloud sync, import history Spotlight/clipboard manager khác, snippets learning hoặc đọc typed text.
- File URL/Finder file copy không biến thành text-history path chỉ vì event có `.string`. Multi-item image/file event bị skip ở v1; plain URL text không là file copy. Cần classify types trước lấy payload.
- Không cam kết auto paste Enter vào app khác ở v1 nhỏ nhất. V1 Enter/click copy lại, user tự Cmd+V; auto paste là nâng cấp riêng sau focus/permission spike. Không dùng IMK insert để paste vào app bất kỳ hoặc gọi clipboard macro như typing engine.

## 6. Worker riêng, cùng sản phẩm VXKey

Khuyến nghị **một sản phẩm/UX VXKey, hai phần native**:

```text
Input method → Swift composer → Rust core/engine
                    (không clipboard polling / DB / encryption / search)

Native companion (có thể phát triển từ launcher stub)
  NSPasteboard adapter → capture policy → encrypted bounded store
                                      → native history/search panel
```

Companion sống độc lập input source, nên history/hotkey vẫn dùng khi chuyển ABC, tắt VI hoặc hard-bypass một app. Không dựa vào IMK activateServer/handleKey để poll clipboard hoặc bảo đảm process luôn sống. Một process native IME hiện tại host cả UI không có nghĩa clipboard always-on nên được thêm vào main thread đó.

Companion chịu consent/pasteboard permission/tray/hotkey/history; IME chỉ đọc settings cần thiết qua IPC/config đã xác định. Identity/team/permissions riêng nhưng cùng product, một tray owner để không hai icon VI/EN. IPC typed/limited và authenticates cùng app/user khi cần; không gửi raw keystrokes/conversation/clipboard body sang engine để gõ. Companion crash/update không kéo IME chết; journal/prune/search không giữ lock IME. Cần xác định migration launch-at-login và updater/installer cho cả hai cùng version.

Phương án rẻ hơn là module background queue ngay trong process IME: ít code packaging nhưng cùng failure domain và phụ thuộc vòng đời input method. Chỉ dùng cho spike, không mặc định daily-use always-on. Phương án app VXClipboard riêng tạo UX/scope mới user chưa yêu cầu; không chọn cho lượt tích hợp này.

## 7. Privacy và correctness gates

1. **Quyền clipboard thật:** SDK AppKit 27.0 trên máy khai `accessBehavior` từ macOS 15.4: default/ask/alwaysAllow/alwaysDeny, property read-only. User kiểm soát per-app trong System Settings. Onboarding phải hỗ trợ denied/revoked/ask, không poll gây prompt loop hoặc sửa OS defaults để bypass. Availability API không chứng minh mọi OS/build đã bật cùng cơ chế; cần test Mac 26.7 thật. [Apple accessBehavior](https://developer.apple.com/documentation/appkit/nspasteboard/accessbehavior-9k4t4).
2. **Không xin quyền thừa:** copy-back/history không cần event injection; global shortcut dùng registration API phù hợp và báo conflict. Không ép Input Monitoring/FDA/admin cho clipboard text. Auto paste cần đánh giá quyền post event/Accessibility riêng, không lấy quyền IME làm đủ. Chưa xin/cấp quyền trong audit.
3. **Filter trước payload/persistence:** skip các marker `org.nspasteboard.ConcealedType`, `TransientType`, `AutoGeneratedType` và known sensitive formats đã validation. Không hash/index/preview nội dung đã bị chặn. Đây là convention, không đảm bảo mọi producer dùng đúng. [NSPasteboard markers](https://nspasteboard.org/).
4. **Source app không xác thực chắc:** frontmost lúc poll có thể khác app đã copy. `org.nspasteboard.source` do producer tự khai và có thể rỗng, chỉ là hint. Nếu marker source hoặc observed app nằm blocklist thì chặn; source mơ hồ/race không gán certainty. Unknown/no source mặc định skip theo chế độ bảo thủ; normal estimated source phải có quality metadata và không hứa blocklist tuyệt đối. Không loại theo app display name. Password manager có marker cũng phải thử từng app/version; secret copy từ browser/Terminal/Notes dạng text thường vẫn có thể lọt, không có bộ lọc tự nhận mọi mật khẩu.
5. **Stable snapshot/polling:** `changeCount != lastCount`, tick khoảng 500 ms, snapshot count/types trước đọc và count sau; đổi trong lúc đọc thì discard/retry bounded. Không serialize promised data nặng trên IME main. Clipboard chỉ có current snapshot, polling có thể bỏ lỡ các copy quá nhanh, không hứa ghi mọi Ctrl/Cmd+C. Sleep/wake/lock/resume baseline không đọc lại copy trong lúc pause/locked.
6. **Own writes:** copy-back ghi self marker và theo dõi expected change count để không nhân entry; không suppression theo content hash hoặc timeout toàn cục làm bỏ copy mới của user. Chỉ commit trạng thái success khi write thành công; không auto restore clipboard sau delay vì có thể đè bản copy mới. Có thể dùng currentHostOnly cho nội dung lấy từ history để không tự phát ra Universal Clipboard. [Apple currentHostOnly](https://developer.apple.com/documentation/appkit/nspasteboard/contentsoptions/currenthostonly?language=objc).
7. **Không cloud ngoài ý muốn:** app không sync history; marker Handoff/remote clipboard có thể skip mặc định v1. History riêng không kiểm soát việc OS/app gốc đồng bộ clipboard hiện tại; không tuyên bố tắt Handoff toàn máy. Không private SPI của Spotlight để chặn history OS.
8. **Data lifecycle:** opt-in, pause, clear, limits/TTL, hide/redact khi lock, close panel/clear decrypted search cache, không export history kèm settings. Entry delete/clear áp store và index/cache/WAL liên quan; không hứa secure erase khỏi SSD/backups hoặc bảo vệ khỏi malware cùng user. Mã hóa at-rest không thay bộ lọc capture. DB/key mất/lỗi không crash IME hoặc regenerate key rồi báo history rỗng như bình thường.

Giới hạn byte của entry không tự giới hạn chi phí materialize dữ liệu clipboard: provider có thể trả text rất lớn/chậm trước khi app đo được độ dài. Cần spike lazy/provider behavior, đọc trên worker phù hợp AppKit và isolation/timeout/recovery của helper; không chặn IME main thread để chờ. Dedup chỉ sau privacy filter, dùng memory comparison hoặc keyed digest nếu lưu, tránh plaintext preview/index hoặc hash bí mật dùng như proof an toàn.

Các gates này là scope cần thiết, không checklist xin approval toàn máy. Mục tiêu là không biến clipboard feature thành nguồn lưu secrets âm thầm hoặc đường làm lag typing.

## 8. Effort và lựa chọn cắt scope

So sánh effort là tương đối, chưa estimate ngày vì chưa spike permission/helper packaging. Phần đáng kể là lifecycle/privacy/focus, không phải câu lệnh insert text vào DB.

| Gói | Effort tương đối | Giá trị/đánh đổi |
|---|---|---|
| A. Text session-only + search/copy-back | Nhỏ nhất nhưng vẫn có monitor/filter/UI/hotkey/helper | Dùng nhanh, không giữ qua restart; không né permission/privacy |
| B. Text + bounded encrypted persistence + copy-back | **Vừa, khuyến nghị v1 hoàn chỉnh** | Thực dụng hơn, thêm Keychain/TTL/migration/deletion tests; vẫn không auto paste |
| C. B + auto paste vào app cũ | Vừa bổ sung, phụ thuộc runtime spike | Thêm post-event permission/focus race/wrong-target prevention; permission-denied fallback copy-only |
| D. Thêm ảnh | Lớn hơn text đáng kể | Decode/thumbnail quota, memory/disk, multi-representation/privacy, clipboard formats và restore fidelity |
| E. Thêm files/folders | Lớn và scope khác | URL/bookmark/permission/rename/missing-file/multi-item semantics; giữ path không là giữ file |

Đề xuất bắt đầu B, có thể dùng A làm milestone nội bộ. C là tùy chọn sau spike, D/E backlog; text-only không cần Tauri/arboard nếu shell Mac đã native. Không gộp clipboard rollout vào việc thay toàn bộ UI/engine. Nếu mục tiêu chỉ vài bản copy trong ngày, OS Spotlight đã có lựa chọn không phải viết code; user đang yêu cầu integration nên draft vẫn hướng native module VXKey.

## 9. Acceptance và task bổ sung

- **App policy:** hard bypass không engine/macro/retone/direct writes; soft EN macro behavior rõ; migration excludedApps không giả hard rule; manual pin/unknown app/pending/helper focus/restart/import đều có tests. Rà comment/UI đang mô tả list cũ.
- **Capture privacy:** fixtures cho concealed/transient/autoGenerated/file/image/multi-item/unknown/excluded/remote, empty/oversized text và rapid change; blocked content không nằm DB/index/log/UI. Không test bằng mật khẩu user thật.
- **History:** exact UTF-8/multiline/emoji/whitespace round-trip, dedup đúng sau filter, quota/TTL, corrupt DB/key unavailable, delete/clear/restart, lock/pause/resume không backfill.
- **Integration:** history hoạt động khi ABC/VI disabled/bypassed app; helper crash không phá composition; panel không pin sai VI/EN hoặc giữ pending sai; một hotkey/tray/listener, CPU idle đo trên artifact.
- **Permissions:** ask/deny/revoke cho đúng helper identity, no permission spam; copy-only không injection; auto paste nếu có phải đúng PID/focus và skip khi không chắc, không paste vào field đã đổi/người dùng chưa chọn.
- **Packaged Mac:** launcher/companion/IME identities, signing/version/installation/update/uninstall/rollback đủ; chưa build/XCTest/TCC/live pasteboard QA trong audit này.

Lượt này chỉ source tests và scope. Không đọc clipboard hiện tại, không chạy monitor, không đăng ký hotkey hoặc thay memory/preferences/input sources. Chưa giao Anti triển khai; Windows vẫn backlog. Bản work to do cần thêm hard bypass và Clipboard milestone sau baseline/composition safety, phối hợp identity/companion trước installer.
