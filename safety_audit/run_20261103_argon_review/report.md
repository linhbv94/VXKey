# QA và safety review bản sửa Argon — 2026-11-03

## Kết luận

**Chưa nghiệm thu bản vá LOG_005; có finding safety cần sửa trước native typing QA.** Existing test gate PASS không phủ đường phát phím chỉ dựa vào empty proof. Không build/cấp quyền/chạy candidate mới; app đang có quyền tone_order_candidate giữ nguyên. Không sửa source trong lượt QA này, không gọi agent.

## Phạm vi

Working tree VXKey tại HEAD `26314079efe6908dfe890c6c81a754bc84117f99`, nhiều thay đổi tracked/untracked từ trước. Ba file Argon bàn giao đều là untracked so với HEAD nên không có diff chính xác riêng lượt Argon trong Git; đối chiếu report/packet trước và lưu [snapshot/hash](snapshot_hashes.json). Không gọi mọi vấn đề phát hiện là regression do Argon.

Đọc sâu ContextGuard, nhánh reset/verifyPending/ACTION_NONE/ACTION_SEND của EventDriver, StreamWindow, EditPlan và fixture mới. Tài sản cần bảo vệ: text đang nhập, selection/focus app khác, quyền đọc/phát phím OS. Trust boundary: state nội bộ composer/owned → AX proof → CGEvent rewrite. Áp dụng `_safe0_system`, `_safe5_diff_review`, `_safe8_desktop_security` vào boundary thay đổi này; không audit toàn bộ app/dependencies/clipboard/updater.

## Checks tự chạy

- `sh scripts/test_engine.sh`: PASS, **1200 Rust tests/doctests**, backend smoke, **77 Swift checks**, Settings integrity. [Log](engine_checks.log), [Rust](rust_tests.log), [Swift](swift_checks.json).
- Swift fixture riêng dùng **ContextGuard và EditPlan thật**, reader giả, main run loop thật: PASS tái hiện finding. [Source](main.swift), [kết quả](empty_proof_result.txt). Không gọi EventDriver.handle hay phát CGEvent; predicate ACTION_SEND được chép đúng từ source, vì vậy đây là fixture/source proof, chưa phải native data loss reproduction.
- Warning AX service connection invalid từ sandbox trong suite được lưu; không suy quyền app/native từ fixture.
- Native ChatGPT Cmd+A/delete và Shift+Enter, package upgrade/TCC retention: **chưa chạy**. Bản mới chưa đóng gói vì finding dưới đây.

## VXKEY_SAFE_001 — Empty probe được dùng để cấp phép rewrite chưa có proof của text

- Severity: **high** (integrity text đầu vào, có thể xóa/sửa nội dung không được xác minh); confidence: **high** cho source/predicate, chưa tái hiện thiệt hại native. Verdict: **confirmed** ở source/fixture.
- Vị trí: `native/macos_background/event_driver.swift:278–287`; `context_guard.swift` nhánh publish giữ empty snapshot và latest chỉ kiểm epoch/tuổi proof.
- Input/precondition: sau reset, empty probe đang bay; phím `d` đi qua ACTION_NONE tạo owned `d`; empty probe trả về trước probe xác nhận slice `d`; phím `d` thứ hai cần rewrite trong cửa sổ proof80ms/window120ms. Đây là trạng thái timing có thể xảy ra, không cần actor độc hại.
- Call chain: reset/expect empty → expectOwned(d) → publish empty (start=-1,text="",verified=true) → latest → resolvedStart lấy **caret** empty snapshot → guard chỉ kiểm verified và resolvedStart≥0 → EditPlan kiểm d trong **bộ nhớ**, không kiểm text đích → batch/tapPostEvent.
- Đối chứng fixture: ContextGuard thực trả empty snapshot khi nonempty d pending; predicate mới chấp nhận caret=5 và EditPlan(backspaces:1, output:đ) hợp lệ dù chưa có slice d được xác minh. Nếu foreground field/caret/selection thay đổi mà AX chưa cập nhật trước phím rewrite, proof này không bảo vệ ký tự thực bị Backspace.
- Phản biện: tuổi proof, epoch/PID, giới hạn pending và EditPlan đều vẫn có; chúng giới hạn rủi ro nhưng không chứng minh focus element/owned suffix đã hiện và khớp. Không khẳng định mọi lượt rewrite đang xóa sai, hay toàn bộ focus race là mới.
- So với trước: report Argon ghi nhánh cũ không vượt được guard khi empty caret=0 và owned=d. Nhánh mới chủ đích cho phép fallback caret; chưa có snapshot baseline source riêng để gán chính xác mọi regression.

## Hướng xử lý

Giữ các cải tiến defer/retry và unanchored suffix matching, nhưng **empty proof chỉ phục vụ bootstrap, không cấp phép destructive rewrite của nonempty owned**. Cần proof không rỗng cùng epoch/element khớp owned hoặc checkpoint không rỗng hợp lệ trong bounded stream; không đơn giản yêu cầu latest==owned cho mọi phím vì sẽ phá bounded speculation hợp lệ. Empty probe một mình không đủ. Nếu thiếu proof: fail-open an toàn hoặc thiết kế buffer chờ có giới hạn với invariant rõ, không xóa trước rồi mới kiểm.

Thêm regression thực exercise decision gate EventDriver (không chỉ ContextGuard): empty proof + owned d phải từ chối rewrite; verified d phải cho phép; stale empty probe không hồi sinh proof; đổi element/selection khi unanchored không dùng proof cũ; typing nhanh vẫn bounded; giữ prefix sau Cmd+A/Shift+Enter khi AX về đúng. Test hiện tại gọi việc giữ empty proof là PASS nhưng không thử gate destructive.

Các giải thích cụ thể về ProseMirror DOM/RPC trong report Argon là **giả thuyết**, chưa có native trace chứng minh từng nguyên nhân. Mức P0/Critical trong report không được kế thừa tự động. Chưa kết luận LOG_005 đã sửa từ fixture.

## Handoff

[Work log](../../../_summary/work_log.md), [log lỗi](../../typing_issue_log.md). Đây là QA của vòng sửa1; không tự tăng thành vòng sửa2, không reset bộ đếm. Mac1.0.0 theo user xác nhận sau khoảng1–2vòng vẫn giữ; chưa có lệnh release. Track dài hạn MỞ, Windows PLAN ONLY. Permission flow đầy đủ và update retention N→N+1 vẫn TODO, không được chứng minh bởi audit này.
