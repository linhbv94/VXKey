# Sửa thứ tự dấu trước horn — 2026-11-02

## Lỗi và thay đổi

Trace fixture user gửi có bước `đưo → đuợ → đuọw → đuọwc`. Browser không thấy phím bị tap chặn; suy luận `ddwojwc` được đối chiếu bằng C ABI trực tiếp, tái hiện cùng bước trước bản sửa. Không lấy capture giới hạn 200 events làm raw hardware log cho mọi dòng.

Core chuẩn hóa `ưo` thành `uơ` trước tone, sau đó coi `w` trên `uợ` là hoàn tác. Với Telex và TelexAdvanced, nhánh mới hoàn thiện horn cho `u` còn thiếu khi `uơ` cuối từ đã có tone. Giữ nguyên `uoww` escape khi chưa có tone, glide `qu`, VNI và tone style. Không thay native timing/pipeline trong lượt này.

## Kiểm chứng

- Regression trước sửa FAIL: `ddwojwc` thực tế `đuọwc`, kỳ vọng `được`.
- Sau sửa: `ddwojwc`, `ddwowjc`, `ddwowcj` → `được`; `mwojwt`, `mwowjt` → `mượt`; uppercase và hai tone styles, hai Telex modes PASS. `thuở`, `quở`, `uoww`, `ww` giữ đúng fixture.
- `sh scripts/test_engine.sh`: 1199 Rust tests/doctests PASS; backend isolated smoke PASS; 67 Swift assertions PASS; Settings source/CSS/copy integrity PASS. Fixture Swift có warning AX service trong sandbox; không coi đây là native permission/typing acceptance.
- C ABI test bổ sung: lặp 40 lần × 8 từ, mỗi mode 320 từ, cả hai modes PASS, gồm đặt dấu trước/sau horn, `casi`, `casch`, `cachs`.
- [Log toàn bộ](tone_order_verification.log), [C ABI regression](tone_order_ffi_tests.log), [artifact](tone_order_pair_artifacts.json), [build](tone_order_pair_build.log).
- Candidate `build/tone_order_candidate/desktop_candidate/VXKey.app` strict signature verification PASS; ad-hoc dev pins, không phải release identity hay retention proof. Data root riêng, config từ latency_candidate, marker suspended mới. Không ghi đè app đang có quyền.

## Phần còn mở

Native QA của binary mới chưa chạy; chờ quyền cho đúng binary trước Resume. Bản đang dùng vẫn latency_candidate. LOG_005 (Cmd+A/xóa), LOG_007 (khựng) và biến thể lỗi xen kẽ chưa được giải thích toàn bộ chỉ bằng bug core này. User gửi nguyên mẫu `đayas`; chưa có raw sequence đủ xác nhận nguyên nhân, không tự sửa lời báo lỗi. Theo dõi trong log.

Track dài hạn MỞ, chỉ user đóng. Windows PLAN ONLY. Không gọi agent, commit/push/release hay thay setting OS ở lượt sửa này.

## Kích hoạt native

User duyệt quyền đúng tone_order_candidate. Đã Quit latency_candidate, thay riêng entry VXKey qua Settings UI, switch ON. Helper mới xác nhận Accessibility/listen/post=true; Resume ACK applied, attached=true, suspended=false, fault=null. [Evidence](tone_order_native_evidence.json). Chờ user gõ tay trong ChatGPT; chưa đánh dấu native typing PASS.
