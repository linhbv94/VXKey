# AGENTS.md — Quy tắc riêng VXKey

Version: 1.1.0
Ngày update: 2026-11-02

Áp dụng cùng quy tắc global và [quy tắc workspace](../.agents/AGENTS.md). Đây là chỉ đạo trực tiếp của user dành riêng VXKey.

## Track bảo trì dài hạn — chỉ user được đóng

- VXKey có một track bảo trì dài hạn trong [work log tổng](../_summary/work_log.md), được mô tả ở [maintenance_track.md](maintenance_track.md).
- **Agent không có quyền tự đóng, đánh dấu hoàn tất hoặc loại bỏ track dài hạn này. Chỉ user được đóng bằng yêu cầu rõ ràng.**
- QA đạt, release thành công, xong subtask, checkpoint, hết lượt chat/limit hoặc chuyển agent không đóng track. Quy tắc đóng turn tự động của workspace không áp dụng cho track dài hạn VXKey.
- Agent được đánh dấu hoàn thành từng subtask có bằng chứng, ghi checkpoint hoặc tạm dừng theo yêu cầu user; track dài hạn vẫn mở. Không cần giữ process chạy hoặc tự tạo automation để biểu thị track còn mở.
- Giữ anchor VXKey trong work log khi rút gọn lịch sử hoặc dồn track. Có thể rút gọn các lượt cũ, nhưng phải giữ link track/log lỗi/checkpoint hiện hành và trạng thái chỉ user được đóng.
- Theo dõi lỗi qua sử dụng thực tế dài ngày; không tự coi một lượt gõ đúng hoặc fixture/build/QA đơn lẻ là ổn định lâu dài.

## Phạm vi effort và chi phí

Track dài hạn **không đồng nghĩa effort cao**. Chọn effort/model theo tác vụ và chỉ đạo hiện hành. Việc user từng gọi Argon review không phải quyền gọi model cao cấp thường trực. Không tự gọi agent, chạy nền liên tục hoặc mở rộng phạm vi chỉ vì track chưa đóng.

## Nhật ký và điểm tiếp tục

- [Work log tổng](../_summary/work_log.md): nguồn tiến độ/bàn giao chung.
- [Track dài hạn](maintenance_track.md): trạng thái bền vững và việc còn mở.
- [Nhật ký lỗi gõ](typing_issue_log.md): nơi user ghi lỗi trong quá trình dùng.
- Mỗi lần bàn giao phải phân biệt source/fixture/native trial với trải nghiệm dùng dài hạn. Không tự sửa lời báo lỗi của user thành một lỗi đã tái hiện hoặc đã sửa.


## Windows — chỉ lập plan, chưa được triển khai

User chốt ngày2026-11-02: [Windows adaptation plan](windows_adaptation_plan.md) là PLAN ONLY. Chỉ bắt đầu code/prototype/build/gọi agent hoặc cài app Windows khi user ra lệnh triển khai rõ ràng sau đó. Không suy quyền này từ mục tiêu sớm có hai platform hoặc từ việc track bảo trì còn mở. [Milestones](docs/tasks/windows_adaptation_plan.json) giữ execution_authorized=false cho đến chỉ đạo mới.

## Lịch sử cập nhật

- 1.1.0 (2026-11-02): Windows PLAN ONLY; execution requires a subsequent explicit user instruction.
- 1.0.0 (2026-11-02): Theo chỉ đạo user, đặt track VXKey dài hạn, chỉ user được đóng; phân biệt với subtask/checkpoint và effort/model.
