# Đối chiếu review Argon — VXKey

Ngày: 2026-11-02. Phạm vi: review chỉ đọc, không triển khai bản vá.

Đã gọi thật gemini-4-argon-high, effort high, mode plan; conversation 51fe611e-0bc6-4b73-b373-b98525f2c571. Agent đã nhận source snapshot và evidence, chỉ gọi read_file, không sửa nguồn hoặc gọi agent khác. Hai lần đọc AGENTS.md tại đường dẫn không tồn tại được ghi trong log; sau đó agent tiếp tục đọc source và trả kết quả. Một lượt review ban đầu, không follow-up, không tham vấn phụ.

[Báo cáo gốc](argon_review.md), [event log](argon_review_events.jsonl), [task/state](../tasks/argon_review_state.json), [packet](../tasks/argon_review_packet.md). CLI trả SUCCESS/exit0 nhưng báo cáo kết thúc giữa câu ở phần finding1; giữ nguyên bản gốc, không giả vờ có đủ danh sách8findings. Phần phân tích1–4 vẫn có nội dung để đối chiếu dưới đây.

## Kết luận đã đối chiếu

1. **Đã xác nhận bằng source và native log:** phím tiếp theo đến trước AX verification gọi fault(edit_unverified_before_next_key), tháo tap và làm đoạn sau mất chuyển dấu. Native trace185keydown/31rewrite/30verified/1unverified/1fault; unified log lúc16:17:53.622 xác nhận reason. [Cổng pending](../../native/macos_background/event_driver.swift:187).
2. **Xác nhận lỗi thiết kế từ source, chưa tái hiện riêng bằng fixture:** cả phím không rewrite cũng gọi expect(), xóa snapshot; phím kế không có proof sẽ reset composer. Vì vậy chỉ đổi fault thành reset vẫn có thể làm đứt âm tiết khi gõ nhanh. [Expect](../../native/macos_background/context_guard.swift:31), [gating](../../native/macos_background/event_driver.swift:223).
3. **Xác nhận race trong source:** expect() dùng try-lock rồi bỏ yêu cầu âm thầm nếu bận. State của EventDriver có thể đã đổi nhưng ContextGuard còn expected cũ. latest() miss là thiếu readback; expect() miss là mất cập nhật, không nên đánh đồng. [Lock](../../native/macos_background/context_guard.swift:33).
4. **Xác nhận thiếu invalidation trong source, nguy cơ cần tái hiện:** probe thoát sớm khi AX/focus/secure/selection không hợp lệ nhưng có thể để snapshot verified cũ còn được dùng trong cửa sổ freshness. Mouse/navigation reset đã che một số đường; thay đổi programmatic cùng PID vẫn cần fixture riêng. Chưa tuyên bố đã chứng minh xóa nhầm dữ liệu thật. [Probe](../../native/macos_background/context_guard.swift:45).
5. **Xác nhận thiếu cleanup trong source:** consumed key codes không được xóa khi teardown. Nếu keyUp diễn ra lúc tap đã tháo, trạng thái đó có thể lọt sang lần Resume/app kế tiếp. Hậu quả mất keyUp cần test; không khẳng định chắc chắn gây autorepeat/kẹt bàn phím vật lý. [Teardown](../../native/macos_background/event_driver.swift:82), [keyUp](../../native/macos_background/event_driver.swift:160).

## Các nhận xét không nhận nguyên xi

- Khoảng15–50ms, độ trễ AX và số dòng sửa40–60 trong báo cáo là ước lượng, không phải benchmark trên máy này.
- Snapshot.time được gán khi probe hoàn tất; không kết luận một probe dài hơn80ms tự làm snapshot mới bị quá tuổi ngay lập tức. Snapshot cũ có thể hết hạn trong lúc chờ probe, là vấn đề khác.
- Lỗi producerPID=0, Unicode chunk/WebView, modifier/private event source và smart punctuation là giả thuyết/case compatibility, chưa có native evidence tương ứng. Fixture trước đã xác nhận producerPID của events được tạo trên máy này.
- Bằng chứng nil/.null suppression chỉ áp dụng trial per-PID hiện tại, không khái quát tất cảmacOS/tap.
- Khuyến nghị optimistic ownership trong âm tiết chưa đủ chứng minh an toàn trước programmatic caret/focus/document changes. Không tự bỏ exact range proof hoặc đổi tất cả mismatch thành reset để báo đã sửa.
- Chưa cần thay kiến trúc tổng thể, bỏ Tauri/Funput hoặc quay lại Input Source chỉ từ các finding này. Phần cần thiết kế lại có trọng tâm là điều phối phím với AX verification.

## Yêu cầu gõ dấu linh hoạt

User xác nhận casch và cachs đều phải ra cách. Đã chạy thật lõi Rust qua C ABI: cả hai đạt. [Evidence](flexible_tone_core_evidence.json). Chưa phải native tốc độ cao acceptance. Mọi sửa đổi driver phải giữ hai thứ tự này, không bắt user gõ dấu ở vị trí duy nhất.

## Bước tiếp theo được đề xuất, chưa thực thi

1. Viết invariant rõ cho pipeline: không đánh đồng verification đang chờ với lỗi vĩnh viễn; không cho rewrite dùng proof đã bị thu hồi.
2. Cô lập state machine/AX scheduler bằng fixture có độ trễ, lock contention, thất bại probe và phím dồn; bảo toàn casch/cachs cùng văn bản ngoài vùng sở hữu.
3. Thiết kế thứ tự phím và invalidation trước khi chọn queue/stream strategy; xét app tự đổi selection/text, không chỉ keyboard/mouse.
4. Cleanup consumed theo vòng đời tap, đồng bộ revision/pending boundaries; sau đó mới build và native QA TextEdit/WebView, gõ nhanh/chậm/Undo/focus/secure fields.

User chỉ yêu cầu review, nên không sửa source, không build bản vá mới, không đổi quyền trong task này. App hiện tự suspended sau lỗi; icon mờ45% phản ánh trạng thái này, không phải engine đang xử lý bình thường.
