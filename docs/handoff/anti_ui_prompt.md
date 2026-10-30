# Prompt giao Anti làm UI VXKey

User tự dùng prompt này khi muốn tiếp tục. Codex chưa khởi chạy lượt mới.

---

Bạn là Anti, vai trò thực thi UI VXKey tại workspace `vxkey`. Đọc AGENTS.md global/workspace trước. Làm bằng cấu hình model user đã chọn; không dùng Argon. Lượt trước dùng gemini-3.8-flash-medium, nhưng không tự đổi model/config toàn máy.

Đọc `docs/handoff/ui_handoff.md` đầy đủ, `docs/qa/engine_implementation_report.md`, `docs/index.md` và toàn bộ spec 00–08, schema/defaults cùng chuẩn suite liên kết. Triển khai Tauri 2 + React + TypeScript frontend theo đúng UI handoff. Native engine từ Funput đã có; không viết lại và không tạo SwiftUI.

Được chỉnh `desktop` frontend/tooling, thêm app scaffold Tauri tối thiểu để dùng backend builder hiện có. Giữ command authority theo native window label; không sửa canonical Settings vendor trực tiếp, không giả capability/native success. Không chỉnh `crates`, `native`, `platforms`, config/schema/decisions ngoài task. Không cài/kích hoạt IME, autostart, TCC/keychain, commit/push/release hoặc thay global grants.

Hiện chỉ có canonical Settings seeds/assets và Tauri backend library; package/entry/config frontend chưa có. Tự tạo trong scope. Không đọc/copy cấu hình app khác ngoài context được cấp; dùng file đã có và chuẩn liên kết. Nếu headless permission bị từ chối, báo action/path cụ thể, không bypass, không ghi SUCCESS khi chưa viết UI. Báo thiếu quyền thay vì thử mở rộng toàn máy.

UI dùng invoke `vxkey_command`, native window labels settings/tray/history/onboarding/conversion. Capability false phải khóa control và giải thích; backend hiện không có event emitter, native capture/status/ack/hotkey registration hay app token. Chỉ Settings persist và text conversion có thể nối ngay; lịch sử RAM launch rỗng là đúng.

Giữ defaults Ctrl+Option Mac, Ctrl+Win Windows; history Ctrl+Cmd+V Mac/Win+V provider native default trên Win; autostart OFF cả hai; double-space Mac OFF/Win ON, khóa khi native on hoặc unknown; glyph serious V/E, playful Ư/W từ 4 SVG user.

Checks: typecheck/build frontend, cargo check Tauri adapter, canonical integrity, browser visual VI/EN/light/dark, config persistence/readback/revision conflict. Đừng install input source để test UI. Report file list, commands/results, screenshot thực, gaps và phạm vi chưa QA native/Windows.

Theo AGENTS.md mục 0.C: nếu chạy dưới Codex thì trả câu hỏi về Codex chủ trì. Nếu user giao trực tiếp, được tham vấn Codex chỉ đọc tối đa 2 lần/lượt; prompt bắt buộc “Chỉ tham vấn/review chỉ đọc; không sửa nguồn; không gọi agent khác”. Không tạo chuỗi agent. Giữ bộ đếm CLI trước đó; lượt bổ sung đã được user cho phép trước khi dừng chưa hề dùng. Chỉ bắt đầu lại khi user trực tiếp giao; không tự diễn giải tài liệu này là lệnh chạy.
