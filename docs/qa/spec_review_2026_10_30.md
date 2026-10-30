# VXKey — Review cấu trúc bộ spec theo skills

Ngày: 2026-10-30. Version tài liệu: 0.3.0. Phạm vi: tài liệu, không triển khai hoặc native QA.

## Output theo skill

| Skill | Deliverable | Phần đã kiểm hiện diện |
|---|---|---|
| `_spec0_system` | [00](../spec/00_system_overview.md), [07](../spec/07_implementation_contract.md) | Scope/feature registry/architecture, milestone/dependency, handoff/gates |
| `_spec1_business_process` | [01](../spec/01_business_process.md) | Actors/lanes, swimlane Mermaid, SIPOC, decision table, operating fallback |
| `_spec2_feature_flow` | [02](../spec/02_feature_flow.md) | FL01..09, sequence/state diagrams, transitions, pre/postconditions/error branches |
| `_spec3_ui_wireframe` | [03](../spec/03_ui_wireframe.md) | Screen tree/wireframes/props/states/keyboard/responsive/visual prompts/assets |
| `_spec4_api_data` | [04](../spec/04_api_data.md) và JSON artifacts | Typed commands/results/errors, schema, ERD, lifecycle/IPC/auth boundaries |
| `_spec5_qa_acceptance` | [05](../spec/05_qa_acceptance.md) | 24 Gherkin scenarios, edge/validation/traceability matrix, native gates/checklist |
| `_spec6_demo_presentation` | [06](../spec/06_demo_presentation.md) | Seed/timeline/talk track, hotspot map/storyboard/visual prompt, contingency |

## Checks và giới hạn

- 44 liên kết relative trong index/spec được kiểm target tồn tại; 3 JSON code blocks parse được; code fences cân bằng. Không kiểm rendering/anchor layout bằng browser.
- Ba JSON artifacts parse được; config example khớp các keyword dùng trong schema bằng checker subset cục bộ; 10 positive/negative cases đạt. Máy và bundled Python không có `jsonschema`, bundled Node không có validator phù hợp, nên chưa chạy validator chuẩn Draft2020-12. Không coi subset checker là full JSON Schema certification; semantic byte length/counter range/unique IDs/native trust vẫn cần validators khi triển khai.
- Registry22commands;16requirement IDs đều map QA;7features map flow/screens/commands; scenarios Q01..Q24 hiện diện.
- Review logic sửa alias fields, uint64 crossing JS thành decimal strings, config tổng cap/IPC frame cap, stale target khi UI nhận focus, reset scope, import revision authority, copy-back outcome_unknown và shortcut ownership.
- Không render Mermaid, dựng prototype, chạy input source, clipboard, permissions, build/cài/update/release. Native/runtime và support platform vẫn not_run.

Đã hợp nhất các draft0.2 vào spec00..07 và cập nhật index/worklog. “Viết specs” giữ ✅; “specs ready” chờ user xác nhận, không tự đánh dấu từ các checks này.

## Bổ sung review spec0.5 — shortcut, icon và input recovery

User chốt modifier-only Mac Control+Option/Windows Control+Win có custom; serious V/E và playful Ư/W với4SVG user chuẩn bị. Chỉ đạo cuối giữ autostart default OFF khi chưa an toàn, không auto bật sau QA/update.

Đã kiểm link targets/fences/JSON parse, positive config/schema subset và rejection của modifier-only cho history, register toggle typed chord, platform defaults/config examples thống nhất;24command variants,39Gherkin scenarios và11features có mapping. Full Draft2020-12 validator vẫn chưa có; structural/subset checks không thay validation runtime.

[08_keyboard_resilience.md](../spec/08_keyboard_resilience.md) ghi problem framing/evidence, hypotheses,4solution directions/trade-offs, mandatory input isolation/suspension/native recovery/safe-start và fault matrix. Các tests native/frame timing/shortcut coexistence/recovery/asset rendering vẫn not_run. Chưa sửa source Funput, cấp quyền, chọn nguồn nhập, register startup, install hoặc release trong lượt spec.
