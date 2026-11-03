# VXKey — checkpoint pipeline cho background driver

Ngày: 2026-11-02. Thay thế chính sách lỗi theo từng phím trong delta cuối spec09 trước đó. Nguồn: [event_driver](../../native/macos_background/event_driver.swift), [context_guard](../../native/macos_background/context_guard.swift), [stream_window](../../native/macos_background/stream_window.swift).

## Quy tắc hiện hành

- Chỉ khởi đầu stream từ snapshot AX đã xác minh: đúng PID, element, không secure, selection collapsed. Có thể dùng TextArea chuẩn không có AXSubrole; lỗi RPC hoặc single-line field không xác định subrole chuyển nguyên phím.
- Trong stream, AX có thể xác nhận một revision cũ đã biết. Các revision mới hơn vẫn được giữ, không reset composer chỉ vì checkpoint trễ.
- Cửa sổ chưa xác minh bị giới hạn8key và120ms từ key chưa xác minh cũ nhất; snapshot còn phải mới dưới80ms tính từ đầu probe. Không coi đây là application ACK hoặc bảo đảm atomic.
- Hết cửa sổ/mất snapshot: bỏ ownership và chuyển nguyên phím tới hết từ. Space/navigation cho phép neo lại stream mới. Không tự đặt global driver_fault chỉ vì AX chậm.
- Hook bị disable, mất quyền, lỗi engine hoặc bridge vẫn dùng cơ chế suspend/watchdog hiện có. Không che các lỗi đó bằng fallback từ.
- Context mailbox chỉ được ghi/đọc trên main thread. Chỉ một chuỗi AX worker in-flight; expectation mới coalesce không bị mất do try-lock. Worker không giữ lock mà callback cần.
- Probe focus/selection/secure/RPC thất bại sẽ thu hồi proof, sticky trong epoch. Kết quả worker thuộc epoch cũ không được publish; checkpoint cũ không được ghi đè checkpoint mới.
- Snapshot waiting khi chưa khớp một trạng thái stream đã biết không làm mới tuổi proof. Proof cũ tự hết hạn, không tồn tại vô hạn.
- Callback không đợi AX RPC, không disk/socket; vẫn dùng per-PID tap, proxy tapPostEvent và suppress in-place .null đã qua native probe trước.
- Tháo tap dọn consumed keyUp và modifier gesture; config không apply khi còn revision chưa xác minh. Không reload cùng config revision liên tục.
- casch và cachs đều phải ra cách. Không ép vị trí đặt dấu để tránh gate.

## Ranh giới và giới hạn

Backspace retone chỉ dùng exact current proof; không dự đoán sửa lại vùng không còn được xác minh. Word-boundary và ASCII input vẫn theo engine hiện có. Không đọc full document hoặc clipboard để khôi phục từ bị bỏ.

Hàng rào AX là bất đồng bộ. Programmatic selection/focus/text thay đổi trong khoảng từ probe tới event vẫn là race cần native QA; cửa sổ dự đoán hữu hạn không xóa được race đó. Không tuyên bố bảo vệ atomic mọi app. Khi checkpoint không theo kịp, một từ có thể đi nguyên phím; mục tiêu là không tự tắt toàn bộ bộ gõ và không tiếp tục sửa bằng proof vô hạn.

## Bằng chứng và nghiệm thu

[49 Swift checks](../qa/background_edit_evidence.json), [test log](../qa/checkpoint_driver_tests.log): real Funput/EditPlan và StreamWindow, delayed prefix ACK, pending cap/age, lossless coalescing khi AX giả bận, invalid proof sticky/epoch reset, casch/cachs. Không post phím thật. [Bridge fixture](../qa/checkpoint_bridge_tests.log): runtime_settings thật + Rust bridge/cache, hook OS mock;3Resume/Pause.

Native candidate phải thử: câu dài ở tốc độ thường/nhanh, casch/cachs, Space/Enter/Tab ngay sau dấu, Pause/Resume khi giữ phím, đổi app/field/selection, secure field, prefix/suffix sentinels và Undo. TextEdit và WebView là hai track. Chưa có native acceptance cho bản checkpoint này.

## Delta neo prefix sau bàn giao Argon

ACTION_NONE được ghi nhận prefix phím gốc đã gõ từ lần reset khi chưa có anchor AX. Worker xác minh đúng prefix ngay trước caret để xác định start; không scan từ/document tùy ý. ACTION_SEND vẫn cần snapshot verified và start>=0. Reset khi chuyển nguyên navigation/mouse có thể trì hoãn probe tức thì; epoch rỗng bị revoked có thể re-prime. Xem [report tiếp nhận](../qa/word_handoff_review.md); native acceptance còn chờ.
