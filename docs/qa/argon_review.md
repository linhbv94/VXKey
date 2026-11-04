# Báo cáo Review Kiến trúc & Mã nguồn — VXKey Background Driver (macOS)

---

## 1. Cơ chế gây hiện tượng gõ Telex đúng lúc đầu rồi mất chuyển đổi giữa câu

Trong kiến trúc hiện tại, có **hai đường dẫn lỗi khác nhau** trong [event_driver.swift](native/macos_background/event_driver.swift) và [context_guard.swift](native/macos_background/context_guard.swift) giải thích trực tiếp vì sao gõ chậm từ đơn `tieengs Vieetj` ban đầu ra đúng `tiếng Việt`, nhưng khi gõ câu dài tự nhiên `'ok, [Vietnamese text works] ... nayf ddaay (doan cuoi mat roi)'` lại mất biến đổi ở đoạn cuối (`nayf ddaay`):

### A. Lỗi xác minh từ mã nguồn (Source-Supported Defects)

1. **Cơ chế 1 (Chết hẳn giữa câu — Permanent Driver Fault khi gõ phím tiếp theo nhanh hơn vòng lặp AX): `edit_unverified_before_next_key`**
   - **Vị trí:** `native/macos_background/event_driver.swift:187-193`, `244-247`, `125-128` và `native/macos_background/runtime_settings.swift:175, 252`.
   - **Diễn biến:**
     - Mỗi khi một phím kích hoạt rewrite (ví dụ phím `s` cuối từ `tieengs`, phím `j` cuối `Vieetj`, phím `f` trong `nayf`, hoặc phím `d` thứ hai trong `dd`), dòng `244-245` gọi `context.expect(...)` (xóa ngay `snapshot = nil` tại `context_guard.swift:36`) và đặt `pendingAt = ProcessInfo.processInfo.systemUptime`.
     - Để `pendingAt` được xóa về `nil`, toàn bộ chuỗi sau phải hoàn tất **trước khi người dùng chạm phím vật lý tiếp theo**:
       `tapPostEvent(proxy)` đẩy chuỗi sự kiện tổng hợp qua WindowServer → ứng dụng đích (TextEdit/WebView) chạy main run loop xử lý toàn bộ Backspace + Unicode và cập nhật cây Accessibility → timer 10ms của `ContextGuard.probe()` kích hoạt **sau** thời điểm đó → `probe()` thực hiện tuần tự **5 lệnh IPC đồng bộ của AX** (`kAXFocusedApplicationAttribute`, `kAXFocusedUIElementAttribute`, `kAXSubroleAttribute`, `kAXSelectedTextRangeAttribute`, `kAXStringForRangeParameterizedAttribute`) và ghi lại `snapshot.verified = true`.
     - Khi gõ câu tự nhiên, khoảng cách giữa phím dấu thanh cuối từ (`s`, `f`, `j`, `r`, `x`) và phím `Space` (ngón cái), hoặc giữa `dd` và nguyên âm `a` kế tiếp trong `ddaay`, thường chỉ **15ms đến 50ms** (key rollover).
     - Khi phím vật lý tiếp theo tới callback tại dòng `187-192` trong khi `ContextGuard` chưa kịp xác minh chuỗi mới:
       `if pendingAt != nil { fault("edit_unverified_before_next_key"); return pass }`
     - Hàm `fault()` (`event_driver.swift:125-128`) đặt `failed = true`, gọi `setRunning(false)` (tháo event tap) và `AppSettings.shared.driverFailed()` (`suspended = true`). Theo hợp đồng không tự phục hồi sau fault (`docs/spec/09_background_driver.md:21`), **chỉ cần 1 lần chạm ngưỡng này, bộ gõ sẽ tắt vĩnh viễn cho toàn bộ phần còn lại của câu** (`... nayf ddaay`), biến mọi từ phía sau thành gõ thô.

2. **Cơ chế 2 (Mất dấu cục bộ giữa từ — Silent `reset()` khi 2 phím thường liên tiếp nhanh hơn vòng lặp AX)**
   - **Vị trí:** `native/macos_background/event_driver.swift:223-228`, `235-239` và `native/macos_background/context_guard.swift:36`.
   - **Diễn biến:**
     - Ngay cả với các phím bình thường không gây rewrite (`result.result.action == ACTION_NONE`, ví dụ `n` → `a` → `y` trong `nayf` hoặc chữ `d` đầu tiên trong `ddaay`), dòng `238` vẫn gọi `context.expect(pid: pid, text: owned, start: start, epoch: epoch)`.
     - Lệnh `expect()` lập tức xóa `snapshot = nil` (`context_guard.swift:36`).
     - Khi phím kế tiếp tới ở dòng `223`, `guard let snapshot = context.latest(...), snapshot.verified, snapshot.text == owned` đòi hỏi `ContextGuard.probe()` phải kịp đọc lại ký tự vừa gõ từ ứng dụng đích qua 5 lệnh AX IPC.
     - Nếu người dùng gõ 2 phím liên tiếp (`n`→`a`, `a`→`y`, hoặc `d`→`d`) nhanh hơn độ trễ cập nhật AX của ứng dụng đích + chu kỳ 10ms của `probe()`, `guard` ở dòng `223` thất bại (`note("unverified")`) và dòng `227` thực thi:
       `else if pendingAt == nil { reset() }`
     - `reset()` gọi `composer.clear()` (xóa sạch âm tiết đang ghép trong `FunputComposer`), đặt lại `owned = ""`, tăng `epoch`, và trả phím hiện tại đi thẳng (`return pass`) mà không đưa vào `composer`. Kết quả là âm tiết bị đứt gãy giữa chừng, nên khi gõ tới phím dấu (`f`) hoặc phím đôi (`d` thứ hai), `composer` không còn ngữ cảnh nguyên âm/phụ âm trước đó và xuất nguyên văn `nayf` / `ddaay`.

3. **Cơ chế 3 (`lock.try()` làm rơi âm thầm lệnh `expect()` hoặc `latest()`)**
   - **Vị trí:** `native/macos_background/context_guard.swift:33, 39`.
   - **Diễn biến:** Cả `expect()` và `latest()` đều dùng `guard lock.try() else { return }`. Khi main thread gọi `expect()` (dòng `244` sau khi rewrite) đúng lúc worker `probe()` đang giữ `lock` ở dòng `46` hoặc `81-86` để copy biến, `expect()` bị bỏ qua âm thầm: `EventDriver.owned` đã đổi sang từ mới nhưng `ContextGuard.expected` vẫn giữ từ cũ. Khi đó `s.text == owned` tại `verifyPending()` (`event_driver.swift:122`) không bao giờ khớp được nữa, chắc chắn dẫn tới `fault("edit_unverified_before_next_key")` ở phím kế tiếp hoặc `fault("edit_verification_timeout")` sau 150ms.

4. **Cơ chế 4 (Tích lũy dấu câu vào `owned` + tính năng tự đổi ký tự của macOS/App làm lệch `owned` → `edit_verification_timeout`)**
   - **Vị trí:** `native/macos_background/event_driver.swift:123, 226, 231, 236-238`.
   - **Diễn biến:** Dòng `231` chỉ reset `owned = ""` khi ký tự cuối là dấu cách (`owned.last == " "`). Các dấu câu ASCII như `,`, `.`, `'`, `(`, `)` đều đi qua dòng `236` (`owned.append(...)`). Trong câu mẫu có `'ok, ... nayf ddaay'`, khi gõ `...` hoặc `'`, tính năng Smart Quotes / Smart Punctuation của macOS (TextEdit, WebView) tự động thay `...` (3 UTF-16 code units) thành `…` (`U+2026`, 1 UTF-16 code unit) hoặc `'` thành `’` (`U+2019`). Ngay khi độ dài hoặc nội dung trong tài liệu lệch khỏi `owned`, mọi lần kiểm tra `range.location == start + expected.utf16.count` và `found == expected` tại `context_guard.swift:72-77` đều trả về `verified = false`, gây `reset()` hoặc `fault("edit_verification_timeout")`.

### B. Giả thuyết & Hành vi môi trường chưa được chứng minh trực tiếp (Hypotheses)

- **Giả thuyết H1 (Thay đổi nút `AXUIElement` nội bộ trong WebView/Rich Editor):** Trong các trình soạn thảo dùng DOM/ProseMirror/Lexical (như ChatGPT desktop nêu ở mục 7 của packet), việc gõ dấu cách, xuống dòng hoặc định dạng tự động có thể tạo lại phần tử con `AXUIElement` đang focus. Vì `context_guard.swift:34, 84` chỉ xóa `element` khi đổi `pid` hoặc `epoch` (mà việc tách từ mới sau dấu cách tại `event_driver.swift:231` lại không tăng `epoch`), điều kiện `CFEqual(prior!, el)` tại `context_guard.swift:73` có thể thất bại nếu `AXUIElement` của đoạn văn đổi handle giữa câu.

---

## 2. Đánh giá chính sách xác minh AX (150ms timeout, 80ms freshness, next-key fault) và tính nhất quán của các ranh giới

### A. Chính sách xác minh AX hiện tại có tương thích cấu trúc với gõ bình thường/nhanh không?

**Không tương thích về mặt cấu trúc.** Lý do định lượng như sau:

1. **Xung đột thời gian thực giữa tốc độ gõ phím (15ms–50ms) và độ trễ vòng lặp AX liên tiến trình (15ms–100ms+):**
   - Trong gõ 10 ngón thông thường (60–100+ WPM), khoảng thời gian giữa 2 phím lệch ngón (rollover như `e`→`e`, `d`→`d`, phím dấu → `Space`) chỉ từ **15ms đến 50ms**.
   - Trong khi đó, `ContextGuard.probe()` chạy trên timer định kỳ `10ms` (`context_guard.swift:27`) và thực hiện **5 lời gọi Mach RPC đồng bộ sang tiến trình đích** (mỗi lời gọi có timeout `40ms`, tổng ngân sách tối đa `200ms`).
   - Quan trọng hơn: trên macOS (AppKit/WebKit), cả việc xử lý sự kiện bàn phím (`NSEvent`) lẫn việc trả lời truy vấn Accessibility (`AXUIElementCopyAttributeValue`) của ứng dụng đích **đều chạy trên main thread của ứng dụng đích**. Khi `tapPostEvent(proxy)` bơm một loạt 6–10 sự kiện (ví dụ 3 Backspace down/up + chuỗi Unicode down/up), `probe()` rất dễ đọc trúng trạng thái trung gian (khi ứng dụng đích mới xóa được 1–2 ký tự, chưa chèn xong chuỗi mới), tạo ra snapshot có `verified = false`. Nếu phím vật lý tiếp theo đến trước chu kỳ `probe()` thành công kế tiếp, dòng `192` lập tức đánh sập driver (`fault("edit_unverified_before_next_key")`).
2. **Mâu thuẫn nội tại giữa ngưỡng tươi 80ms (`0.08s`) và timeout AX (`5 × 40ms = 200ms`):**
   - `latest()` (`context_guard.swift:42`) từ chối mọi snapshot có tuổi `> 80ms`.
   - Nhưng một chu kỳ `probe()` đơn lẻ có thể hợp lệ mất tới `50ms–120ms` khi ứng dụng đích đang bận render/layout. Chỉ cần 1 chu kỳ `probe()` kéo dài quá `80ms`, `latest()` trả về `nil`, làm phím gõ tiếp theo bị `unverified` → `reset()` (nếu `pendingAt == nil`) hoặc `fault()` (nếu `pendingAt != nil`).
3. **Chi phí IPC nền không cần thiết:**
   - Timer `10ms` chạy liên tục không điều kiện (`100 lần/giây × 5 AX RPC = 500 lệnh IPC/giây` sang ứng dụng foreground) ngay cả khi người dùng không gõ phím nào.

### B. Tính nhất quán của các ranh giới (Reset / Lease / Config / Focus / Ownership)

| Ranh giới | Trạng thái | Phân tích chi tiết |
| :--- | :--- | :--- |
| **Ownership & Word Boundary** | **Không nhất quán** | Khi qua từ mới sau dấu cách tại `event_driver.swift:231` (`if owned.last == " ", composer.buffer().isEmpty { owned = ""; start = snapshot.caret }`), driver không tăng `epoch`, không làm mới `ContextGuard.element`, và không coi dấu câu (`,`, `.`, `(`, `)`, `:`, `/`) là ranh giới ngắt `owned`. |
| **Reset vs Non-Text / Navigation Keys** | **Không nhất quán** | Phím có tổ hợp `Cmd`/`Ctrl`/`Opt` (dòng `176`) nằm **trước** cổng kiểm tra `pendingAt` (dòng `187`) nên gọi `reset()` an toàn. Ngược lại, các phím điều hướng/kết thúc dòng (`Enter` `36`, `Tab` `48`, `Escape` `53`, mũi tên `123..126` tại dòng `219`) và `Backspace` (`51` tại dòng `195`) lại nằm **sau** dòng `187`. Do đó gõ từ có dấu rồi nhấn `Enter` hoặc `Tab` ngay lập tức sẽ làm driver bị `fault("edit_unverified_before_next_key")` thay vì chỉ `reset()`! |
| **Config Apply Boundary** | **Hở điều kiện** | Tại `runtime_settings.swift:95, 171`, `composing` chỉ kiểm tra `!(driver?.composer.buffer().isEmpty ?? true)`. Khi `composer.buffer()` rỗng nhưng `pendingAt != nil` hoặc `owned` đang giữ từ vừa gõ (phục vụ retone sau Backspace), `apply(config)` + `driver?.syncSettings()` vẫn chạy giữa chừng mà không đồng bộ lại `owned`/`pendingAt`. |
| **Bridge Lease Boundary** | **Nhất quán** | Cơ chế giữ lease `0.5s` khi `busy == true` (`runtime_settings.swift:156-163, 176, 248`) hợp lý, tránh disconnect giả khi mutex phía Rust bận ngắn hạn. |
| **Focus Boundary** | **Nhất quán cấp App, hở cấp Element** | Cấp PID (`NSWorkspace.didActivateApplicationNotification` + `refreshTarget()` + kiểm tra `focusedPID == pid` trong `probe()`) chặt chẽ. Tuy nhiên nếu đổi ô nhập liệu trong cùng một PID bằng phím `Tab` ngay sau một rewrite, dòng `187` sẽ gây `fault()` trước khi tới dòng `219`. |

---

## 3. Kiểm toán Callback Suppression, Event Batch, Synthetic Down/Up, Nonce/PID Skip, KeyUp Tracking & Lifecycle

1. **Per-PID Tap Callback Suppression (`event_driver.swift:16-19, 160, 173-174, 211, 248`):**
   - Bằng chứng thực nghiệm #4 và #5 đã xác nhận trên macOS 26 / SDK 27 rằng trả về `nil` từ callback của `CGEvent.tapCreateForPid` không chặn được phím gốc, trong khi gán `event.type = .null` rồi trả `Unmanaged.passUnretained(event)` chặn được phím gốc.
   - **Rủi ro phụ:** Sự kiện `.null` vẫn đi tiếp vào luồng sự kiện của PID đích. Để phòng thủ chiều sâu (tránh trường hợp một handler cấp thấp vẫn đọc trường keycode/unicode của event `.null`), nên xóa thêm chuỗi Unicode của event gốc (`event.keyboardSetUnicodeString(stringLength: 0, unicodeString: [])`) trước khi đặt `event.type = .null`.

2. **Thứ tự Event Batch & Synthetic Down/Up (`event_driver.swift:129-149, 211, 247`):**
   - Việc gọi `e.tapPostEvent(proxy)` cho toàn bộ batch trước khi biến sự kiện gốc thành `.null` đảm bảo thứ tự `[Backspace down/up ..., Unicode down/up ...]` tại đúng vị trí tap trong stream.
   - **Rủi ro 1 — Gán `virtualKey: 0` (`kVK_ANSI_A`) và đóng gói tới 20 UTF-16 code units vào 1 sự kiện (`event_driver.swift:141-143`, `edit_plan.swift:8-17`):**
     - `virtualKey: 0` trên macOS là phím chữ `A` (`kVK_ANSI_A`). Trong AppKit `NSTextView` chuẩn, `keyboardSetUnicodeString` ghi đè ký tự xuất ra; nhưng trong WebView/Electron/ứng dụng lắng nghe `KeyboardEvent.keyCode`/`code` hoặc khi chuỗi `output` có nhiều ký tự (ví dụ `"ếng"` = 3 ký tự gộp chung trong 1 sự kiện `keyDown` có `virtualKey: 0`), một số khung nhập liệu web chỉ lấy ký tự đầu tiên hoặc xử lý sai sự kiện đa ký tự. Phát từng scalar (1 ký tự / cặp surrogate mỗi cặp `keyDown`/`keyUp`) an toàn hơn gộp 20 UTF-16 units.
   - **Rủi ro 2 — `e.flags = []` và `keyboardEventSource: nil` khi người dùng đang giữ phím `Shift` (`event_driver.swift:135, 141, 147`):**
     - Tạo `CGEvent(keyboardEventSource: nil, ...)` dùng trạng thái nguồn sự kiện mặc định chung trước khi gán `e.flags = []`. Nếu người dùng đang giữ `Shift` để gõ chữ hoa (`Vieetj` hoặc `DD`), việc bơm các sự kiện tổng hợp có `e.flags = []` từ `source: nil` có thể làm một số ứng dụng hiểu nhầm trạng thái modifier vật lý. Nên khởi tạo từ một `CGEventSource(stateID: .privateState)` riêng biệt.

3. **Rủi ro hỏng văn bản (Text Corruption) do `ContextGuard.probe()` không xóa `snapshot` khi `guard` thất bại (`context_guard.swift:42, 47-69`):**
   - **Đây là lỗi an toàn nghiêm trọng:** Trong `ContextGuard.probe()`, nếu một `snapshot` đã được xác minh (`verified = true`) tại thời điểm `t = 0ms`, rồi ở `t = 10ms` người dùng bôi đen văn bản (`range.length > 0` tại dòng `69`), hoặc chuyển vào ô mật khẩu (`kAXSecureTextFieldSubrole` tại dòng `64`), hoặc đổi focus (`dòng 54, 58`), thì `probe()` thoát sớm bằng `guard ... else { return }` **mà không xóa `self.snapshot = nil`**!
   - Do `latest()` cho phép dùng lại `snapshot` trong suốt `80ms` (`context_guard.swift:42`), nếu người dùng bấm phím gây rewrite hoặc Backspace trong khoảng 70ms còn lại đó, `EventDriver` vẫn nhận được `snapshot` cũ (`verified = true`) và bắn chuỗi `Backspace` tổng hợp, **xóa mất vùng văn bản người dùng vừa bôi đen hoặc xóa nhầm vị trí con trỏ mới**.

4. **Nonce / PID Skip (`event_driver.swift:147, 157`):**
   - Dòng `157` kiểm tra: `event.getIntegerValueField(.eventSourceUserData) == nonce && event.getIntegerValueField(.eventSourceUnixProcessID) == Int64(getpid())`.
   - Khi tạo sự kiện bằng `CGEvent(keyboardEventSource: nil, ...)`, trường `.eventSourceUnixProcessID` không được gán tường minh trong `batch()`. Tương tự như hiện tượng `eventTargetUnixProcessID == 0` ở bằng chứng #1, nếu macOS để `.eventSourceUnixProcessID == 0` trên sự kiện tổng hợp và sự kiện đó đi qua callback, phép `&&` ở dòng `157` sẽ thất bại dù `nonce` (63-bit ngẫu nhiên) đã khớp.

5. **Theo dõi `keyUp` (`consumed: Set<UInt16>`) và nguy cơ kẹt phím (`event_driver.swift:48, 82-88, 104-108, 160`):**
   - Tập `consumed` **không bao giờ được làm sạch** trong `removeTap()` (`dòng 82-88`) cũng như trong `reset()` (`dòng 104-108`).
   - Nếu một phím `keyDown` được đưa vào `consumed` (ví dụ phím tắt toggle ở dòng `174` hoặc phím rewrite ở dòng `246` ngay trước khi driver bị `fault()` / `removeTap()`), và sự kiện `keyUp` của lần bấm đó diễn ra khi tap đã tháo, mã phím `code` đó sẽ **nằm kẹt vĩnh viễn trong `consumed`**. Ở phiên Resume sau (hoặc khi chuyển sang app khác), lần đầu người dùng bấm phím `code` đó: `keyDown` đi qua bình thường vào app, nhưng `keyUp` lại bị dòng `160` (`consumed.remove(code) != nil`) nuốt mất (`Self.suppress(event)`), gây hiện tượng **kẹt phím (stuck key / autorepeat)** ở ứng dụng đích!

---

## 4. Đánh giá hướng tiếp cận: Sửa tăng tiến (Incremental Fix) hay Thiết kế lại ranh giới (Boundary Redesign)?

- **Giữ nguyên kiến trúc tổng thể người dùng đã chọn:** Tiến trình chạy nền + giữ nguyên nguồn nhập ABC + per-PID `CGEvent.tapCreateForPid` + chặn phím tại chỗ bằng `event.type = .null` + phát sự kiện qua `tapPostEvent(proxy)` + lõi Rust `FunputComposer`. **Không cần và không nên quay lại `InputMethodKit` / Input Source.**
- **Tuy nhiên, cần thiết kế lại có trọng tâm ở đúng 1 ranh giới nội bộ: Ranh giới Xác minh AX Đồng bộ Giữa Từng Phím (`Per-Key Synchronous AX Gate`).**

### So sánh các phương án cho ranh giới Ownership / Verification (trong kiến trúc Background Driver)

| Tiêu chí | Phương án 1: Chỉ vá nhỏ chính sách hiện tại (Giữ cổng AX sau mỗi phím, tăng timeout, đổi `fault` thành `reset`) | Phương án 2 (Khuyến nghị): Optimistic Stream Ownership trong âm tiết + AX Guard bất đồng bộ theo thế hệ (Generation) |
| :--- | :--- | :--- |
| **Cơ chế hoạt động** | Giữ nguyên việc xóa `snapshot = nil` ở mỗi phím (`expect()`); chỉ sửa `lock.try()` → `lock.lock()`, và khi phím kế tiếp đến trước khi AX kịp verify (`pendingAt != nil`) thì gọi `reset()` thay vì `fault()`. | `ContextGuard` làm hàng rào an toàn về **Focus / Element / Non-Secure / Collapsed Selection** (xóa ngay snapshot nếu bất kỳ guard nào trượt hoặc có chuột/điều hướng). Trong phạm vi một âm tiết trên cùng `AXUIElement` khi không có sự kiện ngoại lai (chuột, điều hướng, modifier), `EventDriver` tin tưởng chuỗi sự kiện tuần tự do chính mình phát ra (`owned` + `composer`) mà **không khóa cứng phím kế tiếp chờ AX IPC đọc lại từng ký tự**. Việc đọc lại AX (`kAXStringForRangeParameterizedAttribute`) chỉ dùng để neo đầu từ hoặc hậu kiểm khi luồng phím nghỉ (`> 80–120ms`); nếu lệch chỉ `reset()`, không `fault()`. |
| **Tương thích tốc độ gõ thực tế (60–120 WPM)** | **Kém:** Không còn chết hẳn bộ gõ giữa câu, nhưng mỗi khi gõ lướt nhanh (`dd`, `ee`, phím dấu + `Space` < 40ms) vẫn bị `reset()` làm rớt dấu của từ đó. | **Tốt:** Chịu được key rollover 10ms–30ms vì luồng sự kiện trong cùng `CGEventTap` (`tapPostEvent(proxy)` + `.null`) đã được xếp hàng tuần tự tại điểm tap trước phím vật lý kế tiếp. |
| **Tương thích WebView / Electron (ChatGPT, IDE)** | **Kém:** Cây AX của WebView cập nhật trễ theo frame (16ms–50ms+), liên tục trượt cổng `latest()` giữa 2 phím. | **Tốt:** Không bị phụ thuộc vào độ trễ render DOM/AX giữa hai phím liên tiếp trong cùng một từ. |
| **An toàn (Chống sửa nhầm ngoài vùng sở hữu)** | Cao nhưng quá nhạy (false-positive cao). | Vẫn đảm bảo an toàn nếu xóa ownership ngay khi có `mouseDown`, phím điều hướng, tổ hợp `Cmd`/`Ctrl`/`Opt`, đổi PID/Element, hoặc khi `ContextGuard` phát hiện `range.length > 0` / secure field. |
| **Chi phí thay đổi** | Rất nhỏ (~15 dòng trong Swift). | Vừa phải (~40–60 dòng gói gọn trong `event_driver.swift`, `context_guard.swift` và cập nhật mục Delta trong `docs/spec/09_background_driver.md:50-52`). |

---

## 5. Danh sách phát hiện xếp hạng theo mức độ ưu tiên (≤ 8 Findings)

### Nhóm I — Các lỗi đã xác minh từ mã nguồn & bằng chứng (Verified Defects)

#### Finding 1 [P0 — Critical]: Gõ phím tiếp theo khi `pendingAt != nil` gây `fault("edit_unverified_before_next_key")` làm tắt vĩnh viễn bộ gõ giữa câu
- **Vị trí:** [event_driver.swift](native/macos_background/event_driver.swift) (`native/macos_background/event_driver.swift:187-193, 244-247, 125-128`) và [09_background_driver.md](docs/spec/09_background_driver.md) (`docs/spec/09_background_driver.md:52`).
- **Điều kiện kích hoạt (Trigger):** Người dùng gõ một phím sinh rewrite (ví dụ `e` thứ hai trong `tiee`, `d` thứ hai trong `dd`, hoặc phím dấu `s`/`f`/`j` cuối từ) rồi bấm phím vật lý tiếp theo (như `Space`, nguyên âm kế tiếp, hoặc `Enter`) trong khoảng `15ms–50ms` — trước khi ứng dụng đích xử lý xong chuỗi `tapPostEvent` và `ContextGuard.probe()` hoàn tất 5 lệnh AX IPC để cập nhật `snapshot.verified = true`.
- **Hậu quả (Consequence):** Dòng `192` gọi `fault("edit_unverified_before_next_key")`, đặt `failed = true`, `suspended = true` và tháo event tap. Toàn bộ văn bản gõ tiếp theo trong câu (`... nayf ddaay`) mất hoàn toàn khả năng gõ tiếng Việt cho đến khi người dùng mở UI bấm Resume thủ công.
- **Độ tin cậy (Confidence):** **High** (Khớp trực tiếp với mã nguồn dòng `187-193` và hiện tượng mất biến đổi ở đoạn cuối câu khi gõ liên tục).
- **Hướng sửa tối