# VXKey — triển khai checkpoint pipeline

Ngày: 2026-11-02. User đã yêu cầu triển khai sau review. Không gọi thêm Anti; task Argon trước chỉ review, không sửa nguồn.

## Đã thay đổi

- [StreamWindow](../../native/macos_background/stream_window.swift): theo dõi revision chưa xác minh, ACK cũ chỉ xác nhận phần đầu, giữ phần phím mới hơn. Giới hạn8key/120ms.
- [ContextGuard](../../native/macos_background/context_guard.swift): mailbox main-thread và tối đa một AX worker in-flight; không try-lock làm mất expectation. Coalesce revision mới. Proof invalid sticky trong epoch; không publish worker epoch cũ hoặc đè checkpoint mới bằng kết quả cũ. Snapshot age tính từ đầu probe; waiting không làm mới tuổi proof.
- [EventDriver](../../native/macos_background/event_driver.swift): không global-fault chỉ vì phím mới tới trước checkpoint. Cho phép tiến hữu hạn theo proof cũ còn mới; hết cửa sổ/mất proof thì bỏ ownership và chuyển nguyên phím đến hết từ. Hook/engine/bridge lỗi vẫn suspend. Dọn consumed/gesture khi teardown. Backspace retone vẫn cần exact proof.
- [Runtime settings](../../native/macos_background/runtime_settings.swift): config chờ cả các revision chưa xác minh; không apply lại cùng revision liên tục. Quay lại external app từ UI VXKey reset epoch kể cả PID không đổi.
- [Spec10](../spec/10_checkpoint_pipeline.md) thay chính sách per-key-fault cũ ở spec09. Không thay engine Funput, UI, shortcut mặc định, mode icons, autostart hoặc Input Source.

## Kiểm tra thực sự đã chạy

- [49 Swift assertions](background_edit_evidence.json), [log](checkpoint_driver_tests.log): real Rust composer/EditPlan, suppression fixture, StreamWindow cap/expiry/late ACK, coalesce20expectations khi AX giả bận25ms, invalid proof sticky/epoch reset/stop; casch/cachs đều cách.
- [Real Swift settings/timer + Rust bridge fixture](checkpoint_bridge_tests.log):3Resume/Pause transactions và focus-prime/own-UI-return assertions; OS driver/permission mock. Lượt đầu host không khởi động trong sandbox; đã rerun fixture với quyền socket cần thiết và PASS. Không lấy lần lỗi đầu làm PASS; stderr host được bổ sung để lần sau không chỉ báo JSON decode.
- [SDK27 native + desktop build/canonical Settings/typecheck](checkpoint_pair_build.log), [exact pair hashes](checkpoint_pair_artifacts.json). Candidate riêng, ký ad-hoc theo workflow QA cũ; không production retention proof.
- Không chạy full1197Rust tests vì core Rust không sửa. Không lấy số kiểm tra lịch sử làm số mới.

## Candidate và bước tiếp

Candidate: build/checkpoint_candidate/desktop_candidate/VXKey.app. QA root riêng, copy config revision26 và recovery suspended từ null_candidate; không có driver_probe_mode nên composer thật. Không sửa binary đã có quyền. Chưa mở hoặc cấp quyền candidate mới.

Cần approval đúng binary nếu thay riêng entry VXKey trong Accessibility. Sau đó kiểm tra AX/PostEvent, Resume và user gõ tay TextEdit foreground: tieengs Vieetj; casch/cachs; câu dài tốc độ thường/nhanh; Space/Enter/Tab sau dấu. Pause chốt counters. verified bây giờ là số checkpoint tiến được, không yêu cầu bằng số rewrites vì có ACK coalesce.

Native focus/selection/secure/Undo/keyup và WebView vẫn cần kiểm chứng. Cửa sổ proof hữu hạn chưa tạo application ACK atomic; programmatic text/caret/focus thay đổi giữa probe và phím vẫn là giới hạn cần native QA, không tuyên bố an toàn mọi app. AX quá trễ có thể chuyển nguyên một từ thay vì tự tắt toàn bộ bộ gõ.

Autostart OFF, MKey/layout/app khác giữ nguyên; không install Applications, Input Source, signer/keychain/trust/TCC database, commit/push/release.
