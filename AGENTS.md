# AGENTS.md — Quy tắc riêng VXKey

Version: 1.6.0
Ngày update: 2026-11-05

Áp dụng cùng quy tắc global và [quy tắc workspace](../.agents/AGENTS.md). Đây là chỉ đạo trực tiếp của user dành riêng VXKey.

## Track bảo trì dài hạn — chỉ user được đóng

- VXKey có một track bảo trì dài hạn trong [work log tổng](../_summary/work_log.md), được mô tả ở [maintenance_track.md](docs/maintenance_track.md).
- **Agent không có quyền tự đóng, đánh dấu hoàn tất hoặc loại bỏ track dài hạn này. Chỉ user được đóng bằng yêu cầu rõ ràng.**
- QA đạt, release thành công, xong subtask, checkpoint, hết lượt chat/limit hoặc chuyển agent không đóng track. Quy tắc đóng turn tự động của workspace không áp dụng cho track dài hạn VXKey.
- Agent được đánh dấu hoàn thành từng subtask có bằng chứng, ghi checkpoint hoặc tạm dừng theo yêu cầu user; track dài hạn vẫn mở. Không cần giữ process chạy hoặc tự tạo automation để biểu thị track còn mở.
- Giữ anchor VXKey trong work log khi rút gọn lịch sử hoặc dồn track. Có thể rút gọn các lượt cũ, nhưng phải giữ link track/log lỗi/checkpoint hiện hành và trạng thái chỉ user được đóng.
- Theo dõi lỗi qua sử dụng thực tế dài ngày; không tự coi một lượt gõ đúng hoặc fixture/build/QA đơn lẻ là ổn định lâu dài.

## Phạm vi effort và chi phí

Track dài hạn **không đồng nghĩa effort cao**. Chọn effort/model theo tác vụ và chỉ đạo hiện hành. Việc user từng gọi Argon review không phải quyền gọi model cao cấp thường trực. Không tự gọi agent, chạy nền liên tục hoặc mở rộng phạm vi chỉ vì track chưa đóng.

## Nhật ký và điểm tiếp tục

- [Work log tổng](../_summary/work_log.md): nguồn tiến độ/bàn giao chung.
- [Track dài hạn](docs/maintenance_track.md): trạng thái bền vững và việc còn mở.
- [Nhật ký lỗi gõ](docs/typing_issue_log.md): nơi user ghi lỗi trong quá trình dùng.
- Mỗi lần bàn giao phải phân biệt source/fixture/native trial với trải nghiệm dùng dài hạn. Không tự sửa lời báo lỗi của user thành một lỗi đã tái hiện hoặc đã sửa.


## Windows — được triển khai riêng engine, giữ nguyên macOS

User đã thay chỉ đạo PLAN ONLY bằng lệnh rõ ràng ngày2026-11-04: làm engine Windows, bảo toàn tuyệt đối bản Mac, chạy các QA có thể làm trên Mac; kiểm thử native Windows để khi có máy. [Plan](docs/windows_adaptation_plan.md), [milestones](docs/tasks/windows_adaptation_plan.json) và [báo cáo engine](docs/qa/windows_engine/implementation.md) là điểm tiếp hiện hành.

- Engine mới nằm riêng tại `platforms/windows_engine`, có workspace/lockfile riêng. Không sửa shared Funput, adapter/runtime Mac, workflow/bundle production hoặc quyền OS để port Windows.
- Phân biệt portable fixture, MSVC target typecheck, linked EXE, native trial và installer/update. Compile check trên Mac không phải Windows native PASS.
- Phạm vi lệnh hiện tại là engine; không tự suy quyền push/CI dispatch/publish/release/cài app, gọi agent hoặc thay production. Full Tauri/helper integration theo gate của plan, không vượt native proof bằng một báo cáo source.

### Bàn giao sang Windows — 2026-11-04

User đã mở máy Windows, pull workspace và yêu cầu cập nhật hướng dẫn rồi push code để tiếp tục tại đó. Lượt bàn giao này được commit/push riêng engine Windows, AGENTS và docs/work log VXKey đến các remote origin hiện có; không là quyền release/tag hoặc sửa Mac. Hướng chỉ dev trên Mac/test bằng app đóng gói đã được user thay bằng tiếp tục trực tiếp trong workspace Windows.

Agent trên Windows đọc work log tổng và [Windows handoff](docs/qa/windows_engine/windows_handoff.md); nếu symlink `docs` không có thì đọc canonical `../_summary/docs/vxkey/qa/windows_engine/windows_handoff.md`. Kiểm tra môi trường thực rồi build/link GUI Test, probe trước engine thật. Không chạy shell Slint cũ ở `platforms/windows`, không suy native PASS từ37 portable tests. Track dài hạn vẫn MỞ.

### Tiếp tục app Windows có tray — 2026-11-04

Sau native trial cơ bản PASS, user giao hoàn thiện app/tray để quay Mac release và xác nhận triển khai bộ cài per-user không admin. Scope hiện hành gồm Windows Tauri owner/helper, tray, Settings/config, startup paused và NSIS Test; được sửa nhánh `cfg(windows)` của companion và UI platform routing cần thiết. Bảo toàn hành vi Mac, native/shared engine, root Cargo/lock, signer và workflow Mac. Default JSON được chuyển thành runtime resources có nội dung byte-for-byte như examples để Windows không cần docs private khi compile.

Windows child dùng anonymous inherited pipes riêng thay cho named pipe listener đề xuất: bounded frames/session/sequence/PID, Job Object kill-on-close trước bootstrap hook. Không expose control listener hay raw keyboard/text qua IPC. Bản cài `VXKey Test`, namespace/data riêng, updater production tắt; không tự coi là release production hoặc full native matrix PASS. [Báo cáo app Windows](docs/qa/windows_app_20261104/report.md) là entry point mới. Không suy quyền publish/tag/dispatch từ lệnh hoàn thiện local; chỉ user đóng track dài hạn.

### Chuẩn bị đem sang Mac release — 2026-11-05

User xác nhận bản cuối gõ OK và giao tiếp tục tự chủ khi AFK đến khi sẵn sàng đem sang Mac release. Phạm vi gồm profile production Windows, workflow build/merge Windows vào draft Mac, gate publish kết hợp và commit/push riêng VXKey + docs để Mac pull; không tạo tag, dispatch hoặc publish trong lượt chuẩn bị này. `windows-production` phải đi cùng `tauri.windows.release.conf.json`; Test và production tách ID/data/singleton/startup. Không cài production lên máy để thay Test đã nghiệm thu.

Native Mac, shared Funput, signer/key và root Cargo/lock tiếp tục được bảo toàn. Sửa plumbing release tối thiểu được ghi riêng: default JSON lấy từ code resource nguyên trạng thay docs private; thiếu file notes private trên CI thì tạo draft với generated notes để user review; publish Mac-only giữ gate cũ, Windows là lựa chọn bổ sung có run ID riêng. [Handoff Mac release](docs/qa/windows_app_20261104/mac_release_handoff.md) liệt kê bước cần chạy trên Mac/CI và phần chưa được kiểm chứng ở Windows.

## Lịch sử cập nhật

- 1.6.0 (2026-11-05): Ghi nhận lệnh AFK/hoàn thiện release handoff, tách Test/production Windows, build/manifest/publish gates và plumbing code-only CI.
- 1.5.0 (2026-11-04): User mở scope app/tray Windows và NSIS per-user sau native cơ bản; ghi owner/helper/private pipe, Test isolation và ranh giới Mac/release.
- 1.4.0 (2026-11-04): User chuyển tiếp sang workspace Windows và cho commit/push code+docs bàn giao; thêm entry point và thứ tự native QA, giữ bảo toàn Mac/release boundary.
