# VXKey — Bộ đặc tả bàn giao theo Specification Track

> **Bàn giao lượt tiếp2026-11-04:** [Anti next handoff](../anti_next_handoff.md), user tự giao; Codex chưa gọi agent.

> Kiến trúc hiện hành 2026-11-02: bộ gõ chạy nền, giữ ABC/nguồn nhập đang dùng, Rust Funput + native keyboard driver + Tauri UI. Không yêu cầu Input Source VXKey. Đọc [hợp đồng background driver](spec/09_background_driver.md) trước các hướng dẫn IME lịch sử.

> **Điểm tiếp hiện hành:** [Session adapter integration candidate](qa/session_adapter_integration.md), app Test đã build và đang native QA; source/fixture không thay gõ thật. Default build và production chưa đổi.

> **Pilot lịch sử:** [Review và kế hoạch adapter macOS](spec/11_mac_driver_replan.md), [pilot report](qa/session_spike_review.md). V6 return-original đã qua P1 Telex TextEdit/Figma và P2 boundary/sentinel cơ bản theo user; prototype đã Quit sau QA. Chưa P2 đầy đủ/P3/P4, chưa tích hợp hoặc thay contract09/10/production. Không suy full parity/daily-use acceptance từ pilot PASS.

> **Quyết định2026-11-04 — MAC_ADAPTER_002:** chọn route v6 làm hướng tích hợp tiếp theo. Giữ Funput/Tauri; chỉ post Backspace, Unicode đi trên event phím gốc. [Spec11](spec/11_mac_driver_replan.md) ghi evidence matrix và backlog MAC_ADAPT_01…05; đây là điểm đọc trước khi sửa adapter/config/lifecycle tiếp.

**Bridge/recovery baseline:** [implementation, fixture và native subset](qa/background_driver_review.md), [evidence summary](qa/native_bridge_summary.json). Đây là snapshot lịch sử; xem [track hiện hành](../maintenance_track.md) và [issue log](../typing_issue_log.md) cho trạng thái sử dụng.

Version: 0.7.0 — Ngày cập nhật: 2026-10-30. **Đã viết để review; chưa specs ready.**

**Chốt UI mới (native subset đã kiểm, xem giới hạn trong report):** [Settings native full-window, tray và About](qa/ui_window_review.md); [spec03](spec/03_ui_wireframe.md) v0.7 theo feedback user, chưa tự xác nhận specs ready.

**Checkpoint tích hợp trước đó:** [review/tích hợp sau UI](qa/integration_review.md), [evidence](qa/integration_evidence.json). Frontend và companion host đã có; các handoff/engine report trước là snapshot lịch sử. Chưa nghiệm thu IME dùng hằng ngày.

UI là **Tauri2 + React dùng chung Mac/Windows** theo user; Mac dùng background driver + Rust core, Windows PLAN ONLY. InputMethodKit là hướng đã hủy. Bộ này thay các bản draft0.2 chia theo chủ đề trước đây.

| Thứ tự | Tài liệu | Skill / output bắt buộc |
|---|---|---|
|00 | [Tổng quan hệ thống](spec/00_system_overview.md) | `_spec0_system`: scope, architecture, catalog, ownership |
|01 | [Quy trình nghiệp vụ](spec/01_business_process.md) | `_spec1_business_process`: actors, swimlane, SIPOC, decision table |
|02 | [Feature flow](spec/02_feature_flow.md) | `_spec2_feature_flow`: flows, sequence, state diagrams/matrix, execution contracts |
|03 | [UI wireframe](spec/03_ui_wireframe.md) | `_spec3_ui_wireframe`: component tree/props/states, wireframes, responsive, visual prompts |
|04 | [API và dữ liệu](spec/04_api_data.md) | `_spec4_api_data`: typed requests/results/errors, schema, ERD, integration |
|05 | [QA acceptance](spec/05_qa_acceptance.md) | `_spec5_qa_acceptance`: Gherkin, edge/validation matrix, traceability, QA checklist |
|06 | [Demo presentation](spec/06_demo_presentation.md) | `_spec6_demo_presentation`: seed, timeline/talk track, clickthrough, storyboard, contingency |
|07 | [Hợp đồng triển khai](spec/07_implementation_contract.md) | Coordinator: milestone/dependency, install/update/quyền, handoff gates |
|08 | [Keyboard resilience](spec/08_keyboard_resilience.md) | Invariant và recovery |
|09 | [Background driver](spec/09_background_driver.md) | Contract driver hiện tại |
|10 | [Checkpoint pipeline](spec/10_checkpoint_pipeline.md) | AX stream baseline hiện tại |
|11 | [Lập lại kế hoạch adapter Mac](spec/11_mac_driver_replan.md) | P1 pilot native/boundary/sentinel cơ bản PASS; P2/P3/P4 còn mở; chưa thay09/10 |

Machine-readable artifacts: [config schema](spec/config_schema.json), [config example](spec/config_example.json), [command schema](spec/command_schema.json). JSON Schema không thay semantic validators hoặc native acceptance.

Bắt đầu đọc00, rồi01..06 và07 trước task triển khai. Feature→flow→screen→command→QA ở05. Số requirement và defaults ở00, runtime branch ở02, presentation ở03, data/IPC ở04. Không tự tạo nguồn spec song song. Khi user thay quyết định phải đồng bộ tài liệu liên quan và tracker.

Các chuẩn suite bắt buộc: [index chuẩn](../../_summary/standard_docs/index.md), [UI](../../_summary/standard_docs/desktop_ui_standards.md), [theme](../../_summary/standard_docs/master_theme.md), [Settings](../../_summary/standard_docs/settings_standards.md), [component](../../_summary/standard_codes/settings_ui/index.md), [song ngữ](../../_summary/standard_docs/bilingual_standards.md), [titlebar](../../_summary/standard_docs/titlebar_standards.md), [app menu](../../_summary/standard_docs/macos_app_menu_standards.md), [updater](../../_summary/standard_docs/auto_update_standards.md), [SDK](../../_summary/standard_docs/macos_build_baseline.md), [quyền](../../_summary/standard_docs/desktop_system_permissions.md).

Nguồn nghiên cứu lịch sử: [audit](../audit_2026_10_27.md), [clipboard/app scope](../app_exclusion_clipboard_scope.md), [work cũ](../work_to_do.md). Không suy findings hoặc runtime PASS từ snapshot cũ. Namespace/signer/baseline/support/defaults/assets cuối có blocking decisions ở07. Chỉ user xác nhận mới ghi specs ready; chưa build/cài/cấp quyền/release trong lượt viết spec.

Review tài liệu và giới hạn kiểm chứng: [spec_review_2026_10_30.md](qa/spec_review_2026_10_30.md).

User decisions0.4: giữ đủ upstream desktop capabilities; [default theo OS](spec/platform_defaults.json), [Windows fresh example](spec/config_example_windows.json); double-space Mac=false/Windows=true, macOS native on khóa VXKey toggle. Global shortcut remap ngoài scope.

Conversion parity đã nhận vào scope. Text conversion wrapper đã nối Rust và bổ sung command_schema; file/batch/session wrappers còn gate tích hợp ở04 mục13/14 trước full M3. Không coi text conversion là toàn bộ parity.

User decisions0.5: toggle Mac Ctrl+Option/Win Ctrl+Win với custom modifier-only; autostart vẫn mặc định off khi chưa an toàn theo chỉ đạo cuối; serious V/E hoặc playful Ư/W, user cung cấp4SVG. [Brainstorm và contract chống treo bàn phím](spec/08_keyboard_resilience.md) là phần bổ sung cần đọc trước native implementation.

History0.5.1: Windows giữ Win+V và cho chọn native/VXKey, default native; scoped routing có native integration/fault gates trước claim hoạt động. Mac Control+V có navigation binding trong text apps, chỉ kiểm chưa đổi default Control+Command+V.


## Snapshot triển khai và bàn giao 2026-10-30

Ngày 2026-10-30: [engine implementation report](qa/engine_implementation_report.md), [UI handoff chi tiết](handoff/ui_handoff.md), [prompt tự giao Anti](handoff/anti_ui_prompt.md). Khi đó Rust foundation/native candidate và Tauri backend library đã có, frontend chưa triển khai. Đây là lịch sử, không phải trạng thái frontend hiện tại. Các checks source/fixture/build không thay native runtime QA; xem track và kế hoạch11 ở đầu index để tiếp tục.
