# Điều tra fade và Figma canvas — 2026-11-03

## Hai hiện tượng user báo

LOG_008: thỉnh thoảng icon fade, phải bật lại từ tray. LOG_009: text trên canvas Figma gõ chập chờn. Giữ tách biệt; chưa có evidence chúng luôn cùng incident. Không sửa code hoặc thay quyền/setting trong lượt điều tra.

## Evidence readonly

- Recovery state production tại Library/Application Support/VXKey: reason_code=bridge_disconnected, suspended=true, failed_component_version=0.1.0, mtime11:22:05+0700. Không copy incident hoặc cấu hình/clipboard user. Marker này có thể từ restart/disconnect trước đó; không có timestamp sự cố user nên không quy mọi fade cho marker này.
- Unified log filter riêng com.vxkey.engine + driver_fault/tap_prepare_failed last2h: không có record. Lần đầu log tool sandbox exit64; đã chạy lại bằng escalation đọc riêng subsystem, exit0. Absence không bác bỏ fault đã xảy ra hoặc log cũ.
- Figwright ping server-only, plugin timeout, không có plugin connected. Không dùng tool thiết kế hoặc browser workaround. CUA đọc Figma OS accessibility để xem editor, không sửa file/canvas. Tree thấy canvas expose document và Figma notice board screen-reader disabled. Lúc đọc không ở focused editing session; **chưa biết AXSelectedTextRange/StringForRange của text đang edit trả gì**. Không lưu nội dung file user vào evidence.

## Call chain đã đối chiếu

### Fade

tray_state.rs dimmed = state != Live, áp dụng alpha45%. Suspended, disconnected, bypassed và inactive đều fade. Vì vậy hình icon không đủ phân biệt “tạm dừng toàn cục” với “bỏ qua app”. Tooltip có mô tả nhưng user vẫn không thấy lý do incident rõ ràng.

bridge/local.rs host_session đã authenticated kết thúc → latch.set(bridge_disconnected), service.native=None, suspended=true. Client session mất cũng set cùng reason. Socket timeout1s, helper tick100ms/lease0.5s, stale/progress/driver_fault/permission_required có đường latch. Đây là cơ chế source, **chưa chứng minh nhánh nào gây lần fade tự phát**. Cần diagnostic reason/timestamp không chứa text cho lần kế tiếp; log bridge chi tiết hiện dev-only.

### Figma canvas

ContextGuard.read yêu cầu đúng focused PID/element, không secure, AXSelectedTextRange collapsed và exact AXStringForRange cho đoạn owned. Snapshot chỉ dùng<80ms; StreamWindow tối đa8keys/120ms. EventDriver abandonWord reset và bypassWord=true tới delimiter; khác fault toàn driver. Khi AX canvas không expose range đúng, trả chậm hoặc đổi node trong lúc edit, có khả năng từ lúc chuyển dấu bị bỏ qua. Đây là **hypothesis**, chưa reproduce bằng controlled editing/attribute receipts.

Không kết luận bật screenreader sẽ sửa: notice chỉ gợi ý khác biệt canvas accessibility, không chứng minh bật tùy chọn có thể cung cấp API VXKey cần. Không tự bật hoặc sửa file user.

## Bước điều tra tiếp theo

1. Cần incident receipt production không chứa key/text: reason_code, timestamp monotonic/wall, component version/session, permissions, tap active, target app id, AX result class (role, selection/slice success/timeout, epoch/revision, không value). Hiển thị reason khi tray dim và copy diagnostics rõ ràng; không log toàn phím/AXValue/full field.
2. Controlled trial trên text Figma test do user chọn, so với rename/property/TextEdit cùng chuỗi và tốc độ; tách main fault/latch với proof-fail per-word. Plugin không cần để user gõ, nhưng không dùng thiếu plugin làm lý do sửa file thật qua đường khác.
3. Nếu xác nhận AX canvas không đủ: thiết kế strategy compatibility riêng có test và invariant rõ. Không bỏ ownership/selection proof, không spam Backspace dựa buffer nội bộ hoặc tự đổi inputsource để ép gõ.

Kết quả hiện tại: đã log2lỗi, tìm được đường source phù hợp, **chưa xác nhận root cause hoặc bản sửa**. App production và bản test giữ nguyên; no bundle/release. Track dài hạn MỞ, Windows PLAN ONLY.
