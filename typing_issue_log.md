# Nhật ký lỗi gõ VXKey

> VXKey thuộc [track bảo trì dài hạn](maintenance_track.md), chỉ user được đóng. Agent hoàn tất từng lỗi không được tự đóng track.


Ghi trực tiếp vào file này trong lúc dùng. Chưa rõ nguyên nhân cứ ghi hiện tượng; không cần tự chẩn đoán. Bản đang thử: checkpoint_candidate, ngày 2026-11-02.

## Ghi nhanh lỗi mới ở đây

### Lỗi mới — [ngày/giờ nếu nhớ]

- Platform / bản build: 
- App / ô nhập: 
- Tôi đã gõ các phím: `...` (nếu nhớ; đừng đoán lại từ kết quả)
- Thực tế hiện ra: `...`
- Tôi muốn hiện ra: `...`
- Ghi chú: đầu câu/giữa câu, nhanh/chậm, vừa đổi app/click/chọn chữ/Backspace/Enter… nếu có liên quan.

Có thể chỉ dán câu bị lỗi và đánh dấu đoạn sai. Một lỗi một mục; thêm mục mới lên trên. Không cần ghi nội dung nhạy cảm.

---

## Lỗi đang theo dõi

### LOG_005 — Cmd+A/xóa làm từ đầu mất chuyển dấu trong ChatGPT

- User gõ dở → Cmd+A → xóa → gõ tiếp; từ đầu mất chuyển dấu, Space rồi gõ lại thì được.
- App: chính khung soạn ChatGPT, word_candidate; ngày2026-11-02.
- Trạng thái: **Đã sửa ở mã nguồn native (`context_guard.swift`, `event_driver.swift`); chờ Codex đóng gói candidate mới và QA gõ trực tiếp trên ChatGPT.**
- **Trial latency_candidate:** user làm lại Cmd+A/xóa và báo `đưojwc cái`, cảm giác vẫn chưa mượt. LOG_005/006/007 vẫn MỞ; TCC worker chưa giải quyết toàn bộ native pipeline.
- **Bản sửa & bằng chứng kỹ thuật (2026-11-03 — Anti, vòng sửa 1 sau chỉ đạo):**
  - Báo cáo chi tiết: [argon_cmd_a_fix_report.md](docs/qa/argon_cmd_a_fix_report.md).
  - Xác định và khắc phục 5 nguyên nhân phối hợp khiến từ đầu tiên (`ddwojwc`) sau `Cmd+A` → xóa hoặc `Shift+Enter` rơi vào `abandonWord()` trong khi các từ sau `Space` (`được được mượt`) vẫn đúng:
    1. `EventDriver.handle` trước đây gán sớm `start = snapshot.caret - owned.utf16.count` ngay trên phím 1 (`ACTION_NONE`) từ probe rỗng `rev: 1` (`snapshot.start == -1`), khóa cứng `start` vào `caret` cũ trước khi `Shift+Enter`/`Backspace` cập nhật AX (hoặc tính ra `0 - 1 = -1` ở phím 2 sau `Cmd+A` → xóa). Đã sửa để chỉ lấy `start = snapshot.start` khi `snapshot.start >= 0` trước `composer.processText`, giữ `start = -1` cho các phím `ACTION_NONE` đầu từ để worker AX xác định `entryStart = range.location - entry.text.utf16.count`, và tại cổng `ACTION_SEND` dùng `resolvedStart` không trừ `owned.utf16.count` vào `snapshot.caret` của probe rỗng.
    2. `ContextGuard.read` / `ContextGuard.evaluate` trước đây trả `.invalid` (gán `revoked = true` sticky cho toàn epoch) khi `kAXStringForRangeParameterizedAttribute` lỗi tạm thời lúc dòng mới của ProseMirror/ChatGPT đang ở `<br class="ProseMirror-trailingBreak">` hoặc vừa xóa vùng chọn. Đã sửa thành `continue` (trả `.waiting`, không làm mới tuổi proof, không thu hồi epoch).
    3. Selection `Cmd+A` (`range.length > 0`) chưa kịp thu gọn trong AX sau `Backspace` (hoặc khi gõ đè trực tiếp lên vùng chọn `Cmd+A`) trên epoch mới chưa neo (`input.prior == nil && input.entries.allSatisfy({ $0.start < 0 })`) nay trả `.waiting` thay vì `.invalid`; khi đã neo từ (`input.prior != nil`) thì thay đổi selection vẫn trả `.invalid`.
    4. `ContextGuard.publish` giữ lại proof rỗng `rev: 1` khi `snapshot == nil` (kiểm tra `snapshot` thay vì `entries.last`), chỉ khóa `element = s.element` khi `!s.text.isEmpty || s.start >= 0`, và backfill `s.start` cho các entry chưa neo trong cùng từ.
    5. `expectOwned()` dùng `immediate: false` kết hợp `scheduleRetry()` (`2ms`) trong `ContextGuard` để sự kiện phím kịp tới ứng dụng đích trước khi đọc AX và tự động thử lại nhanh sau `.waiting`.
  - Giữ nguyên không ghi đè `build/tone_order_candidate/desktop_candidate/VXKey.app` đang có quyền TCC; chờ Codex đóng gói candidate tiếp theo và kiểm thử gõ thật trong ChatGPT.

### LOG_006 — W sau đ lúc được lúc không

- User gửi các output: `đuọwc`, `đưojwc`, `được`, `đwoc`, `đw`. Không coi đây là chuỗi phím chính xác.
- Core độc lập: ddw→đư, ddwowcj/dduwowcj/ddwocj→được, casi→cái. [Evidence](docs/qa/restart_word_core_evidence.json).
- Trạng thái: **MỞ.** Core đúng không chứng minh native đúng; cần thử cùng Cmd+A/xóa và đầu từ bình thường.

### LOG_007 — Cảm giác gõ hơi khựng

- User nói hơi khựng, chưa rõ; word_candidate, ngày2026-11-02. Chưa có số đo input latency.
- [Sample helper3giây](docs/qa/word_engine_idle_sample.txt) bắt được main/tap loop chờ TCC preflight đồng bộ; không phải phép đo lúc user gõ.
- Source mới chuyển preflight quyền sang worker/cache hạn, bỏ TCC preflight mỗi AX probe và query tap trùng lặp. [Report](docs/qa/latency_review.md).
- Trạng thái: **Candidate đã build, chưa native/perceived-latency acceptance.** Chưa khẳng định nguyên nhân duy nhất.

### LOG_004 — Giữ A hiện bảng chọn dấu của macOS

- Ngày: 2026-11-02; bản đang dùng word_candidate.
- User xác nhận **giữ phím A**, không phải bấm–nhả từng lần.
- Quan sát: hiện bảng à/á/â… có số lựa chọn. Đây là Accent menu native khi press-and-hold trên macOS, theo [Apple Support](https://support.apple.com/en-ie/guide/mac-help/mh27474/27/mac/27).
- [Ảnh user gửi](docs/qa/screenshots/press_hold_accent_menu.png).
- Phân loại: hành vi native OS tương tác với nhu cầu giữ phím để kéo dài chữ; chưa phải bằng chứng lỗi core Telex. Phân biệt với REG_002 (chuỗi phím rời/kéo dài chữ).
- Trạng thái: **Đã xác định trigger; chưa đổi setting hệ thống hoặc triển khai override.** Nếu thiết kế cách giữ phím để lặp chữ trong VXKey, phải xét native setting/app scope và không âm thầm đổi toàn hệ thống.

### LOG_001 — W chưa ra Ư theo thói quen gõ

- Ngày: 2026-11-02.
- App: TextEdit, bản checkpoint_candidate.
- User báo: `thwr`, `nhwng`, `chwx`, `vwaf` còn nguyên w ở các đoạn cần chữ ư/dấu.
- Yêu cầu đã chốt: `w` lẻ → `ư`, `W` → `Ư`; `thw` → `thư`, `vwng` → `vưng`. Không giới hạn chỉ sau phụ âm.
- Trạng thái: **Đã sửa ở mã nguồn & C ABI (`funput-core`/`funput-engine`/`funput-ffi`); chờ build lại candidate và QA gõ trực tiếp trên TextEdit.**
- Ví dụ từ câu user gửi: “để tôi thwr gõ một câu dài xem nó ra cái gì nhé, đây câu này dài, nhwng chwx W k chuyển thành ư.”
- Lưu ý: ví dụ trên là văn bản quan sát được, không phải bản ghi chính xác mọi phím vật lý đã bấm.
- **Bản sửa & bằng chứng kỹ thuật (2026-11-02 — Anti):**
  - Nguyên nhân: `InputMethod::Telex` (`mode = 0`, mặc định trong config) trước đây không bật `TelexShortcut::LeadingW` và `TelexShortcut::RepeatedW` (chỉ bật ở `InputMethod::TelexAdvanced`, `mode = 2`), khiến `w`/`W` đứng đầu hoặc ngay sau phụ âm đầu (`thw`, `vwng`, `thwr`, `nhwng`, `chwx`) giữ nguyên chữ `w`, còn `vwaf` biến thành `vằ`.
  - Đã cập nhật `crates/funput-core/src/input_method/telex/advanced.rs`, `crates/funput-core/src/input_method/telex/mod.rs`, `crates/funput-core/src/composition/transform/mod.rs` và `crates/funput-core/src/composition/uo_horn/continuation.rs`: kích hoạt `LeadingW` + `RepeatedW` cho cả `InputMethod::Telex` (`mode = 0`) lẫn `InputMethod::TelexAdvanced` (`mode = 2`), đồng thời giữ `[` → `ơ` và `]` → `ư` riêng cho `TelexAdvanced`; gõ `ww` hoàn tác về `w` nguyên bản (`swwift` → `swift`, `lwwa` → `lwa`).
  - Bằng chứng kiểm thử core & bridge: `sh scripts/test_engine.sh` PASS (toàn bộ unit/integration tests của `funput-core`, `funput-engine`, `funput-ffi`, `vxkey-runtime` và 60/60 checks trong [background_edit_evidence.json](docs/qa/background_edit_evidence.json) xác nhận `w` → `ư`, `W` → `Ư`, `thw` → `thư`, `vwng` → `vưng`, `thwr` → `thử`, `nhwng` → `nhưng`, `chwx` → `chữ`, `vwaf` → `vừa` ở `.telex`).
  - **Phần còn mở cho Codex / QA:** Binary đang cấp quyền trong `build/checkpoint_candidate/` chưa được ghi đè để không làm mất quyền Accessibility/Input Monitoring hiện tại. Cần Codex điều phối đóng gói lại `checkpoint_candidate` (hoặc cấp lại quyền nếu ad-hoc hash đổi) và xác nhận gõ thật trên TextEdit.


- **Codex tiếp nhận:** đã review source, tự chạy tests và build `word_candidate` riêng; không ghi đè bản có quyền. [Report](docs/qa/word_handoff_review.md). Native typing của bản sửa mới vẫn chưa nghiệm thu.


- **Native trial 2026-11-02 — user xác nhận:** nhóm từ trong bài thử và đầu câu/câu dài đã đúng trên `word_candidate`. Đây là xác nhận theo lượt thử của user, chưa phải mọi ứng dụng/tốc độ/tình huống focus đều đã nghiệm thu.

### LOG_002 — Từ đầu câu đôi lúc không chuyển dấu

- Ngày: 2026-11-02.
- App: user báo chữ “cái” ở đầu câu vừa mất chuyển dấu; app của lần này chưa xác nhận.
- Văn bản báo lại: `casi`; user nói “vừa xong chữ 'cái' ở đầu câu bị mất vi”.
- Mong muốn: `casi` → `cái`, kể cả ở đầu câu hoặc sau khi quay lại ô nhập.
- Trạng thái: **Đã sửa ở mã nguồn native (`context_guard.swift`, `event_driver.swift`); chờ build lại candidate và QA gõ trực tiếp.**
- Không được yêu cầu user gõ chậm hơn để né lỗi.
- **Bản sửa & bằng chứng kỹ thuật (2026-11-02 — Anti):**
  - Nguyên nhân phối hợp trong `native/macos_background/context_guard.swift` và `native/macos_background/event_driver.swift`:
    1. Khi có sự kiện pass-through (`Enter`, `Backspace`, phím điều hướng, click chuột, tổ hợp phím) ngay trước từ đầu câu, `handle()` gọi `reset()` → `context.expect()` → `pump()` đồng bộ ngay trong callback `CGEventTap` *trước khi* sự kiện tới ứng dụng đích, làm `ContextGuard` đọc trạng thái caret/selection cũ (hoặc đang bôi đen chờ xóa) và đánh dấu `revoked = true` cho epoch mới.
    2. `verifyPending()` bỏ qua khi `!window.isPending`, nên trạng thái `revoked = true` lúc nghỉ (`owned.isEmpty`) không tự phục hồi cho tới khi có sự kiện `reset` khác.
    3. Ở các phím đầu từ chỉ có `ACTION_NONE` (như `c`, `a` trong `casi`), nếu `context.latest()` chưa kịp trả snapshot trước phím đầu, `EventDriver` lập tức gọi `abandonWord()` (`bypassWord = true` đến hết từ) dù `c` và `a` chưa hề cần phát xung sửa văn bản (`ACTION_SEND`), chỉ tới phím dấu `s` mới cần snapshot AX đã xác thực.
  - Đã cập nhật `native/macos_background/context_guard.swift` và `native/macos_background/event_driver.swift`:
    1. Dùng `reset(immediate: false)` cho các sự kiện pass-through để không đọc AX trước khi ứng dụng đích nhận phím/click; tự động re-prime epoch trong `verifyPending()` hoặc đầu từ mới nếu epoch rỗng trước đó bị `revoked`.
    2. Cho phép các phím đầu từ có `ACTION_NONE` ghi nhận prefix chưa neo (`start = -1`, `owned = "c"`, `"ca"`) và kích hoạt `ContextGuard.read` xác thực chuỗi ngay trước caret bằng `kAXStringForRangeParameterizedAttribute` để suy ra `start = range.location - entry.text.utf16.count`; mọi thao tác phát xung sửa chữ (`ACTION_SEND`, như phím `s` trong `casi`) vẫn bắt buộc có `snapshot.verified` và `start >= 0`.
  - Bằng chứng kiểm thử fixture: `sh scripts/test_engine.sh` PASS (60/60 Swift checks trong [background_edit_evidence.json](docs/qa/background_edit_evidence.json), bao gồm kiểm thử neo `start` cho prefix đầu câu `ca` và khôi phục sau `revoked`).
  - **Phần còn mở cho Codex / QA:** Chưa kiểm thử gõ thực tế trên `checkpoint_candidate` mới vì giữ nguyên binary đang có quyền TCC; cần Codex rebuild candidate khi sẵn sàng và kiểm chứng gõ thật đầu câu / sau click / sau `Enter` / sau chọn-xóa.


- **Codex tiếp nhận:** đã review source, tự chạy tests và build `word_candidate` riêng; không ghi đè bản có quyền. [Report](docs/qa/word_handoff_review.md). Native typing của bản sửa mới vẫn chưa nghiệm thu.


- **Native trial 2026-11-02 — user xác nhận:** nhóm từ trong bài thử và đầu câu/câu dài đã đúng trên `word_candidate`. Đây là xác nhận theo lượt thử của user, chưa phải mọi ứng dụng/tốc độ/tình huống focus đều đã nghiệm thu.

### LOG_003 — Gõ câu dài từng làm cả bộ gõ ngừng chuyển dấu

- Ngày: 2026-11-02.
- Bản cũ: null_candidate.
- Hiện tượng: câu bắt đầu gõ Việt đúng nhưng phần cuối thành `nayf ddaay`; icon menubar mờ.
- Bằng chứng kỹ thuật: `edit_unverified_before_next_key`; 31 rewrite, 30 checkpoint verified rồi driver fault/suspended.
- Trạng thái: **Đã đổi pipeline, cần theo dõi hồi quy.** Trên checkpoint_candidate, user đã gõ được câu dài và nhiều lần “cách”; vẫn còn LOG_001/LOG_002. Chưa đánh dấu ổn định hoặc daily-driver ready.

---

## Các hành vi cần giữ đúng

### REG_001 — Đặt dấu linh hoạt

- `casch` và `cachs` đều phải ra **cách**.
- User đã thử trên checkpoint_candidate và gửi nhiều dòng “cách”.
- Lõi Rust C ABI cũng đã kiểm tra cả hai.
- Trạng thái: **Đã đúng trong các lượt thử này; giữ làm hồi quy.**

### REG_002 — Kéo dài chữ

- User xác nhận tình huống kéo dài chữ đang làm đúng với VXKey.
- Các ví dụ user nhắc: `áaaaaa`, `aaaaá`; chưa có cặp phím đầu vào → kết quả kỳ vọng riêng cho từng ví dụ.
- Trạng thái: **Giữ hành vi hiện tại.** Nếu lần sau sai, ghi rõ chuỗi phím và kết quả, không tự suy quy tắc từ ví dụ.

### REG_003 — Kiểu đặt dấu oà / uý

- User đã yêu cầu **giữ nguyên setting hiện tại**; không thay vì đoạn minh họa trước đó.
- Config QA hiện tại: `tone_style = modern`.
- Trạng thái: **Không có yêu cầu đổi.**

---

## Ghi chú cho agent khi đọc log

Đây là lời báo lỗi và yêu cầu của user, không phải evidence mọi lỗi đã tái hiện. Không tự đánh dấu đã sửa chỉ vì build/test qua. Khi xử lý một mục, bổ sung bản sửa, evidence thật và phần còn mở ngay dưới mục đó; giữ nguyên mô tả gốc.

[Work log tổng](../_summary/work_log.md) · [Report candidate hiện tại](docs/qa/checkpoint_pipeline_review.md) · [Spec pipeline](docs/spec/10_checkpoint_pipeline.md)


### LOG_006 — Bổ sung bằng chứng thứ tự phím 2026-11-02

- Trace user gửi từ `typing_probe.html` (plain, giới hạn 200 events) có bước lỗi `đưo → đuợ → đuọw → đuọwc`. Các chunk `uợ` và `ọw` là text do bộ gõ phát ra. Trace trình duyệt không thấy trực tiếp các phím đã bị tap chặn, vì vậy thứ tự phím dưới đây là suy luận đối chiếu, không phải raw hardware log.
- Chạy độc lập qua Rust C ABI: `ddwojwc` → **đuọwc**, khớp từng bước; `ddwowjc` và `ddwowcj` → **được**. Biến thể lỗi đặt `j` trước `w` thứ hai.
- Đã xác nhận có lỗi core theo thứ tự phím, không cần timing/browser để tái hiện. Các test trước bỏ sót `ddwojwc`. Chưa sửa core, chưa build candidate mới; LOG_005/007 và native QA vẫn mở. User không cần đổi thói quen gõ.

- **Bản sửa core 2026-11-02:** regression `ddwojwc` FAIL trước sửa, PASS sau sửa; hai Telex modes/tone styles và C ABI lặp nhiều từ PASS. [Report](docs/qa/tone_order_review.md). Candidate mới `tone_order_candidate` đã build riêng, native QA còn mở; không đánh dấu LOG_006 hết lỗi trong dùng thật.
- **Bổ sung user giữ nguyên mẫu:** `đayas <= lỗi tôi giữ nguyên nhé`; đồng thời báo `mượt đuọwc mượt được mượt đuọwc`. Chưa có raw sequence đầy đủ cho các mẫu này; không quy hết cho core bug đã tái hiện hoặc tốc độ gõ.

### Native trial tone_order_candidate — bàn giao review

- User gõ tay sau Cmd+A/xóa trong ChatGPT và gửi nguyên văn: `ddwojwc được được mượt`. User xác nhận “cmd A xong bị cái chữ đầu nhé” và “mượt hơn hẳn”.
- LOG_005 vẫn MỞ: từ đầu raw; chưa xác định nguyên nhân. LOG_006: core regression đã sửa, nhóm từ sau đúng trong lượt này; chưa coi ổn định mọi thứ tự/tốc độ. LOG_007: user thấy cải thiện rõ, chưa đo latency hay xác định nguyên nhân, giữ theo dõi.
- [Packet Argon review](docs/handoff/argon_cmd_a_review.md). User tự giao; Codex không gọi agent, không sửa thêm nguồn trong lượt bàn giao.

### LOG_005 — Bổ sung Shift+Enter (user 2026-11-02)

User báo “lại lỗi chữ đầu sau khi shift enter” khi đang dùng tone_order_candidate. Giữ như biến thể đầu dòng mới của lỗi chữ đầu; chưa có raw key trace hoặc tái hiện độc lập xác định nguyên nhân. Không suy là lỗi quyền. Cần thử Shift+Enter → gõ ngay từ tiếng Việt trong khung chat, cùng Cmd+A → xóa.
- **Cập nhật 2026-11-03 (Anti):** Đã xử lý chung trong bản sửa [argon_cmd_a_fix_report.md](docs/qa/argon_cmd_a_fix_report.md) (tránh khóa `start` vào `caret` dòng cũ trước `Shift+Enter`, tránh thu hồi epoch khi đọc `kAXStringForRangeParameterizedAttribute` qua `<br>` trước khi text node đầu dòng hình thành, và bổ sung regression tests `LOG_005 Shift+Enter`). Chờ QA gõ trực tiếp trên candidate mới. Trạng thái MỞ cho tới khi user xác nhận trên ChatGPT.

### QA safety bản sửa Argon 2026-11-03

LOG_005 chưa nghiệm thu: [VXKEY_SAFE_001 và report](safety_audit/run_20261103_argon_review/report.md) xác nhận source/fixture empty proof cho phép rewrite nonempty owned. Chưa build/native QA bản mới; tone_order_candidate giữ nguyên. Suite PASS không chứng minh lỗi chữ đầu đã hết.

- 2026-11-03 Codex vòng sửa2: [vá safety](safety_audit/run_20261103_cmd_a_fix/report.md) empty-proof rewrite; 86Swift checks PASS, candidate riêng cmd_a_safe_candidate. LOG_005 native chưa nghiệm thu, không sửa bằng cách bỏ ownership proof.

### Physical trial cmd_a_safe_candidate — 2026-11-03

User trả lời bài Cmd+A/xóa và Shift+Enter: “lần đầu thì k được. nhưng xong thì lại được”. Chưa tách được nhánh nào lỗi hoặc raw output lần đầu; LOG_005 tiếp tục MỞ. Codex chạy thêm86Swift checks PASS và kiểm lại candidate hashes/strict codesign; chưa đủ native acceptance. [Bàn giao release](docs/handoff/release_1_0_0_handoff.md); không đóng track dài hạn.

### LOG_008 — Icon tự fade, phải bật lại từ tray

- Ngày2026-11-03; user: “thi thoảng nó bị fade đi mà tôi k rõ tại sao, phải bật lại ở tray menu”. Trạng thái **MỞ**, chưa có thao tác tái hiện/thời điểm chính xác; không đánh đồng với chủ động Pause hoặc app bypass.
- Kiểm readonly: recovery marker production có `reason_code=bridge_disconnected`, suspended=true, mtime11:22:05+0700; failed_component_version=0.1.0. Marker không phải log đầy đủ và có thể do restart/disconnect có chủ đích, chưa chứng minh đúng sự cố user kể.
- Source dim mọi trạng thái ngoài Live; `bridge_disconnected` đặt latch cần Resume. Cần lưu reason/timestamp và show lý do ngay tray thay vì chỉ fade chung. [Điều tra](docs/qa/figma_fade_investigation.md).

### LOG_009 — Gõ tiếng Việt chập chờn trong text trên canvas Figma

- Ngày2026-11-03; user xác nhận xảy ra ở **text trên canvas**, không phải kết luận cho đổi tên layer hay input thuộc tính. Trạng thái **MỞ**. Chưa có chuỗi raw/output tối thiểu có chủ đích hoặc test controlled.
- Readonly CUA thấy canvas/document AX và screen-reader board disabled tại lúc xem; không ở phiên edit có focused text attributes. Không sửa thiết kế/layer/setting Figma, không paste/type thử vào file thật; không lưu nội dung canvas vào report.
- VXKey dựa AX selection + StringForRange, cần proof<80ms và stream tối đa8keys/120ms; lỗi proof có thể bỏ riêng từ tới Space. Canvas custom AX là giả thuyết cần kiểm chứng, không khẳng định “Figma không hỗ trợ bộ gõ”. Không nới proof guard để chữa. [Điều tra](docs/qa/figma_fade_investigation.md).

### Cập nhật LOG_008/009 — chỉ sửa VXKey, 2026-11-03

User yêu cầu không đổi setting Figma. Đã sửa timeout bridge1s→health3s (lease0.5s giữ nguyên), thêm timestamp cho marker mới; AXDocument/AXWebArea trong Figma được thử exact-proof như editor, không nới proof hoặc blind rewrite. Thêm nontext/rate-limited AX diagnostic.54Rust +92Swift +backend check PASS. [Report source fix](docs/qa/figma_fade_source_fix.md). **Chưa bundle/native QA; hai lỗi còn MỞ, Figma role/range thực chưa xác nhận đầy đủ.** Không đổi Figma/OS setting.

### Rà compatibility theo chỉ đạo user — 2026-11-03

Không đặc cách Figma. Đã gỡ role/app allowlist và thay việc một phím thiếu proof → bỏ cả từ bằng deferred observed/logical reconciliation cho mọi editor có API đủ.102Swift checks PASS, chưa bundle/nativeQA. [Report và gap AX còn mở](docs/qa/compatibility_rule_review.md). Editor thiếu AX API usable là gap cần strategy khác của VXKey; không yêu cầu user đổi setting app ngoài. LOG_005/008/009 vẫn MỞ.

### Pilot riêng v6 — không đóng lỗi production

- User xác nhận Telex TextEdit/Figma, Enter/Shift+Enter/Cmd+A→xóa, prefix/emoji/suffix và mouse selection đúng trên `build/session_spike_candidate_v6/vxkey_session_spike.app`. [Report/giới hạn](docs/qa/session_spike_review.md), [evidence](docs/qa/session_spike_evidence.json).
- Thay đổi có hiệu quả trong trial: trả Unicode trên event phím gốc, chỉ post Backspace. Private source riêng lẻ không sửa case mất á/à; core và downstream packet đã có trace đúng. Chưa xác định cơ chế nội bộ client, chưa tích hợp default/production.
- LOG_005/007/008/009 vẫn theo dõi production/dùng dài hạn; không suy fade đã sửa hoặc cảm giác mượt đã được user xác nhận từ corpus PASS. Spike đã Pause/Quit, MKey mở lại; không đổi mode/TCC MKey.

### Checkpoint native adapter tích hợp — 2026-11-04

LOG_005/007/008/009 vẫn MỞ. User xác nhận prescribed Telex + macro dài trên TextEdit/Figma và đầu từ sau Cmd+A/xóa/Shift+Enter trong TextEdit đúng; Illustrator cũng đang gõ ổn theo user. Exclusion TextEdit giữ nguyên phím và chặn macro, Figma vẫn gõ Việt. Đây là native QA có scope, không đóng lỗi ChatGPT/daily-use hoặc coi mọi editor/layout đã đạt. [Report](docs/qa/session_adapter_integration.md).

### LOG_010 — Khởi động lại Test bị Pause giả dù quyền vẫn còn

- Ngày2026-11-04, Codex tái hiện khi Quit/mở lại cùng session_integrated_candidate đãgrant. AX true sau observation, không cấp lại; nhưng tick đầu cache chưa có dữ liệu báo permission_required và tạo recovery latch, bắt Resume.
- Source đã sửa: unknown không cho hook chạy, chỉ báo lỗi quyền sau observation thật; denied thật vẫn latch.104Swiftchecks/6bridgegates PASS, hook mock. **Chưa nằm trong binary đang chạy; chưa native accept fix.** Không đồng nhất startup false-latch này với mọi lần fade LOG_008. [Evidence](docs/qa/session_adapter_evidence.json).

### LOG_011 — Flip thử nghiệm Ctrl+Option+F5 mở gợi ý từ

- Ngày2026-11-04; user: “ấn xong thấy nó có một list xổ xuống gợi ý từ 'pas'”. Đây là binding QA Codex gán tạm, factory flip vẫn null; chưa phải shortcut user chốt.
- Source flip bỏ modifiers nhưng giữ virtual keycode F5 khi trả Unicode. AppKit có F5 word completion, phù hợp triệu chứng; chưa exact native event trace/final output. **MỞ, chưa native accept flip**; không coi engine flip sai chỉ vì menu này.
- Sửa adapter cho special-key delivery/paired release + native QA sau; không đổi OS/TextEdit settings. [Report](docs/qa/session_adapter_integration.md).
- **Bản sửa & bằng chứng kỹ thuật (2026-11-04 — Anti, lượt followup):**
  - Nguyên nhân: Nhánh `flipShortcut` trong `session_event_driver.swift` trước đây chỉ gán `event.flags = []` rồi đẩy event vật lý gốc (vẫn giữ `keyCode: 96` của F5) qua `deliver()`. AppKit tiêu chuẩn trên macOS ánh xạ phím F5 (`\uF708`) tới selector `complete:` của `NSTextView`, làm mở menu gợi ý từ cho "pas".
  - Đã khắc phục qua delivery độc lập (`StreamDelivery.prepareStandalone`):
    1. Tổng hợp `plan.backspaces * 2` sự kiện phím ảo Backspace (virtualKey 51, `[down, up]`).
    2. Tổng hợp các sự kiện chèn chuỗi ký tự thay thế mang virtualKey an toàn `0` (`kVK_ANSI_A`), `flags: []`, đánh dấu `nonce` và kiểm tra xác thực CoreGraphics chấp nhận trọn vẹn chuỗi UTF-16 qua `keyboardGetUnicodeString`.
    3. Tiền kiểm an toàn: giới hạn Backspaces (0..128), kích thước output (<= 8192 units), non-null codepoints trước khi phát sự kiện nào ra ngoài.
    4. Nhánh `flipShortcut` trong `session_event_driver.swift` nuốt phím khi autorepeat, chuẩn bị gói standalone và phát qua `proxy.tapPostEvent()`, đồng thời nuốt trọn cả `keyDown` lẫn `keyUp` của phím tắt flip (`consumed.insert(code); return nil`), ngăn triệt để F5 lọt vào AppKit.
  - Kiểm thử fixture: 111/111 checks PASS trong `test_session_adapter.sh` (tăng từ 104); 6/6 bridge checks PASS; 59/59 Rust runtime tests PASS; 145/145 UI permission matrix checks PASS.
  - Đã đóng gói candidate test revision mới tại `build/session_followup_candidate` (data `build/sf_data`, artifacts [session_followup_artifacts.json](docs/qa/session_followup_artifacts.json)) tích hợp cả LOG_010, LOG_011 và UX frontend; giữ nguyên candidate cũ đang cấp quyền. Chờ native QA gõ thực tế với flip shortcut.

### Bổ sung QA2026-11-04 — toggle / Backspace / Z

User xác nhận toggle Control+Option `â→aa→â` và `chaof`, Backspace, `s`→`chá` đúng trên trial TextEdit; báo thêm “gõ dấu rồi bỏ dấu với Z cũng đang tốt”. Không có exact chuỗi cho Z hoặc full retone/tone matrix; giữ scope đã xác nhận. Không đóng các lỗi dùng dài hạn chỉ từ lượt QA này.
