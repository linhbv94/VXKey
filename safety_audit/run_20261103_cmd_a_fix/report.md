# Vá VXKEY_SAFE_001 — 2026-11-03

## Thay đổi và kiểm chứng

Đưa quyết định cấp phép rewrite thành `ContextGuard.rewriteStart`, EventDriver gọi đúng hàm này trước EditPlan/batch. Nếu owned khác rỗng hoặc có Backspace, bắt buộc nonempty snapshot đã neo; không fallback caret của empty snapshot. Giữ kiểm PID/epoch/tuổi proof80ms và StreamWindow, giữ nonempty checkpoint cho bounded speculation. Đặt start=resolvedStart trước expectOwned, tránh bỏ quên anchor vừa xác minh. Empty bootstrap chỉ còn cho insertion đầu từ không xóa, giữ leading-w. Không đổi Rust, permission cache, OS setting hay app đang có quyền.

- 86 Swift assertions PASS, gồm 9 checks mới dùng gate production: empty proof bị chặn dù start số hợp lệ, verified d được nhận, bounded speculation còn hoạt động, sai PID/epoch, hết tuổi proof và key cap bị từ chối. [Evidence](swift_checks.json), [log](../../docs/qa/cmd_a_safety_fix_tests.log).
- [Reproduction sau vá](main.swift) dùng ContextGuard thật + gate production với reader giả: cùng empty proof + d pending nay bị từ chối. [Kết quả](result.txt). Không phát OS input.
- Full gate1200Rust +77Swift đã tự chạy trước vá trong audit; lượt này Rust không thay đổi nên chỉ chạy lại native fixture có86assertions và Settings integrity. Candidate build cũng compile toàn helper/desktop.
- [Candidate artifacts](../../docs/qa/cmd_a_safe_pair_artifacts.json), [build log](../../docs/qa/cmd_a_safe_pair_build.log): pair riêng cmd_a_safe_candidate, suspended; strict signature PASS. Dev ad-hoc pins, chưa phải identity ổn định/permission retention.

**VXKEY_SAFE_001 đã sửa và xác minh ở source/fixture, chưa native runtime.** Chưa khẳng định hết mọi focus/selection race hay LOG_005 đã sửa. Khi AX chậm hơn phím dấu thứ hai, driver vẫn fail-open nguyên từ; không đổi thành đợi/buffer trong callback. Retry2ms/unanchored matching của Argon được giữ để tăng cơ hội có proof đúng. Cmd+A/delete và Shift+Enter vẫn cần physical typing QA trên binary mới.

User yêu cầu sửa sau audit; đây là **vòng sửa2** sau chỉ đạo hướng tới1.0.0, không phải lệnh release. Chờ user xác nhận mới release Mac trước; track dài hạn MỞ, Windows PLAN ONLY. Không gọi agent/commit/push/release hoặc thay TCC ở lượt vá source này.
