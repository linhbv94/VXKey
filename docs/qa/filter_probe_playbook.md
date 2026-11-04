# Native event filter contract probe

Ngày 2026-11-02. Scope: QA-only, không phải bản sửa typing đã được xác nhận.

- Candidate: `../../build/filter_candidate/desktop_candidate/VXKey.app`; [artifacts](filter_pair_artifacts.json), [build](filter_pair_build.log), [tests](filter_probe_tests.log).
- Bật probe bằng file `build/filter_candidate/qa_integration_data/driver_probe_mode` chứa `filter`, đọc một lần khi helper khởi động và chỉ khi VXKEY_BRIDGE_DEV_TRUST=1. Không cần sửa binary/pins để trở về composer: Quit, bỏ file probe, mở lại cùng candidate; vẫn explicit Resume.
- Chỉ hành động với TextEdit foreground có AX collapsed/fresh proof. Mode này không gọi composer, không AX write, không clipboard, không synthetic posting. z trả nil, x đổi Unicode của event trả về thành X; phím khác chuyển nguyên. Cmd/Ctrl/Option vẫn bypass trước probe. Autostart OFF, marker paused từ trial cũ được giữ.
- User click foreground TextEdit Untitled6 thật, dòng mới và gõ tay `azx` không modifiers. Mong đợi `aX`. Sau đó native Pause để flush counters.
- Nếu vẫn `azx` dù probe decisions/callback đã chạy: kiểm chứng per-PID active-filter contract đang không đạt; không quay lại sửa Funput trước khi xác định tap phù hợp. Nếu `aX`: filter contract đạt cho trial này; cô lập batch replacement/AX ownership tiếp, chưa có typing acceptance.
- Không suy trường hợp OS/app/Windows khác từ probe TextEdit này. Không ghi user key/text vào runtime log; chỉ fixture/user-report được ghi trong report.

## Null-event probe - 2026-11-02

User: azx -> azX. Native Pause ACK. In-place mutation works; nil did not suppress z in this trial. Whole-session counters seen149/decisions10/fault0 are not isolated three-key counts.

New candidate: `build/null_candidate/desktop_candidate/VXKey.app`. Suppress now mutates original event.type=.null and returns that event, including consumed keyDown/keyUp, shortcut and both rewrite paths. [Artifacts](null_pair_artifacts.json), [build](null_pair_build.log), [32 fixture checks](background_edit_evidence.json), [log](null_filter_tests.log). Native .null behavior NOT YET VERIFIED.

After exact binary permission approval: Quit old app, replace only VXKey entry, open/Resume new app in filter mode. User foreground TextEdit azx must produce aX. Pause and read counters. If PASS: Quit, remove only private QA file `build/null_candidate/qa_integration_data/driver_probe_mode`, reopen SAME binary and Resume for manual Telex. No rebuild or new permission entry between modes. If FAIL: stop composer trial and investigate tap suppression further.
