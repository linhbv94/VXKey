# VXKey — lập lại kế hoạch adapter macOS

Version: 0.6.0 — Ngày cập nhật: 2026-11-04.

Trạng thái: **P1/basicP2 native PASS trên spike; candidate tích hợp đã native PASS corpus/macro/TextEdit boundary, exclusion và Pause/Resume/Quit; P2 matrix còn mở. Startup false permission latch tái hiện sau reopen, source/fixture đã sửa nhưng chưa bundle.** [Integration report](../qa/session_adapter_integration.md) là điểm tiếp mới; [pilot report](../qa/session_spike_review.md) giữ evidence v6. Default build/contract production chưa đổi; chưa full parity/daily-use acceptance/specs ready. Track dài hạn MỞ; Windows PLAN ONLY.

## Quyết định sau pilot — MAC_ADAPTER_002

**Chọn `session tap + Funput local + return_original` làm hướng triển khai tiếp theo.** Đây là quyết định kỹ thuật có bằng chứng native để tiếp tục, không phải promote prototype hoặc quyền release. User yêu cầu cập nhật docs khi hướng có triển vọng.

Phần đã thay nằm ở adapter macOS. Rust Funput/C ABI và Tauri/React giữ nguyên. Với một phím cần sửa dấu, adapter chuẩn bị đủ các Backspace hữu hạn, đặt Unicode output lên chính event gốc và trả event ấy về luồng; giữ keycode/flags và cho keyUp vật lý đi qua. Phím không cần chuyển đổi được chuyển nguyên trạng. AX đọc caret/text không nằm trong điều kiện bắt buộc của đường gõ thường; quyền Trợ năng OS vẫn là vấn đề riêng cần kiểm tra.

Luồng mục tiêu:

```text
Phím vật lý → session tap → cache policy/config → Funput local
  → không sửa: trả event nguyên trạng
  → cần sửa: chuẩn bị Backspace batch → đặt Unicode lên event gốc
             → post Backspace qua proxy → trả event gốc → keyUp vật lý
```

Bằng chứng để chọn: v5 trả đúng kết quả core và có packets đúng tới annotated tail nhưng editor vẫn mất á/à; v6 đổi delivery thì user xác nhận TextEdit/Figma và các ca boundary/sentinel đã đúng. Private event source riêng lẻ không sửa được case. ReverseA/B trên cùng v6 và cơ chế nội bộ macOS/client gây mất chữ chưa được xác minh; không biến kết quả này thành kết luận lỗi toàn bộ core hoặc bảo đảm mọi app.

### Phạm vi bằng chứng đã đạt

| Phần | Đã kiểm chứng | Chưa suy ra |
|---|---|---|
| Gõ thật | Corpus Telex trên TextEdit/Figma canvas nháp; `casch/cachs`, `ddwojwc`, `w` trong corpus | Mọi app/layout/tốc độ hoặc tất cả tính năng |
| Boundary | Enter, Shift+Enter, Cmd+A→xóa→gõ ngay trên TextEdit | Mọi thay đổi caret/text do app thực hiện bằng code |
| Phạm vi sửa | Prefix/emoji/suffix và mouse selection trong TextEdit giữ nguyên | Document ownership tuyệt đối/transaction atomic |
| Quyền | Exact spike chạy/gõ trên máy QA với AX=true, post/listen preflight=false | Mọi macOS, updater retention hoặc UI quyền production đã sửa |
| Lifecycle | Harness một process đã Pause/Quit, hook tháo, MKey mở lại | Helper/companion production chịu UI stall, disconnect, update, sleep/wake |

### Hàng đợi triển khai từ quyết định này

| ID / phase | Đầu ra cần làm | Gate hoàn thành |
|---|---|---|
| MAC_ADAPT_01 / P2 | Đủ xử lý ACTION_RESTORE/English restore và chiến lược output dài hơn20UTF16; giữ macro/retone/VNI/TelexAdvanced/hotkey trong phạm vi parity | Core oracle + native output đúng, không cắt output hoặc sửa lấn sentinel; không dùng giới hạn P1 để âm thầm bỏ tính năng |
| MAC_ADAPT_02 / P2 | Mở rộng ma trận uppercase/layout, autocomplete, undo/redo, focus nhanh, editor/terminal và secure-input/password | Ghi riêng client/case/giới hạn; một lỗi sửa lấn vùng nháp là blocker |
| MAC_ADAPT_03 / P3 | Thiết kế và kiểm owner helper, config last-known-good đã xác thực; phân biệt user_pause/permission/engine/tap/control_disconnect | UI/bridge chậm không tự làm ngừng gõ; Pause/fault không tự Resume; Quit/update không để orphan hook; raw keyboard dùng được khi bypass |
| MAC_ADAPT_04 / P4 | Nối adapter mới vào app thật, cache config/policy/language memory đã validate, giữ hard bypass trước engine/macro; thống nhất readiness và luồng quyền UI | App exclusion và setting hiện có hoạt động thật; helper/companion đúng identity; không giả active khi status stale; đồng bộ spec00/02/04/05/07/08/09/10 và command/status schema |
| MAC_ADAPT_05 / P5 | Candidate test tích hợp có danh tính ổn định, canary dài ngày và regression | User acceptance đúng scope; signing/updater/retention theo track release riêng trước phát hành |

Các mục trên là backlog được xác định, chưa đánh dấu đã triển khai. P1cap20UTF16, thiếu ACTION_RESTORE và thiếu config/policy trong harness là nợ phải giải quyết trước app thật. Không chép harness thành default để mất per-app exclusion hoặc capability. Giữ v6 đã cấp quyền nguyên bytes/path làm baseline; mọi binary tiếp theo dùng candidate riêng và kiểm identity theo runbook. Windows vẫn chỉ lập plan.

User đồng ý hướng sau phần giải thích đánh đổi: ưu tiên bộ gõ hoạt động ổn định bằng xử lý event tuần tự, sửa hữu hạn và reset đúng boundary; không bắt mọi editor xác minh caret/text từng phím. AX snapshot cũng không khóa editor hoặc bảo đảm atomic giữa đọc và sửa. Adapter thử nằm riêng trong `native/macos_session_spike`; chưa thay driver hiện tại. Gate native vẫn bắt buộc, không tiếp tục vá timeout/whitelist như hướng chính.

## 1. Kết luận và phạm vi giữ lại

Hướng tiếp tục sau pilot: session event tap → composer Funput cục bộ → Backspace hữu hạn + Unicode trên event gốc theo cùng stream. AX đọc text không là điều kiện bắt buộc cho Telex thông thường. Tách khả năng gõ khỏi lease kết nối companion; không đưa phím qua React hoặc Rust companion.

Giữ Rust Funput/C ABI, bản sửa tone-order có regression, Tauri/React UI chung, config, VI/EN, hard bypass, clipboard và signing/updater hiện có. Không đổi sang engine MKey, không viết lại toàn app, không cài Input Source hoặc thay ABC. Macro, retone, VNI, double-space và các chế độ khác vẫn thuộc scope sản phẩm; mỗi tính năng phải được kiểm lại trên adapter mới, không tự coi có parity vì dùng cùng core.

**Spike native đã được thực hiện có gate và trace**, không chỉ xóa `ContextGuard` khỏi driver hiện tại. Session tap đơn thuần vẫn lỗi: v2/v3 post Unicode pair mất á/à. V6 trả Unicode trên event phím gốc, chỉ post Backspace, đã qua corpus TextEdit/Figma và boundary/sentinel cơ bản. Không kết luận mọi mẫu lỗi cùng nguyên nhân hoặc mọi editor được bảo đảm. Bước tiếp P2 còn lại/P3/P4; giữ contract production hiện tại tới khi tích hợp có bằng chứng.

## Checkpoint triển khai 2026-11-04

[Module tích hợp và evidence](../qa/session_adapter_integration.md): flag `VXKEY_SESSION_ADAPTER=1` thêm driver/composition/delivery/run policy mới, nối config/policy thật, peer role `engine_stream`, owner watchdog và readiness AX thống nhất UI/backend. MAC_ADAPT_01 có fixture96checks gồm restore/long output/VNI/retone/flip/macro VI/EN; MAC_ADAPT_03 có bridge2process thật với hook mock kiểm disconnect/reconnect/Pause; MAC_ADAPT_04 có app/helper Test SDK27/pins và native onboarding nhận quyền. **Chưa coi các milestone này hoàn tất native**: actual parity/exclusion/lifecycle matrix và canary còn mở. Output20unit/ACTION_RESTORE là giới hạn của spike P1 lịch sử, đã được xử lý trong module mới ở mức source/fixture. Không suy client hỗ trợ8192unit từ RAM round-trip.

## 2. Bằng chứng đã đọc và kiểm lại

Nguồn chat: `local / 01a18acf-937b-7651-be6f-f80e827ef1eb`, gồm quyết định giữ ABC, Windows chỉ lập plan, các lần sửa Cmd+A/Shift+Enter, lỗi fade, Figma `â` được nhưng `ngaan` chưa được, và đối chiếu MKey. Báo cáo agent trong chat là đầu mối; kết luận source dưới đây đã đọc lại độc lập.

Snapshot working tree: VXKey HEAD `ab0f2b728515134df8cee14e44a1a0f14379a880`, có 209 mục thay đổi/untracked trước lượt ghi tài liệu này. HEAD không đại diện toàn bộ source đang review. Reference xfunput `e63ac551967dcf17425c5f856591aace2fb8ace5`; xmkey `b3c6f616223f481309456f254448410728e56fca`. Không thay source trong lượt lập plan.

| ID | Quan sát và evidence source | Kết luận / giới hạn |
|---|---|---|
| RP01 | [event_driver.swift](../../native/macos_background/event_driver.swift), `handle`, gọi `ContextGuard.rewriteStart`; [context_guard.swift](../../native/macos_background/context_guard.swift), dòng107–119, bắt snapshot verified đúng PID/epoch, tuổi dưới80ms; [stream_window.swift](../../native/macos_background/stream_window.swift) giới hạn8key/120ms | Xác nhận: thiếu AX proof vẫn chặn rewrite; bỏ role whitelist không khắc phục editor thiếu API range. Phù hợp hiện tượng user báo, chưa phải trace chứng minh nguyên nhân từng lần Figma lỗi. |
| RP02 | [runtime_settings.swift](../../native/macos_background/runtime_settings.swift), dòng148–182 và `shouldBypass`; [bridge/local.rs](../../crates/vxkey-runtime/src/bridge/local.rs), latch `bridge_disconnected` | Xác nhận: local core vẫn có thể bị ngừng do bridge/lease500ms. Đây là đường lỗi có thật trong source; chưa chứng minh lần fade cụ thể do lease. Tăng socket timeout lên3s không bỏ lease500ms. |
| RP03 | [system_tab.tsx](../../desktop/src/components/system_tab.tsx), dòng46–50; [permission_cache.swift](../../native/macos_background/permission_cache.swift), `ready` | Xác nhận: UI khóa theo AX+listen+post, driver khóa theo AX+post. Ba capability check không đồng nghĩa ba checkbox quyền độc lập của OS. |
| RP04 | [background tests](../../native/tests/background_driver/main.swift), [runner](../../scripts/test_background_driver.sh), ô LiveTypingTest | Phần lớn là core/planner/guard fixtures; có kiểm callback bất hoạt và event construction nhưng không thay tap đang hoạt động, client render, focus và phím vật lý. Số tests tăng không chứng minh native compatibility. |
| RP05 | [MKey bridge](../../../xmkey/Sources/Platform/MKBridge.mm), dòng102; [MKey hook](../../../xmkey/Sources/Platform/MKEngineHook.mm), dòng1173–1206 | MKey dùng session tap; AX replacement có fallback phát phím. Đây là bằng chứng có hướng khác, không chứng minh MKey an toàn tuyệt đối hoặc adapter này tự tương thích Funput. |
| RP06 | [MKey README](../../../xmkey/README.md), mục license; VXKey [LICENSE](../../LICENSE) | MKey ghi toàn bộ code GPLv3; VXKey hiện MIT. Chỉ tham khảo hành vi/kiến trúc, tự triển khai; không copy glue/engine hoặc kéo dependency GPL vào task này. Nếu muốn tái sử dụng code cần quyết định license riêng. |

Kiểm mới trong lượt này: `cargo +stable test -p funput-core --offline --test telex` → **50 passed, 0 failed**, gồm corpus, permutations và `tone_between_shortcut_w_and_compound_w_keeps_the_horn`. Lượt lọc `--exact` đầu tiên chọn0tests, không tính là evidence PASS. Chưa chạy native typing, chưa build/launch/cấp quyền binary mới; không ghi đè evidence QA cũ. Core từng có lỗi `ddwojwc` thật và đã có bản sửa: không kết luận core không bao giờ lỗi. Kết quả Telex không bao phủ toàn bộ tính năng engine.

## 3. Vì sao vòng làm cũ tốn công

- Chọn AX readback bắt buộc trong khi mục tiêu là gõ được trên nhiều text client không đồng nhất; sau đó sửa quanh timeout, role và checkpoint. Đây là xung đột mục tiêu với cơ chế, không chỉ thiếu một ngoại lệ Figma.
- Trộn ba vấn đề: correctness core, truyền phím tới editor và vòng đời/quyền/bridge. Một mẫu gõ đúng sau Resume không phân biệt được ba lớp.
- Build nhiều candidate và thao tác TCC trước khi chốt cơ chế input tối thiểu. Packaging, permission UI và source tests che mất gate quan trọng nhất: chữ thật trong app thật.
- Spec/index/work log có snapshot chồng nhau. Spec09/10 mô tả AX proof và latch bridge; index vẫn có đoạn IMK/frontend chưa làm. Không giao agent chỉ dựa vào câu cuối chat hoặc tổng số PASS.

## 4. Các phương án và quyết định đề xuất

| Phương án | Đánh đổi | Quyết định |
|---|---|---|
| Tiếp tục AX bắt buộc, tăng budget/thêm app rules | Giữ document verification khi editor hỗ trợ, vẫn không giải quyết thiếu API range; dễ thêm trạng thái và độ trễ | Không chọn làm hướng tối ưu mặc định. Giữ source làm baseline đối chiếu. |
| Session tap + core cục bộ + event history, AX hỗ trợ | Phù hợp yêu cầu ABC, giảm phụ thuộc client; thay đổi consistency contract, không có xác nhận text tuyệt đối | Chọn hướng return-original sau P1; tiếp tục P2/P3, chỉ promote khi gate tích hợp đạt. |
| Quay lại InputMethodKit | Có text-client protocol nhưng trái quyết định không cài/chọn Input Source; compatibility vẫn cần kiểm | Không tự triển khai. Chỉ đưa lại thành lựa chọn có đánh đổi nếu spike không đạt. |
| Thay core/đưa MKey vào VXKey | Chưa có bằng chứng cần đổi core; thêm parity và license scope | Ngoài kế hoạch. |

### Đường gõ đề xuất

Một session-level filtering tap, một composer native cùng process, một nơi tuần tự hóa composition. Callback chỉ đọc cache và thực hiện công việc hữu hạn; không chờ AX, socket, UI, filesystem hoặc network. Không cần chuyển core sang service khác vì composer hiện đã local.

Giữ focus generation, own-event nonce/PID, keyDown/keyUp pairing, hard bypass trước engine, secure-input bypass, giới hạn Unicode/batch và chuẩn bị toàn batch trước mutation/suppression. Với route đã qua pilot, post Backspace qua proxy rồi trả event gốc chứa Unicode; không suppress keyUp vật lý của phím này. `nil` suppression đã qua filter probe trên session tap, không bê kết luận `.null` của per-PID tap sang. Không lưu proxy để post về sau. Output dài còn gate riêng, không lấy post-pair cũ làm fallback mặc định khi chưa kiểm chứng.

Reset session khi đổi app/layout, mouse/drag/selection/navigation, Cmd+A, cut/paste/undo/redo, Enter/Shift+Enter, phím không được hỗ trợ, config đổi hoặc engine mất đồng bộ. Ngay ký tự đầu ở boundary mới phải khởi động được, không đợi space hay một từ “làm nóng”. Không dùng clipboard để thay chữ. AX hỗ trợ chỉ đọc phạm vi cần thiết ngoài callback; chưa thêm AX direct replacement hoặc rewrite cả document vào spike.

### Giới hạn an toàn phải nói đúng

Event history chỉ chứng minh VXKey đã thấy/gửi gì, không chứng minh editor đã hiển thị đúng hoặc caret không bị app di chuyển bằng code. Chuỗi Backspace+Unicode không là transaction atomic với editor. Autocomplete, selection và thay đổi text không phát sự kiện quan sát được vẫn là rủi ro cần đo.

Do đó **không gọi event history là document ownership đã xác minh**, không tự tuyên bố giữ nguyên KEY01 của [spec00](00_system_overview.md). Muốn promote phải có quyết định contract rõ: giữ bảo đảm verified-only và chấp nhận coverage hạn chế, hoặc dùng best-effort bounded rewriting với rủi ro/coverage được ghi nhận và user chấp nhận. Không giấu lựa chọn này bằng tên “safe fallback”. Trước đó chỉ chạy trong fixture/editor nháp được phép; không sửa dữ liệu thật để chứng minh an toàn.

### Control plane và lifecycle

Đề xuất native driver dùng config last-known-good đã xác thực theo revision; UI/status/clipboard chậm không tạo global typing fault. Không bỏ authentication, signer pins, config validation hoặc single-instance lock để đạt mục tiêu đó. Thiết kế owner/lifecycle trước khi bỏ lease: đóng window/WebView khác Quit app; Quit/update phải dừng đúng helper đang sở hữu, không để orphan driver. Helper crash không tự replay edit hoặc tự Resume từ durable fault.

Phân biệt `user_paused`, `permission_missing`, `engine_fault`, `tap_disabled`, `control_disconnected`. UI disconnect hiển thị tình trạng kết nối riêng; không báo driver active nếu không có status mới. Explicit Pause và fault latch vẫn có hiệu lực. Quyết định bounded auto-recovery cho tap timeout cần gate riêng; chưa được biến mọi fault thành auto-resume.

## 5. Thứ tự triển khai và gate dừng

| Mốc | Việc làm / đầu ra | Gate trước khi sang bước sau |
|---|---|---|
| P0 — baseline | Ghi dirty-tree source fingerprint, candidate hash/path/signer/OS, issue IDs còn mở; xác định đúng binary hiện chạy. Chốt test bundle ID/data/socket path và người sở hữu vòng thực thi. | Không ghi đè candidate đã cấp quyền; không chạy hai bộ xử lý phím. Có run manifest và cách dừng bằng mouse. |
| P1 — tap spike | Nhánh/file thử cô lập, chưa thay mặc định. Session tap smoke `azx → aX`, rồi nối Funput local để gõ `aa`, `ngaan`, `ngaan hangf`. Ghi order/suppression/counter bằng dữ liệu fixture, không log text cá nhân. | TextEdit và Figma canvas nháp: không lọt phím gốc, không đảo/lặp event, không kẹt keyUp. Nếu không đạt, dừng trước sửa UI/permission/packaging mở rộng. |
| P2 — typing và safety | Chạy ma trận dưới đây, physical typing và stress từng phím; so output app thật với core oracle. Có unit/state-machine tests cho lỗi xác định được. | Đúng text, không sửa sentinel ngoài vùng thử, ký tự đầu đúng sau reset; ghi rõ unsupported case. Một lỗi sửa nhầm prefix/suffix là blocker. |
| P3 — lifecycle | Fault injection có kiểm soát: UI stall/close, bridge mất, helper crash, Pause, Quit, sleep/wake, tap disable, config đổi liên tiếp; test capability quyền thực của exact bundle. | UI lỗi không gây pause vô cớ; explicit pause/fault không tự bật lại; Quit không còn hook; keyboard raw luôn dùng được khi bypass/fault. |
| P4 — quyết định và tích hợp | So baseline với spike, giải quyết KEY01/BG04/BG06 và scope retone/macro trước promotion. Sau đó mới thay adapter, nối status UI và hợp nhất permission readiness. | Không claim parity/permission retention từ source. Sync spec00/02/04/05/07/08/09/10 và command/status schema theo quyết định thật; Windows chưa sửa. |
| P5 — canary/release gate | Một candidate danh tính test ổn định, gõ dài hạn trên app user dùng; giữ nguồn/evidence cho rollback code. | User acceptance cho hướng consistency mới; native regression đạt, release/updater/retention theo track riêng. Không tự publish từ plan hoặc test PASS. |

P1 đã hoàn tất pilot giới hạn cho core Telex thông thường, tap/delivery/reset và các ca native nêu trên. Bước tiếp là MAC_ADAPT_01/02 rồi P3/P4; chưa thêm UI trang trí hoặc mở Windows. Không suy thời gian hoàn thiện toàn sản phẩm từ một lượt pilot đạt.

### Kết quả P1 đã kiểm chứng và bước tiếp

Candidate pilot đã kiểm: `build/session_spike_candidate_v6/vxkey_session_spike.app`, source ở `native/macos_session_spike`. [Evidence](../qa/session_spike_evidence.json), [trace return-original](../qa/session_spike_v6_return_original_trace.json), [artifacts](../qa/session_spike_v6_artifacts.json). User xác nhận cả TextEdit/Figma corpus Telex đúng; Enter/Shift+Enter/Cmd+A→xóa→gõ ngay đúng; TextEdit prefix/emoji/suffix và mouse selection giữ nguyên. Đây là subset P1/P2, không native acceptance toàn sản phẩm.

Đường phát được chọn để tiếp tục: giữ identity/keyDown/keyUp vật lý, đặt output Unicode lên event gốc; chỉ Backspace được tổng hợp trước nó. Có A/B post-pair cũ trên cùng v6. V5 trace cho thấy core callback/inlineC/GUIoracle đúng và synthetic packets tới annotated tail đúng nhưng user vẫn mất á/à; v6 thay delivery thì user trial đúng. Điều này xác nhận route mới giải quyết các ca đã thử, **chưa xác định nội bộ macOS/client đã bỏ chữ bằng cơ chế nào**. Private source đơn lẻ không sửa lỗi.

Pilot chưa có config/IPC/authenticated policy/full feature parity; output returned packet cap20UTF16, chưa ACTION_RESTORE/retone/macro/VNI đầy đủ. P2 phải giải quyết parity/long output và ma trận còn thiếu; P3 kiểm lifecycle thật của helper/companion, P4 mới tích hợp và đồng bộ contract. Test isolated AX-only thành công trên máy QA, không hứa mọi OS hoặc retention khi update.

Sau QA, spike đã Pause/Quit (tap_attached=false), MKey được mở lại, không đổi mode/setting/quyền MKey. Production VXKey không launch/thay bytes; giữ v6 đã cấp quyền nguyên path/hash cho vòng tiếp. Không tự chạy lại hoặc promote từ dòng PASS này.

## 6. Ma trận native bắt buộc

| Nhóm | Ca cần thử | Evidence |
|---|---|---|
| Client | TextEdit; Chrome/ChatGPT; Figma canvas text; VS Code/Cursor; terminal với secure-input case riêng | Exact app/build/field, phím đã gõ và text thật. Không suy “hàng nghìn app đều hỗ trợ”. |
| Core/input | `aa→â`, `ngaan→ngân`, `ngaan hangf→ngân hàng`, `ddwojwc→được`, `mwojwt→mượt`, `casch/cachs→cách`, `w→ư`; VI/EN, upper-case, fast/long typing | Core oracle và native output tách riêng. Giữ tone_style modern của user. |
| Boundary | Ký tự đầu sau Cmd+A→Delete, sau Shift+Enter, thay selection, đổi focus ngay giữa từ, space, Backspace, undo/redo, mouse drag, paste | Không từ “warm-up”; không mất/lặp/chuyển chữ sang app khác. |
| Data safety | Prefix/suffix sentinel, emoji/combining, non-BMP, autocomplete, app đổi caret/text bằng code khi không có AX range | Readback độc lập fixture/native, không chỉ planner tự tính output. Không claim atomic hoặc bảo vệ tuyệt đối khi chưa chứng minh. |
| Privacy/policy | Secure Input/password field, hard bypass, VI/EN memory, internal UI; clipboard đang pause/disabled | Không xử lý field nhạy cảm; nếu không nhận biết được một loại field thì ghi gap, không coi Secure Input bao phủ mọi password field. Clipboard không vào typing path. |
| Lifecycle/permission | Hai hook, helper crash, mất bridge, window close vs Quit, mất quyền/restart được phép, sleep/wake | First-cause reason, state transition và output raw; không tự thay TCC hoặc đổi layout. |
| Parity sau spike | TelexAdvanced/VNI, macro VI/EN, retone, double-space/native interlock, custom hotkeys | Từng capability có gate; thiếu coverage thì giữ chưa nghiệm thu, không âm thầm cắt tính năng. |

Đo callback duration và end-to-end latency trên cùng máy/client, có p50/p95/p99 và fast-typing baseline. Không dùng deadline AX80ms/120ms cũ làm chuẩn latency hoặc tăng số đó để che lỗi. Log vận hành không chứa raw keys, text xung quanh, clipboard hoặc nội dung tài liệu.

## 7. Quyền, điều phối và điều kiện dừng

Capability `AXIsProcessTrusted`, listen và post phải được phân biệt với các mục Settings OS thực sự hiển thị. Chỉ xin quyền API/driver cần và xác nhận tap thật chạy trên packaged helper. Không hứa “chỉ một quyền trên mọi macOS” trước phép thử native. Giữ test identity/path/signer nhất quán theo [chuẩn quyền suite](../../../_summary/standard_docs/desktop_system_permissions.md); codesign PASS không thay N→N+1 retention.

Lượt lập plan ban đầu không đổi TCC/process/binary. Sau đó user giao “đọc worklog… triển đi”, Codex tự viết P0/P1 và build identity thử riêng; user đã duyệt chạy/cấp riêng Trợ năng cho spike và tự xác thực OS. Không gọi Anti, không đổi entry production hoặc rebuild candidate đã có quyền. Nếu giao Anti sau này: task hẹp còn thiếu, một người sửa adapter, Codex review source và native evidence; không dùng completion report làm bằng chứng đạt. Đọc các bộ đếm task/log trước khi gọi; không reset các vòng cũ bằng đổi tên spike/chat. Giới hạn global: tối đa3vòng sửa sau lượt đầu, tối đa2tham vấn/lượt, dừng sau2vòng không tiến triển.

Mỗi vòng phải có hypothesis, một thay đổi cơ chế có thể đối chiếu, kết quả native và quyết định tiếp/dừng. Không có quyền hoặc chưa gõ native thì kết luận “chưa kiểm chứng”, không thay bằng build/test counts. Nếu session-tap spike vẫn lỗi, giữ lại trace xác định lỗi nằm ở suppression, event ordering, core hay client; không nối thêm whitelist/timeout mà chưa có nguyên nhân mới. Khi đạt giới hạn, báo phần còn lại; không mở subprocess mới để tiếp tục vòng cũ.

## 8. Quan hệ với spec hiện hành

[Spec09](09_background_driver.md) và [spec10](10_checkpoint_pipeline.md) vẫn mô tả contract driver hiện tại: per-PID tap, AX proof và bridge-fault latch. Đề xuất này cố ý khác các điểm đó và chưa thay thế chúng. [Index](../index.md) dẫn tới đây làm bước review/P1 tiếp theo. Khi P4 có quyết định, cập nhật đồng bộ và đánh dấu các điều bị thay thế; không để agent chọn tùy ý giữa hai contract.

Không đóng LOG_005/007/008/009 hay track dài hạn bằng lượt review này. Các candidate và báo cáo cũ vẫn là evidence lịch sử, không phải evidence của adapter đề xuất.
