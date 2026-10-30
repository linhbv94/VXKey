# VXKey — Bộ đặc tả bàn giao theo Specification Track

Version: 0.5.1 — Ngày cập nhật: 2026-10-30. **Đã viết để review; chưa specs ready.**

UI đích là **Tauri2 + React dùng chung Mac/Windows** theo user; Mac giữ InputMethodKit + Rust core, Windows theo giai đoạn sau. Bộ này thay các bản draft0.2 chia theo chủ đề trước đây.

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

Machine-readable artifacts: [config schema](spec/config_schema.json), [config example](spec/config_example.json), [command schema](spec/command_schema.json). JSON Schema không thay semantic validators hoặc native acceptance.

Bắt đầu đọc00, rồi01..06 và07 trước task triển khai. Feature→flow→screen→command→QA ở05. Số requirement và defaults ở00, runtime branch ở02, presentation ở03, data/IPC ở04. Không tự tạo nguồn spec song song. Khi user thay quyết định phải đồng bộ tài liệu liên quan và tracker.

Các chuẩn suite bắt buộc: [index chuẩn](../../_summary/standard_docs/index.md), [UI](../../_summary/standard_docs/desktop_ui_standards.md), [theme](../../_summary/standard_docs/master_theme.md), [Settings](../../_summary/standard_docs/settings_standards.md), [component](../../_summary/standard_codes/settings_ui/index.md), [song ngữ](../../_summary/standard_docs/bilingual_standards.md), [titlebar](../../_summary/standard_docs/titlebar_standards.md), [app menu](../../_summary/standard_docs/macos_app_menu_standards.md), [updater](../../_summary/standard_docs/auto_update_standards.md), [SDK](../../_summary/standard_docs/macos_build_baseline.md), [quyền](../../_summary/standard_docs/desktop_system_permissions.md).

Nguồn nghiên cứu lịch sử: [audit](../audit_2026_10_27.md), [clipboard/app scope](../app_exclusion_clipboard_scope.md), [work cũ](../work_to_do.md). Không suy findings hoặc runtime PASS từ snapshot cũ. Namespace/signer/baseline/support/defaults/assets cuối có blocking decisions ở07. Chỉ user xác nhận mới ghi specs ready; chưa build/cài/cấp quyền/release trong lượt viết spec.

Review tài liệu và giới hạn kiểm chứng: [spec_review_2026_10_30.md](qa/spec_review_2026_10_30.md).

User decisions0.4: giữ đủ upstream desktop capabilities; [default theo OS](spec/platform_defaults.json), [Windows fresh example](spec/config_example_windows.json); double-space Mac=false/Windows=true, macOS native on khóa VXKey toggle. Global shortcut remap ngoài scope.

Conversion parity đã nhận vào scope; typed conversion wrappers còn gate tích hợp ở04 mục13 trước M3. Không giả command_schema hiện tại bao phủ wrappers chưa được định nghĩa.

User decisions0.5: toggle Mac Ctrl+Option/Win Ctrl+Win với custom modifier-only; autostart vẫn mặc định off khi chưa an toàn theo chỉ đạo cuối; serious V/E hoặc playful Ư/W, user cung cấp4SVG. [Brainstorm và contract chống treo bàn phím](spec/08_keyboard_resilience.md) là phần bổ sung cần đọc trước native implementation.

History0.5.1: Windows giữ Win+V và cho chọn native/VXKey, default native; scoped routing có native integration/fault gates trước claim hoạt động. Mac Control+V có navigation binding trong text apps, chỉ kiểm chưa đổi default Control+Command+V.


## Triển khai và bàn giao hiện tại

Ngày 2026-10-30: [engine implementation report](qa/engine_implementation_report.md), [UI handoff chi tiết](handoff/ui_handoff.md), [prompt tự giao Anti](handoff/anti_ui_prompt.md). Rust foundation/native candidate và Tauri backend library đã có; frontend chưa triển khai. Các checks source/fixture/build không thay native runtime QA. Spec vẫn là mục tiêu; capability matrix trong handoff thể hiện phần chạy được hiện tại.
