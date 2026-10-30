> Trạng thái triển khai 2026-10-30: đã bootstrap fork và build native candidate isolated; chưa cài/kích hoạt/release. Xem [report hiện tại](docs/qa/engine_implementation_report.md) và [bàn giao UI](docs/handoff/ui_handoff.md). Checklist lịch sử bên dưới chưa được dùng làm tracker triển khai.

# VXKey Mac — Work to do sơ bộ

> Cập nhật 2026-10-30: [Bộ spec hiện hành](docs/index.md) thay các đề xuất UI/kiến trúc trong tài liệu lịch sử này. User chọn Tauri UI chung Mac/Windows, Mac vẫn giữ InputMethodKit + Rust core; xem [kế hoạch](docs/spec/07_implementation_contract.md). Chưa specs ready/build/cài/release.

Ngày: 2026-10-27. Draft dựa trên [audit](audit_2026_10_27.md), không là backlog đã triển khai. **macOS trước; Windows backlog theo chỉ đạo user.** Source [xfunput](../../xfunput), đích [vxkey](../../vxkey) còn trống. Mục tiêu đầu là bản dogfood gõ ổn định và có đường rollback, chưa phát hành công khai.

Bổ sung user yêu cầu: kiểm tra cơ chế loại trừ app và tích hợp clipboard history; [audit/scope mới](app_exclusion_clipboard_scope.md). Funput hiện nhớ VI/EN, chưa hard bypass và chưa history. Text-only persistent/copy-back là đề xuất, ảnh/file/auto paste chờ lựa chọn hoặc spike; chưa chốt tất cả scope này như yêu cầu đã duyệt.

## Scope v1 đề xuất

- Rust core/engine/C ABI hiện có + native InputMethodKit/SwiftUI Mac. Giữ tên nội bộ Funput giai đoạn đầu để diff với upstream dễ; product identity/public UI/storage riêng VXKey.
- Telex/VNI, VI/EN toggle, tone style, settings/tray tối thiểu, per-user installation và disable/uninstall có rollback. Lấy typing parity trước redesign UI.
- Không telemetry/typed-content logging, không tự learn/persist từ đã gõ, không updater gọi Funput, không đè input source/settings của Funput gốc.
- Conservative mode cho app/client chưa đáng tin; không coi bỏ gạch chân ở mọi app quan trọng hơn bảo toàn văn bản.
- Windows/Linux/mobile, Tauri Settings rewrite, AI/speech/learning, lower Mac floor và Store nằm ngoài lượt Mac đầu. Feature có sẵn giữ source/tests, không tự bật thêm phạm vi product.

## P0 — Trước khi dùng VXKey làm bộ gõ chính

### K01 — Bootstrap fork có provenance

- [ ] Clone/preserve history từ Funput vào VXKey tại commit `2940d6aa04a9d90ffe24327732d843784be1adaf`; không dùng bản source trôi nổi hoặc copy Git metadata thủ công.
- [ ] Ghi upstream URL/commit/date/license và state sạch, giữ upstream remote; origin VXKey đúng repo đích khi user cung cấp. Không tự tạo/push remote hoặc publish.
- [ ] Giữ copyright/MIT, assets attribution; docs riêng VXKey, docs upstream lưu phân biệt. Chỉ tạo file mới snake_case; chưa bulk rename cấu trúc Xcode/crate upstream.
- [ ] Giữ xfunput sạch và không sửa trực tiếp. Tắt updater của fork ngay trong bản có thể chạy cho tới khi feed/key VXKey sẵn sàng.

**Nghiệm thu:** repo VXKey truy nguyên upstream, tests source baseline chưa đổi logic, không network/updater identity Funput trong bản fork có thể cài.

### K02 — Khóa build/test baseline Mac

- [ ] Có Rust pin 1.98.1/dependencies đầy đủ hoặc chủ động chọn pin mới kèm baseline so sánh; không dùng smoke 1.99 làm proof pinned pass.
- [ ] Chọn full Xcode local hoặc macOS CI, record Xcode/SDK/Rust/Sparkle versions và resolved dependencies. Không đổi global toolchain/cài Xcode hộ user từ task audit.
- [ ] Rerun Cargo core/engine/config/FFI cùng feature convert và tests liên quan; Mac XCTest 17 files, FFI ABI/cbindgen; frontend native không phải npm build.
- [ ] Build native app chưa install; baseline functionality/hash/version và expected platform. Add Mac build/XCTest PR check, không đợi tới release.

**Nghiệm thu:** gói đúng source reproducible, tests đủ dependency chạy có kết quả, original typing behavior được ghi nhận. Nếu build trên CI, tải artifact kiểm chứng local; full Xcode không bắt buộc chỉ để dùng/test artifact.

### K03 — Composition safety và app compatibility spike

- [ ] Chuyển probe false selection thành regression: prefix/suffix không chọn phải giữ nguyên qua rewrite, backspace, focus switch, stale caret/selection, inline autocomplete, noncollapsed selection và UTF-16/emoji.
- [ ] Quyết định guard/fallback cho direct composition; app-level trust không cấp quyền cho field trả selection sai. Với trạng thái không chắc, không rewrite rộng; hỗ trợ marked-text/preedit và per-app safe mode nếu cần.
- [ ] Test TextEdit/Notes/Safari/Chrome/VS Code/Cursor/terminal trên đúng app/OS version, source-switch khi field đang focus, stall/hang và fallback. Tái hiện giới hạn upstream thay vì mặc định đã được Apple/app sửa.
- [ ] Secure/password fields giữ OS protections; macro/restore/retone không can thiệp sai, composition clear khi focus đổi. Không thu thập nội dung gõ trong diagnostic.
- [ ] Tách soft VI/EN memory khỏi hard per-app bypass. Bypass phải ngừng cả English macros/retone/direct composition; không migrate map EN cũ thành hard rule âm thầm. App picker/rule remove/forget memory; helper/history popup không pin nhầm VI/EN vào internal bundle.
- [ ] Đo latency/hot-path và idle memory/CPU trên native artifact; không gọi benchmark core là độ trễ gõ toàn hệ thống.

**Nghiệm thu:** không mất chữ ngoài mục user đang sửa trong matrix bắt buộc; false-selection fixture không còn xóa sentinel; app chưa support có fallback/known limitation rõ. Chưa đạt thì chưa daily-driver/release.

### K04 — Identity, cài/gỡ và rollback độc lập

- [ ] Chốt namespace VXKey riêng cho bundle/input-source/mode/IMK connection/launcher/URL scheme/UserDefaults/pkg receipt/team/version. Không chốt một bundle ID ngẫu nhiên thành immutable release identity khi chưa đối chiếu convention VX.
- [ ] Rewrite install/uninstall/postinstall/launcher theo ownership chính xác; không kill/delete app/preferences Funput. Không search-replace MIT/third-party credits hoặc Rust ABI symbols không cần thiết.
- [ ] Per-user staging → verify → atomic replace/backup, Add Input Source hướng dẫn rõ; giữ ABC/Funput để rollback. Không tự chuyển nguồn nhập của user.
- [ ] Uninstall disable/remove source trước, giữ preferences tùy chọn, chỉ xóa VXKey-owned paths; failure/interrupt không làm mất cả bản cũ lẫn mới. Logout/relogin requirement và launcher discovery được nghiệm thu.
- [ ] Login item/status có error thật, không chỉ checkbox; cơ chế auto-start phải theo IME lifecycle, không nhân đôi launcher/IME process.

**Nghiệm thu:** VXKey và Funput cùng cài mà không collision; bỏ VXKey xong Funput/ABC/settings không đổi; cài lại/update có rollback fixture.

## P1 — Sau baseline gõ ổn định

### K05 — Nhận diện VX và UX thiết yếu

- [ ] Product name/icon/Settings/onboarding/about/tray/support links VXKey, dùng [icon nguồn VX](../zicons/ivxkey_icon_trans.png) nếu user giữ lựa chọn này; copyright/credits vẫn giữ.
- [ ] UI native theo theme VX trong khả năng SwiftUI; tray thể hiện VI/EN, toggle/hotkeys không xung đột macOS và app thường dùng.
- [ ] Explain direct/preedit, English restore, retone, spell check/tone style; không lẫn macro với system hotkey. Defaults dựa trên matrix, không mặc định copy mọi true từ upstream.
- [ ] Config export/import version/namespace riêng; import Funput chỉ opt-in có preview/merge và không sửa domain gốc. Macro có dữ liệu người dùng, không overwrite/import âm thầm hoặc sync/cloud tự động.
- [ ] Nếu giảm Mac min OS: task riêng với availability audit/UI fallback/runtime matrix, không chỉ sửa deployment target.

**Nghiệm thu:** Settings đúng trạng thái engine, invalid import giữ cấu hình cũ, preview/confirmation rõ; không có link updater/donation/brand Funput nhầm thành VXKey nhưng notices vẫn nguyên.

### K06 — Dogfood, privacy và build đóng gói

- [ ] Mac 26+ arm64 trước theo đề xuất; user xác nhận target minimum/Intel sau khi thấy compatibility cost. Không claim Intel chỉ vì lipo tạo universal file.
- [ ] Test native package sau download/install thật trên fixture: typing/backspace/undo/gõ tắt nếu bật, source-switch/sleep/wake/multi-window/onboarding/menu/launcher/disable/uninstall/reinstall.
- [ ] Network trace startup/typing/settings: không text/telemetry egress; update chỉ explicit khi được enable. Check log/panic/report không typed text, không đọc lịch sử/clipboard ngầm.
- [ ] Third-party inventory theo artifact: MIT Funput, Sparkle/deps/assets, dictionary notice chỉ khi ship; license screen/bundled notices có thể đọc offline.
- [ ] Ghi run report: source SHA/commit, app version/arch/OS/app versions, test results, artifact hash, known limits, rollback steps.

**Nghiệm thu:** gói Mac đã QA daily-use matrix, còn limitation công khai; không dùng test pass để suy mọi app an toàn. User dogfood có đường tắt quay ABC khi cần.

### K07 — Chuẩn bị phát hành, khi user giao bước release

- [ ] Chọn per-user zip hoặc signed/notarized distribution, plan Developer ID/notary ownership và chi phí nếu cần; ad-hoc/internal build không gọi là notarized.
- [ ] Nếu bật updater: repository/feed **và** public/private signing ownership riêng VXKey; test invalid signature/old version/offline/partial download/rollback/logout requirement. Không reuse private key hoặc Team ID Funput.
- [ ] CI build/test đúng source + platform và packaging; tách publish/manual gate theo quyền user, không copy workflows deploy 5 nền tảng/Store/sponsors của upstream.
- [ ] User-facing notes/support/known issues, version/manifest/asset naming nhất quán bộ VX.

**Nghiệm thu:** release có evidence và quyền phát hành cụ thể; chưa có signing/updater thì báo chưa có, không làm key giả hoặc publish để thử.

## P1 — Clipboard history (bổ sung)

### K08 — Clipboard history tích hợp VXKey Mac (scope đề xuất)

- [ ] Chốt text session-only hay encrypted persistence; khuyến nghị 200 entries/7 ngày, 128 KiB/item và 10 MiB tổng, search/copy-back/delete/clear/pause, chưa auto paste/ảnh/file/cloud/learning.
- [ ] Native companion/worker tách IME hot path, history vẫn hoạt động khi dùng ABC/VI off; một tray owner, signing/IDs/config IPC và lifetime riêng. Phối hợp K04 trước đóng gói, không chỉ chạy Timer trong handleKey.
- [ ] Clipboard consent/permission ask-deny-revoke và hotkey conflict; block markers/app policy trước content lưu, source provenance chỉ là hint, không ghi clipboard có sẵn lúc enable/resume hoặc nội dung trong pause/lock.
- [ ] Exact UTF-8/multiline/emoji round-trip, stable changeCount snapshot, race/dedup/self-write, quota/TTL/corrupt store/Keychain unavailable/deletion; session-only fallback phải user biết, không lưu plaintext âm thầm.
- [ ] Nếu persistent: mã hóa record, không plaintext preview/FTS/log; hạn chế decrypted memory. Không nhập history vào settings export hoặc gửi vào Rust composition engine.
- [ ] Optional auto paste là gói riêng: permission/focus target/secure input/safety và copy-only fallback; không dùng IMK inject vào app bất kỳ. Images/files backlog.

**Nghiệm thu:** fixtures privacy không rò DB/index/UI/log, helper lỗi không làm lag/chết IME, copy-back không loop/đè copy mới, native permissions đúng identity; typing parity K03 vẫn giữ. Chi tiết [scope clipboard](app_exclusion_clipboard_scope.md). Đưa K08 trước K06/K07 nếu nhận vào v1; đây là milestone đề xuất, chưa implementation.

## P2 — Windows backlog (giữ nguyên)

- [ ] Xác minh hook baseline vs TSF; chưa tự hứa có TSF.
- [ ] Fix/prove SendInput zero/partial/UIPI handling và state recovery; maxlength/autocomplete/selection/emoji/AltGr/elevated/game matrix. Không giải quyết bằng run-as-admin mặc định.
- [ ] Namespace Win/update key/feed/autostart/multi-process/settings riêng; full Windows runtime tests.
- [ ] Slint attribution/license/binary dependencies; quyết định có giữ native Slint hay UI shared sau khi Mac ổn.

## Thứ tự đề xuất và quyết định còn mở

`K01 → K02 → K03 → K04 → K05 → K08 (nếu nhận clipboard vào v1) → K06 → K07`; design/spike K08 có thể làm sớm nhưng identity/companion K04 cần cập nhật trước cài. K03 có thể làm song song review source với K02 nhưng không hai agent sửa cùng file. Không làm rebrand/UI trước baseline rồi mất cách so sánh bug upstream/fork. Task chưa có estimate ngày vì chưa có native build/typing matrix; có thể chia gói nhỏ có acceptance như trên để giao Anti và Codex review khi user yêu cầu triển khai.

User đã chốt Mac trước, Windows backlog. Còn mở: minimum Mac OS, Intel ngay hay sau, các app phải ưu tiên (tạm lấy app Mac trong audit), UI redesign mức nào, bản cá nhân hay public distribution/signing/updater. Không cần chờ các quyết định này để audit hoặc bootstrap baseline, nhưng phải chốt trước claim hỗ trợ/release rộng.
