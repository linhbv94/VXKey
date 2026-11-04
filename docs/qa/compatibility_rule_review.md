# Rà và giảm điều kiện loại use case — 2026-11-03

## Chỉ đạo và kết luận

User không chấp nhận giải pháp riêng Figma trong khi nhiều editor khác có thể gặp cùng vấn đề. Đã bỏ đặc cách bundleID/role và sửa việc bỏ cả từ khi chỉ thiếu proof tạm thời. Đây là sửa source/fixture cho toàn bộ editor có cùng khả năng, không phải whitelist thêm app. Không đổi setting app ngoài hoặc engine core.

## Điều kiện đã bỏ/đổi

| Điều kiện cũ | Thay đổi | Phạm vi |
| --- | --- | --- |
| Thiếu subrole chỉ nhận AXTextArea, rồi đặc cách Figma AXDocument/WebArea | Bỏ toàn bộ role/app filter. Subrole thiếu/unsupported vẫn được thử selection và exact slice API | Mọi app/editor có API tương ứng |
| ACTION_SEND thiếu proof một lần → abandonWord/bypass tới Space | Giữ observed text và logical composer riêng; tiếp tục pass-through, chờ checkpoint; khi đủ proof thì reconcile vùng owned và có thể chuyển dấu ngay trong từ | Không chặn callback, không cần đổi app setting |
| Delta engine luôn giả định rewrite trước đã được áp dụng | Reconcile dùng common scalar prefix của chữ thực quan sát và target logic; Backspace chỉ xóa suffix của vùng owned | Khôi phục sau phím raw do AX chậm |

`context_guard.swift` dùng `allowsSubrole(status,subrole)` thay supportsUnspecifiedSubrole(role,appID); không còn com.figma trong driver. Role chỉ là metadata, không quyết định có gõ được. Lỗi RPC thật vẫn không được giả thành xác minh thành công.

`event_driver.swift`: deferredRendered giữ kết quả core đang chờ, owned giữ đúng raw/injected đã gửi tới editor. Một lần thiếu proof không clear composer hoặc khóa cả từ. Rewrite chỉ khi cổng ContextGuard.rewriteStart hiện có cho phép. Reset/epoch/focus/navigation/modifier/Backspace hủy deferred state để không phát sửa muộn sang vùng khác. Sau ranh giới từ, không sửa hồi tố từ trước khi proof vẫn chưa đủ. Quá128UTF16/streamtimeout/keycap vẫn fail-open.

`edit_plan.swift`: reconcile scalar suffix qua make hiện có, giữ guard combining/surrogate/owned size/NUL/output bounds. Không mở rộng quyền đọc full field và không thêm blind Backspace.

## Các điều kiện giữ và lý do

- Secure Event Input/secure subrole, OS permission, PID/focus/epoch, selection, exact suffix: giữ để tránh sửa/xóa vào ô khác, vùng chọn hoặc mật khẩu.
- User chọn EN/bypass hoặc Pause: giữ vì đó là ý định user.
- Loại own utility khỏi driver: giữ để tránh tự xử lý UI/control; core sandbox đã ghi rõ riêng, không giả native test.
- Giới hạn số phím pending/tuổi proof/độ dài: đây là **tuning của implementation, không phải hằng số phổ quát của bộ gõ**. Chưa nới số tùy tiện vì chưa có native receipts chứng minh ngưỡng nào gây lỗi; không dùng test vài từ để khẳng định các ngưỡng phù hợp mọi editor.

## Gap kiến trúc chưa giải quyết

Editor không cung cấp selection/range/slice usable vẫn chưa có strategy native thay thế. Bỏ filter role không tự tạo ra API còn thiếu. Đã xác định đây là **gap tương thích của VXKey**, không phải điều kiện yêu cầu user sửa app hoặc whitelist hàngnghìn ứng dụng. Cần native matrix + strategy input tương thích khác để đạt coverage rộng hơn; không tuyên bố universal support sau patch này. LOG_008/009 và LOG_005 vẫn MỞ.

Không bỏ guard ownership đã vá VXKEY_SAFE_001 chỉ để demo có dấu. Deferred state chỉ phục hồi khi proof đến trong bounded window, không cấp phép rewrite bằng empty snapshot hoặc nội dung nhớ trong core.

## Kiểm chứng

`sh scripts/test_background_driver.sh`: **102Swift checks PASS**, compile real driver/context/editplan +RustFFI. [Log](compatibility_rule_review_tests.log), [evidence](compatibility_rule_review_evidence.json). Gồm missing subrole admission, secure/RPC fail rejection, missing exact slice, empty-proof/PID/epoch/age/keycap guards và nhiều thời điểm proof chậm cho ddwojwc/casch/cachs/mwojwt, scalar/combining boundary.

Reconciliation fixtures dùng core thật và planner production, **không giả là EventDriver callback hay native app trial**. Chưa build bundle, chưa gõ thật Figma/editor khác; không thay app đang có quyền. Warning AX connection trong sandbox vẫn lưu ở log. Rust core/service không đổi trong lượt này, không chạy lại fullRust suite. Không versionbump/commit/push/release. Track bảo trì MỞ, Windows PLAN ONLY.
