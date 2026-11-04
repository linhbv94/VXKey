# Báo cáo phân tích và khắc phục LOG_005 (`Cmd+A` → xóa / `Shift+Enter` mất dấu chữ đầu tiên)

- **Ngày thực hiện:** 2026-11-03
- **Tài liệu bàn giao đối chiếu:** [argon_cmd_a_review.md](../handoff/argon_cmd_a_review.md), [10_checkpoint_pipeline.md](../spec/10_checkpoint_pipeline.md), [typing_issue_log.md](../../typing_issue_log.md)
- **Phạm vi sửa nguồn:** [context_guard.swift](../../native/macos_background/context_guard.swift), [event_driver.swift](../../native/macos_background/event_driver.swift), [main.swift](../../native/tests/background_driver/main.swift)
- **Trạng thái binary đã cấp quyền TCC:** Giữ nguyên không chạm `build/tone_order_candidate/desktop_candidate/VXKey.app` theo đúng rào cản bàn giao.

---

## 1. Tại sao các từ thứ 2, 3, 4 (`được được mượt`) sau `Space` gõ đúng nhưng từ đầu tiên (`ddwojwc`) sau `Cmd+A` → xóa hoặc `Shift+Enter` bị mất dấu?

Trong [event_driver.swift](../../native/macos_background/event_driver.swift):
- Khi người dùng gõ `Space` giữa các từ thông thường (`owned.last == " "` và `composer.buffer().isEmpty`), driver **không gọi `reset()`**: `epoch` giữ nguyên, `snapshot` đã xác minh trước đó vẫn còn hiệu lực, và `start` được tịnh tiến đồng bộ bằng `start += owned.utf16.count` (hoặc `start = snapshot.caret`) nên luôn `≥ 0`.
- Ngược lại, khi người dùng bấm `Cmd+A` (nhánh modifier `.maskCommand`), `Backspace`/`Delete` khi `owned.isEmpty`, hoặc `Shift+Enter` (`code == 36`), driver gọi `reset(immediate: false)`:
  - `epoch` tăng lên, `owned = ""`, `start = -1`, `snapshot = nil`.
  - `ContextGuard` chỉ có một expectation khởi tạo rỗng `Expected(revision: 1, text: "", start: -1)`.
- Từ trạng thái `start = -1` của từ đầu tiên này, **5 lỗi phối hợp** trong `EventDriver` và `ContextGuard` làm từ đầu tiên (`ddwojwc`) rơi vào `abandonWord()` (`bypassWord = true`) ngay ở phím thứ 2 (`'d'` → `"đ"`), khiến cả từ `ddwojwc` đi nguyên phím cho tới khi gặp `Space` mở lại từ thứ 2 (`được`).

---

## 2. Danh sách Findings theo mức độ nghiêm trọng

### Finding 1 (P0 — Critical): Neo sớm `start` vào `snapshot.caret` của probe rỗng (`rev: 1`) trên phím `ACTION_NONE` và trừ sai `owned.utf16.count` trên phím `ACTION_SEND`
- **Vị trí:** [event_driver.swift](../../native/macos_background/event_driver.swift) (`EventDriver.handle`, nhánh `if start < 0, let snapshot, snapshot.verified`)
- **Cơ chế lỗi:**
  1. **Nhánh 1A — Phím 1 (`'d'`, `ACTION_NONE`) khóa cứng `start` vào `caret` cũ trước khi phím 1 được gõ:**
     - Trước khi sửa, câu lệnh `start = snapshot.start >= 0 ? snapshot.start : snapshot.caret - owned.utf16.count` chạy **trước** `composer.processText` cho cả phím `ACTION_NONE`.
     - Khi probe rỗng `rev: 1` (`snapshot.start == -1, snapshot.text == ""`) hoàn tất trước phím 1 (`'d'`), tại thời điểm phím 1 đi vào `handle`, `owned` đang là `""` (`count == 0`). Do đó `EventDriver` gán `start = snapshot.caret` (tọa độ caret đo **trước khi gõ `'d'`**, và thường là tọa độ cũ trước khi ChatGPT/Chrome kịp cập nhật DOM/AX cho `Shift+Enter` hoặc `Cmd+A` → `Backspace`, hoặc trước khi ProseMirror thay thẻ `<br class="ProseMirror-trailingBreak">` bằng text node).
     - Tiếp theo, `expectOwned()` gửi `Expected(revision: 2, text: "d", start: start)` với `start ≥ 0` cố định thay vì `start = -1`.
     - Trong `ContextGuard.read`, vì `entry.start ≥ 0`, điều kiện `range.location == entryStart + 1` bắt buộc caret sau khi gõ `'d'` phải bằng đúng `caret_cũ + 1`. Khi `caret_cũ` lệch dù chỉ 1 vị trí (do `Shift+Enter` xuống dòng hoặc `Cmd+A` → xóa vừa hoàn tất), `rev: 2` (`"d"`) **không bao giờ khớp**, đồng thời `rev: 1` không còn được làm mới vì `entries` đã có `"d"`. Tới phím 2 (`'d'`), proof hết hạn 80ms hoặc không khớp → `abandonWord()`.
  2. **Nhánh 1B — Phím 2 (`'d'`, `ACTION_SEND`) lấy `snapshot.caret - owned.utf16.count` = `0 - 1 = -1`:**
     - Trong `ContextGuard.read`, mọi snapshot không rỗng (`!entry.text.isEmpty`) đều đã có `snapshot.start = entryStart ≥ 0`. Snapshot duy nhất có `snapshot.start < 0` là snapshot rỗng `rev: 1` (`snapshot.text == ""`), có `snapshot.caret` đo **trước khi `owned` (`"d"`) xuất hiện**.
     - Khi phím 1 (`'d'`) được gõ lúc `rev: 1` chưa kịp trả về (`start` vẫn là `-1`, `owned` thành `"d"`), rồi tới phím 2 (`'d'`) mới đọc `snapshot` của `rev: 1` tại đầu ô nhập (`snapshot.caret == 0` sau `Cmd+A` → `Backspace`), công thức `snapshot.caret - owned.utf16.count` tính ra `0 - 1 = -1`.
     - Kết quả: `start = -1 < 0`, đụng `guard let snapshot, snapshot.verified, start >= 0` → gọi `abandonWord()` ngay lập tức ở phím `'d'` thứ hai.

### Finding 2 (P0 — Critical): Lỗi RPC tạm thời của `kAXStringForRangeParameterizedAttribute` trên dòng mới (`<br>`) hoặc vùng vừa xóa trả về `.invalid` làm thu hồi vĩnh viễn (`revoked = true`) cả epoch mới
- **Vị trí:** [context_guard.swift](../../native/macos_background/context_guard.swift) (`ContextGuard.read` / `ContextGuard.evaluate`)
- **Cơ chế lỗi:**
  - Sau khi `ContextGuard.read` đã xác minh đầy đủ PID, focused element, không phải `AXSecureTextField`, và `kAXSelectedTextRangeAttribute` hợp lệ, vòng lặp `for entry in input.entries.reversed()` gọi `kAXStringForRangeParameterizedAttribute` để đọc lát cắt ứng viên `CFRange(location: entryStart, length: entry.text.utf16.count)`.
  - Trong khung chat ProseMirror của ChatGPT, ngay sau `Shift+Enter` (khi dòng mới chỉ có `<br class="ProseMirror-trailingBreak">` chưa có text node) hoặc ngay sau `Cmd+A` → `Backspace` (khi DOM vừa xóa văn bản cũ và phím `'d'` đầu tiên chưa kịp render), lệnh đọc `kAXStringForRangeParameterizedAttribute` cho `rev: 2` (`"d"`) có thể trả về mã lỗi khác `.success`.
  - Trước khi sửa, `else { return .invalid }` biến một candidate slice chưa sẵn sàng thành `.invalid`, gán `revoked = true` **sticky cho toàn bộ epoch** trong khi `owned == "d"` khác rỗng. Khi phím `'d'` thứ hai tới, `guard !context.isRevoked` thất bại và gọi `abandonWord()`.

### Finding 3 (P0 — High): Selection `Cmd+A` (`range.length > 0`) chưa kịp thu gọn trong AX sau `Backspace` (hoặc khi gõ đè trực tiếp lên vùng chọn `Cmd+A`) làm `revoked = true` trên epoch mới
- **Vị trí:** [context_guard.swift](../../native/macos_background/context_guard.swift) (`ContextGuard.read` / `ContextGuard.evaluate`) và [event_driver.swift](../../native/macos_background/event_driver.swift) (`EventDriver.handle`)
- **Cơ chế lỗi:**
  - Sau `Cmd+A`, `range.length > 0`. Khi người dùng bấm `Backspace` (hoặc gõ thẳng chữ `'d'` đè lên vùng bôi đen), `reset(immediate: false)` tạo epoch mới nhưng trình duyệt/ChatGPT cần vài chục ms để thu gọn selection về `range.length == 0` trong cây AX.
  - Timer 10ms của `ContextGuard` chạy trong khoảng giao thời này thấy `range.length != 0` liền trả `.invalid`, đặt `revoked = true` cho epoch mới (thậm chí hoàn tất sau khi phím 1 `'d'` đã vào `owned`).
  - Ngoài ra, nhánh khôi phục `else if owned.isEmpty, composer.buffer().isEmpty, context.isRevoked` trong `EventDriver.handle` trước đó tăng `epoch &+= 1` nhưng quên đặt lại `start = -1` và quên re-prime `context.expect`.

### Finding 4 (P1 — High): `ContextGuard.publish` vứt bỏ snapshot rỗng `rev: 1` đang in-flight khi phím 1 (`'d'`) vừa được thêm vào `entries`, làm `snapshot` giữ `nil`
- **Vị trí:** [context_guard.swift](../../native/macos_background/context_guard.swift) (`ContextGuard.publish`)
- **Cơ chế lỗi:**
  - Điều kiện cũ `!(s.text.isEmpty && (entries.last.map { !$0.text.isEmpty } ?? false))` kiểm tra `entries.last` thay vì `snapshot`.
  - Nếu probe `rev: 1` (`s.text == ""`) đang bay khi người dùng gõ phím 1 (`'d'`, thêm `rev: 2` vào `entries`), khi `rev: 1` trả về trên main thread, `publish` vứt bỏ `s` ngay cả khi `snapshot == nil`. Nếu người dùng gõ phím 2 (`'d'`) nhanh trước khi probe tiếp theo xong, `snapshot` vẫn là `nil` → `abandonWord()`.
  - Đồng thời, `publish` khóa `element = s.element` ngay từ `rev: 1` (`s.text == ""`), khiến mọi thay đổi node AX nội bộ khi ProseMirror tạo `<p>` mới sau `Shift+Enter` / `Cmd+A` → xóa bị coi là đổi focus (`.invalid`).

### Finding 5 (P1 — Medium): `expectOwned()` gọi `pump()` đồng bộ trước khi sự kiện phím rời `CGEventTap` và không có nhịp retry ngắn (2ms) sau `.waiting`
- **Vị trí:** [event_driver.swift](../../native/macos_background/event_driver.swift) (`expectOwned`) và [context_guard.swift](../../native/macos_background/context_guard.swift) (`expect`, `publish`)
- **Cơ chế lỗi:**
  - Gọi `pump()` với `immediate: true` ngay trong `EventDriver.handle` khiến worker AX đọc ứng dụng đích **trước khi phím `'d'` kịp rời callback `CGEventTap` để tới ChatGPT**, dẫn tới kết quả `.waiting` gần như chắc chắn ở lần đọc đầu, rồi phải đợi nhịp timer 10ms tiếp theo.

---

## 3. Thay đổi tối thiểu đã thực hiện & Bảo toàn bất biến an toàn

1. **[event_driver.swift](../../native/macos_background/event_driver.swift):**
   - Trước `composer.processText`, chỉ nhận `start = snapshot.start` khi `snapshot.start >= 0` (đã xác minh tiền tố khác rỗng trong AX). Các phím `ACTION_NONE` đầu từ giữ `start = -1` để worker AX tự xác định `entryStart = range.location - entry.text.utf16.count` ngay sau khi ký tự thực sự xuất hiện trong ô nhập.
   - Tại cổng `ACTION_SEND`, tính `resolvedStart = start >= 0 ? start : (snapshot.map { $0.start >= 0 ? $0.start : $0.caret } ?? -1)` (không trừ `owned.utf16.count` vào `snapshot.caret` của probe rỗng `rev: 1`), đảm bảo vẫn yêu cầu `snapshot.verified && resolvedStart >= 0` trước mọi thao tác phát `Backspace`/Unicode.
   - Khi khôi phục epoch rỗng bị `revoked`, đặt lại `start = -1` và gọi `context.expect(..., immediate: false)`.
   - Trong `expectOwned()`, dùng `immediate: false` để nhường sự kiện phím tới ứng dụng đích trước khi probe sau `2ms`.
2. **[context_guard.swift](../../native/macos_background/context_guard.swift):**
   - Tách hàm thuần `ContextGuard.evaluate(_:element:range:began:sliceReader:)` để kiểm thử tất định toàn bộ máy trạng thái khớp slice/selection.
   - Khi epoch mới chưa neo (`input.prior == nil && input.entries.allSatisfy({ $0.start < 0 })`), nếu selection chưa kịp thu gọn (`range.length != 0` hoặc `kAXSelectedTextRangeAttribute` trả `.noValue`/`.cannotComplete` tạm thời lúc rebuild DOM), trả `.waiting` thay vì `.invalid`. Khi đã neo từ (`input.prior != nil` hoặc có `entry.start >= 0`), mọi thay đổi selection vẫn trả `.invalid` (sticky revocation).
   - Khi `sliceReader` (`kAXStringForRangeParameterizedAttribute`) chưa đọc được slice ứng viên trong vòng lặp `entries`, thực hiện `continue` (dẫn tới `.waiting`, không làm mới tuổi proof) thay vì `.invalid`.
   - Trong `publish`: đổi điều kiện giữ snapshot rỗng thành `!(s.text.isEmpty && (snapshot.map { !$0.text.isEmpty } ?? false))`, backfill `s.start` cho các entry chưa neo trong cùng từ khi `s.start >= 0`, chỉ khóa `element = s.element` khi `!s.text.isEmpty || s.start >= 0`, và thêm `scheduleRetry()` (`2ms`) khi còn revision mới chưa xác minh trong cửa sổ `150ms` kể từ `lastExpectTime`.
3. **[main.swift](../../native/tests/background_driver/main.swift):**
   - Bổ sung kiểm thử Telex cho `"ddwojwc"` → `"được"` và `"ddwojwc ddwojwc ddwojwc muwotj"` → `"được được được mượt"`.
   - Bổ sung 5 kiểm thử hồi quy `LOG_005` cho `ContextGuard.evaluate` và `ContextGuard`:
     - Selection `Cmd+A` (`range.length > 0`) trên epoch mới chưa neo trả `.waiting`, không thu hồi;
     - Selection thay đổi sau khi đã neo từ (`prior != nil`) vẫn trả `.invalid` và thu hồi proof;
     - Lỗi đọc slice tạm thời qua `<br>` sau `Shift+Enter` trả `.waiting`, không thu hồi epoch;
     - Phím đầu tiên chưa neo sau `Shift+Enter` xác định đúng `start = 11` sau ký tự xuống dòng;
     - Giữ proof rỗng `rev: 1` khi `snapshot == nil` lúc phím 1 đang chờ và tự động retry `2ms` sau `.waiting` để neo `start = 0`.

---

## 4. Bằng chứng kiểm thử & Giả thuyết chưa xác nhận (Chờ QA gõ thật trên ChatGPT)

- **Bằng chứng kiểm thử tự động đã chạy (exit 0):**
  - `sh scripts/test_engine.sh` (từ `vxkey/`): toàn bộ unit/integration tests của `funput-core`, `funput-engine`, `funput-config`, `funput-convert`, `funput-ffi`, `vxkey-runtime`, Settings UI gate, và **77/77 Swift checks** trong [background_edit_evidence.json](background_edit_evidence.json) PASS.
  - `cargo +stable check --manifest-path desktop/src-tauri/Cargo.toml --offline`: PASS.
- **Giới hạn chưa xác nhận:** Vì quy tắc bàn giao cấm ghi đè `build/tone_order_candidate/desktop_candidate/VXKey.app` đang giữ quyền TCC của user, bản sửa này đã được chứng minh bằng bộ kiểm thử tự động Rust + Swift nhưng **chưa chạy native typing QA trực tiếp trong cửa sổ ChatGPT thật**.
- **Bước tiếp theo cho Codex:** dùng nguồn đã sửa để đóng gói ứng viên mới (hoặc cập nhật bundle khi user sẵn sàng cấp lại/giữ TCC) và kiểm chứng trực tiếp chuỗi `Cmd+A` → xóa → `ddwojwc được được mượt` và `Shift+Enter` → `ddwojwc được được mượt` trong khung chat ChatGPT.
