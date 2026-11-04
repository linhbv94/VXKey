# VXKey — track bảo trì dài hạn

Ngày thiết lập: 2026-11-02.
Trạng thái: **MỞ DÀI HẠN — CHỈ USER ĐƯỢC ĐÓNG**.
Người có quyền đóng: **user**.
Nguồn chỉ đạo: user muốn theo dõi chất lượng gõ qua quá trình sử dụng dài, agent không được tự đóng.
Effort/model: **theo từng tác vụ, không mặc định cao**.

## macOS 1.0.0 đã phát hành — tiếp tục bảo trì

User chốt 2026-11-02: sau khoảng 1–2 vòng sửa nữa, **khi user xác nhận thì release 1.0.0 macOS theo trạng thái lúc đó, kể cả còn lỗi**, rồi fix dần. Chưa phải quyền release ngay. [Checklist và bộ đếm vòng sửa trong work log](../_summary/work_log.md) là nguồn theo dõi; không đặt lại khi đổi agent/chat.

Ưu tiên luồng xin quyền đầy đủ và giữ quyền qua auto-update (cần proof N → N+1 thật), tiếp tục lỗi chữ đầu sau Cmd+A/xóa và **Shift+Enter**. Không khẳng định lỗi chữ đầu do quyền. Mac release trước, Windows vẫn PLAN ONLY; release không đóng track dài hạn.

## Cách vận hành

Track này tồn tại qua các lượt chat, checkpoint, bản build, QA và release. Agent có thể hoàn thành subtask có bằng chứng; không được suy ra track dài hạn hoàn tất. Khi user bảo checkpoint/dừng, dừng thực thi và ghi điểm tiếp, không đóng track. Không tự tạo automation, background job hoặc gọi agent liên tục.

Khi work log tổng được gọt lịch sử hoặc dồn track, giữ một anchor VXKey dẫn tới file này, log lỗi và checkpoint mới nhất. File này không thay thế work log tổng bằng một nhật ký lịch sử song song.

## Release checkpoint 2026-11-03

User đã xác nhận phát hành. [VXKey1.0.0 public](https://github.com/linhbv94/VXKey/releases/tag/v1.0.0) cho Mac Apple Silicon/macOS26+, app/helper ký certificate riêng và updater có kiểm signature. CI/package/public gates đạt; [release state](docs/qa/release_1_0_0_state.json). LOG_005 và native typing dài hạn còn mở; giữ quyền qua signed updater N→N+1 chưa kiểm. Windows PLAN ONLY, track dài hạn không đóng.

## Điểm tiếp tục hiện tại

- **Bàn giao user tự giao Anti2026-11-04:** [anti_next_handoff.md](anti_next_handoff.md), đã link nổi bật trong work log; Codex chưa gọi Anti. LOG_010 source đãvá, LOG_011 khóhơn còn cần delivery/nativeQA; không đổi frozen bundle hoặc production.
- [Work log tổng](../_summary/work_log.md): nguồn tiến độ hiện hành.
- **Integration2026-11-04:** [Report/source/build/playbook](docs/qa/session_adapter_integration.md), [artifacts](docs/qa/session_adapter_artifacts.json). P2parity source/fixture, P3control continuity mock và P4Test app đã có; native trial còn mở. Test đã native PASS corpus/macro/TextEdit boundary/exclusion/Pause/Resume/Quit; Illustrator user báo đang ổn. Reopen giữ quyền nhưng phát hiện LOG_010 startup latch giả, source/fixture đã sửa, chưa bundle. Toggle/basic Backspace userPASS, Z user báo đúng; flip F5 test mở completion (LOG_011 còn mở, chưa sửa). Default driver/production chưa đổi; v6 đãgrant giữ nguyên.
- **MAC_ADAPTER_002, 2026-11-04:** chọn `session tap + Funput local + return_original` để triển khai tiếp dựa trên native trial v6. [Spec11](docs/spec/11_mac_driver_replan.md) có evidence matrix và backlog MAC_ADAPT_01…05. Đây là hướng được chọn, chưa tích hợp/promote production; nợ parity/lifecycle/exclusion/quyền vẫn có gate riêng.
- [Review và kế hoạch adapter macOS2026-11-03](docs/spec/11_mac_driver_replan.md): user đã giao triển khai, [P0/P1 source/bundle và native trial](docs/qa/session_spike_review.md) đã có. Session tap + core local, không AX readback/bridge lease trong đường gõ thử. P1/basicP2 PASS; chưa promote production/full acceptance.
- [Nhật ký lỗi gõ](typing_issue_log.md): không đóng LOG_005/007/008/009. User báo Figma canvas `â` được nhưng `ngaan` chưa đúng, và có lúc tự fade/Resume; chưa coi là lỗi core đã tái hiện.
- Snapshot lịch sử trước spike: `build/compatibility_test_candidate/desktop_candidate/VXKey.app` từng được cấp quyền/Resume nhưng Figma FAIL. [Evidence](docs/qa/compatibility_test_evidence.json). Không dùng trạng thái process/quyền hay candidate lịch sử làm mặc định.
- Baseline trước spike có50test Telex source PASS, gồm tone-order `ddwojwc`; mới nhất có484fixturechecks và user native trials của v6 như dưới. Giữ `tone_style = modern`, `w→ư` và `casch/cachs→cách` làm hồi quy.
- **P1 pilot native PASS + basic P2 PASS:** **v6** `build/session_spike_candidate_v6/vxkey_session_spike.app` dùng returned original event + synthetic Backspace, giữ frozenbytes/path. User xác nhận TextEdit/Figma Telex, Enter/Shift+Enter/Cmd+A→xóa và prefix/emoji/suffix/selection đúng. V2/v3 post-pair/private-source đơn lẻ FAIL; v4/v5 traces đã khoanh core đúng, route cũ vẫn mất chữ. [Evidence](docs/qa/session_spike_evidence.json), [report/playbook/giới hạn](docs/qa/session_spike_review.md), [artifacts](docs/qa/session_spike_v6_artifacts.json).
- **Cleanup:** v6 đã Pause/Quit, tap_attached=false; MKey đã mở lại, không thay mode/setting/quyền. Production/candidate cũ không đổi bytes/TCC hoặc launch. P2 còn ma trận/parity/long output/ACTION_RESTORE, P3 lifecycle, P4 config/exclusion/permission UI integration còn mở. Không coi prototype là app hằng ngày hoặc production fixed; Windows PLAN ONLY, chưa release. Các checkpoint phía dưới là lịch sử.

## Điều kiện trước khi gọi một subtask đã đạt

Ghi phạm vi, bản build/hash, cách thử, evidence và phần chưa kiểm chứng. Các kiểm tra core/fixture không thay native typing. Một câu gõ đúng không thay sử dụng liên tục nhiều ứng dụng. Không tự tăng effort hoặc đổi model vì còn backlog.

## Quyền đóng

Chỉ thay trạng thái track khi có yêu cầu rõ ràng từ user về việc đóng **track dài hạn VXKey**. Không suy từ “xong việc này”, “QA đi”, “release”, “checkpoint” hoặc im lặng. Không cần hỏi user đóng track ở cuối mỗi subtask.

## Kế hoạch Windows

[Plan thích ứng Windows](windows_adaptation_plan.md) · [Milestones JSON](docs/tasks/windows_adaptation_plan.json).

**PLAN ONLY, CHỜ USER RA LỆNH TRIỂN KHAI.** Việc lập plan không cho phép bắt đầu prototype/build/code hoặc gọi agent. Cùng một track dài hạn bao phủ macOS và Windows; kết quả native/compatibility/release theo riêng từng platform. Ưu tiên candidate Windows sớm sau khi được giao, không chờ Mac hoàn thiện toàn bộ. Không có deadline cố định khi chưa xác nhận môi trường QA Windows.

## Checkpoint 2026-11-02 — tone trước w

Đã sửa core cho `ddwojwc` → `được`, build candidate riêng; [report và phần còn mở](docs/qa/tone_order_review.md). Bản có quyền hiện tại vẫn latency_candidate. Candidate chờ native QA: `build/tone_order_candidate/desktop_candidate/VXKey.app`. LOG_005/006/007 và mẫu `đayas` vẫn theo dõi; track MỞ, chỉ user đóng.

- Cập nhật kích hoạt: tone_order_candidate hiện đã có quyền và Resume, thay latency_candidate theo user duyệt. [Native evidence](docs/qa/tone_order_native_evidence.json); chờ physical typing, chưa nghiệm thu.

## Bàn giao review Argon — 2026-11-02

User xác nhận `ddwojwc được được mượt` sau Cmd+A/xóa: từ đầu còn raw, cảm giác mượt hơn hẳn. User tự giao Argon một vòng review; Codex dừng sửa để bàn giao. [Packet](docs/handoff/argon_cmd_a_review.md), [work log hiện hành](../_summary/work_log.md). Không đóng LOG_005/007 hoặc track dài hạn.

## Vòng sửa 1 sau chỉ đạo (LOG_005 Cmd+A/xóa & Shift+Enter) — 2026-11-03 (Anti)

- Đã hoàn tất phân tích nguyên nhân gốc và sửa lỗi mất dấu chữ đầu tiên (`ddwojwc được được mượt`) sau `Cmd+A` → xóa và sau `Shift+Enter` trong [context_guard.swift](native/macos_background/context_guard.swift), [event_driver.swift](native/macos_background/event_driver.swift), kèm 7 kiểm thử hồi quy mới trong [main.swift](native/tests/background_driver/main.swift).
- Báo cáo chi tiết cho Codex: [argon_cmd_a_fix_report.md](docs/qa/argon_cmd_a_fix_report.md).
- Giữ nguyên không ghi đè `build/tone_order_candidate/desktop_candidate/VXKey.app` đang có quyền TCC. Bộ đếm vòng sửa sau chỉ đạo trong work log tăng từ `0` lên `1`. Track bảo trì dài hạn giữ **MỞ**.

## Checkpoint bàn giao release — 2026-11-03

Turn QA/fix kết thúc bàn giao theo user, **track dài hạn MỞ**. Candidate đang dùng cmd_a_safe_candidate; trial đầu còn lỗi rồi các lần sau được, LOG_005 chưa đóng. [Packet release](docs/handoff/release_1_0_0_handoff.md), [work log](../_summary/work_log.md). Hai vòng sửa đã xong; releaseMac1.0.0 theo xác nhận user, permission flow/updater identity và retention còn TODO; Windows PLAN ONLY.
