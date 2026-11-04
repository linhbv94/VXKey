# VXKey — baseline khựng và permission-cache candidate

Ngày2026-11-02. Source/fixture/build verified; native acceptance pending.

User báo LOG_005 Cmd+A/xóa trong ChatGPT mất dấu từ đầu, LOG_006 w sau đ không nhất quán, LOG_007 cảm giác khựng. [Log](../../typing_issue_log.md) vẫn MỞ. [Core evidence](restart_word_core_evidence.json) đúng với các chuỗi thử, không phải native timing.

[Sample word_candidate3giây/10ms](word_engine_idle_sample.txt) thấy main/tap run loop chờ permissionStatus→TCCAccessPreflight→sync XPC/mach wait. Đây là stack baseline không đồng bộ với chuỗi gõ đo timing. Không có số input-to-paint latency hoặc kết luận mọi khựng do TCC.

## Bản vá có giới hạn

[PermissionCache](../../native/macos_background/permission_cache.swift) đọc AX/listen/post trên worker, tối đa1job, định kỳ1giây. Main đọc cache; snapshot chưa có/quá2,5giây tính từ đầu query thì ready=false. Không synchronous refresh ở getter. OS tự thực thi quyền; cache không phải auth bypass, không phải retention proof.

[EventDriver](../../native/macos_background/event_driver.swift) dùng cached ready/status, kiểm enabled ở tick/transition một lần rồi status dùng cache. [ContextGuard](../../native/macos_background/context_guard.swift) bỏ AXIsProcessTrusted preflight từng probe, chính AX RPC vẫn chịu OS permission và failure invalidates proof.

Không đổi Rust/Telex hoặc bỏ range proof/ngưỡng speculation. LOG_005/006 chưa resolved. Bản này loại một nguồn chặn run loop có evidence; chưa chứng minh hết lỗi first-word/selection/w hay khựng.

## Checks và candidate

[67Swift assertions](background_edit_evidence.json), [log](permission_cache_tests.log) PASS: worker TCC giả block thì getter vẫn trả; reader off-main; cache chưa khởi tạo/quá hạn fail closed, cùng suite composer/checkpoint/prefix cũ. Không post input thật. Không chạy lại full Rust vì không sửa Rust;1198PASS trước là lịch sử.

[SDK27 build](latency_pair_build.log), [hashes](latency_pair_artifacts.json), strict signatures PASS. Candidate riêng build/latency_candidate/desktop_candidate/VXKey.app; config26 từ word_candidate, probe OFF/autostart OFF/tone modern. Vì root word_candidate healthy không còn recovery marker, tạo marker QA mới qa_candidate_unverified để bản mới khởi động suspended; không sửa root cũ.

## Native gate

Approval đúng binary nếu thay entry Accessibility; Quit bản cũ, thay riêng VXKey, mở/Resume. User ChatGPT gõ dở→Cmd+A/xóa→ddwowcj/casi, w sau đ/câu dài; TextEdit là track riêng. Sample lại cùng điều kiện để xác nhận TCC không ở main. Nếu còn khựng, cần timing aggregate không raw keys/text và profile AX request rate/target app.

MKey/layout/PressAndHold/autostart giữ nguyên. Track dài hạn mở; Windows PLAN ONLY. Không agent/commit/push/release.
