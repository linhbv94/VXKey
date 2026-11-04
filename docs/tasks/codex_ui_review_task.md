# Task: Codex Review toàn diện Frontend UI VXKey (Tauri 2 + React + TypeScript)

> **Mục tiêu:** Chỉ tham vấn/review chỉ đọc; không sửa nguồn; không gọi agent khác.  
> **Người thực thi UI:** Anti  
> **Workspace:** `/Users/vic/_Work/VXUtilities/vxkey`  
> **Tài liệu tham chiếu:**
> - `docs/handoff/ui_handoff.md`
> - `docs/spec/03_ui_wireframe.md`
> - `docs/spec/07_implementation_contract.md`
> - `docs/qa/ui_implementation_report.md`
> - `docs/qa/engine_implementation_report.md`

---

## 1. Nội dung Anti vừa hoàn thành

Anti đã hoàn tất triển khai scaffold frontend và giao diện cho VXKey theo đúng thỏa thuận handoff:
1. **Scaffold Frontend (`vxkey/desktop`):**
   - Đã tạo `package.json`, `package-lock.json` (React 19, Lucide React, Tauri API v2, Tailwind v4, Vite v8, TypeScript v7).
   - Đã tạo `tsconfig.json`, `vite.config.ts`, `index.html`.
   - Đã cập nhật token giao diện `src/index.css` tuân thủ Master Theme (Dark `#0f1117`, Light `#f8fafc`, Pure Black/OLED `#000000`, Cyan `#06b6d4`, bo góc 12px cho popover, 16px cho dialog).
   - Đã đồng bộ `src/types.ts` typed definitions cho Envelope, Snapshot, Config, Capabilities, Runtime, History, Macros theo spec 04.
   - Đã triển khai `src/services/api.ts` (gọi invoke `vxkey_command` qua Tauri IPC, có mock an toàn cho browser preview).
   - Đã triển khai `src/services/copy.ts` hỗ trợ song ngữ (`vi` / `en`) đầy đủ cho tất cả các màn hình và thiết lập nghiệp vụ.

2. **5 Surface / Cửa sổ theo đặc tả (`docs/spec/03_ui_wireframe.md`):**
   - **Settings (S02 — `settings` | 672×560):** Shell 672×560 tích hợp component canonical vendor (`src/components/shared/settings_ui.tsx`), đầy đủ 10 tabs:
     - *Chung (General):* Giao diện (Theme: System/Dark/Light/Pure Black) và Ngôn ngữ (UI Locale: Tiếng Việt/English).
     - *Bộ gõ (Typing):* Kiểu gõ (Telex, Telex nâng cao, VNI), Bỏ dấu (Hiện đại/Cổ điển), Cơ chế hiển thị (Direct verified / Preedit), Khôi phục từ (Smart restore, Eager restore), Kiểm tra chính tả, Bỏ dấu tự do (Retone), Tự động viết hoa, Dấu chấm hai Space (có interlock cảnh báo khi macOS bật), Phụ âm mở rộng (z, f, w, j).
     - *Ứng dụng (Applications):* Ghi nhớ chế độ nhập theo app kèm danh sách đã nhớ và nút Quên (Forget); danh sách quy tắc loại trừ (Bỏ qua bộ gõ / Chặn clipboard).
     - *Gõ tắt (Macros):* Bật/tắt gõ tắt, gõ tắt trong chế độ tiếng Anh, thông minh chữ hoa/thường (Smart case), bảng từ viết tắt và form thêm gõ tắt có validation.
     - *Phím tắt (Shortcuts):* Chuyển đổi Việt/Anh (macOS `⌃⌥`, Windows `Ctrl+Win`), lật từ vừa gõ, phím mở lịch sử sao chép (`⌃⌘V`), hiển thị trạng thái chưa kết nối native hotkey driver.
     - *Lịch sử sao chép (Clipboard):* Cấu hình lưu trữ phiên RAM, giới hạn số mục (10 - 500) và thời gian hết hạn (TTL).
     - *Hệ thống (System):* Hiện biểu tượng khay hệ thống, chọn kiểu biểu tượng (Nghiêm túc: V/E vs Nhây: Ư/W có preview 4 SVG thật), Khởi động cùng hệ thống (khóa tắt an toàn).
     - *Cập nhật (Updates), Giới thiệu (About), Ủng hộ (Support).*
   - **Tray Panel (S01 — `tray` | 320pt):** Bảng popup khay hệ thống rộng 320pt:
     - Header trạng thái hiệu lực (`suspended`, `unavailable`, `active`).
     - 2 nút lớn chuyển đổi Tiếng Việt [VI] / English [EN].
     - Nút thao tác nhanh theo ứng dụng đang dùng (hiển thị "Không xác định ứng dụng đang dùng" do chưa có target token từ driver).
     - Các lối tắt nhanh: Quản lý ứng dụng…, Lịch sử sao chép, Chuyển đổi mã văn bản…, Cài đặt… (⌘,), Giới thiệu, Thoát VXKey.
   - **History Panel (S05 — `history` | 600×460):**
     - Ô tìm kiếm debounce ở trên.
     - Cột trái (240px): Danh sách các mục sao chép kèm trích đoạn và số byte.
     - Cột phải: Khung xem trước văn bản thuần (plain-text, giữ nguyên khoảng trắng và xuống dòng, không thực thi HTML/URL).
     - Footer: Trạng thái chỉ lưu trong phiên RAM, nút Tạm dừng/Tiếp tục, nút Sao chép (Enter).
   - **Conversion Window (S08 — `conversion` | 920×620):**
     - Bảng mã nguồn và đích: Unicode, TCVN3 (ABC), VNI Windows, Unicode tổ hợp (kèm nút Đảo chiều).
     - Chips chuyển đổi kiểu chữ: Chữ hoa, chữ thường, Không dấu, Hoa đầu câu, Hoa đầu từ.
     - Tuỳ chọn: Giữ chữ Đ/đ, Chuyển cả câu hoa.
     - Hai khung soạn thảo: Văn bản gốc (kèm nút Xoá) và Văn bản kết quả (kèm nút Sao chép), kết nối trực tiếp service `convert_text` Rust.
   - **Onboarding Window (S03 — `onboarding` | 560×520):**
     - Icon Ư, giới thiệu 3 bước: Cấu hình bộ gõ, Phím tắt chuyển đổi (`⌃⌥`), Bảo mật & riêng tư.

3. **Tauri Companion Scaffold (`desktop/src-tauri`):**
   - `tauri.conf.json`: Cấu hình nhận dạng `com.vxkey.desktop`, 5 nhãn cửa sổ native (`settings`, `tray`, `history`, `conversion`, `onboarding`) đúng kích thước và quyền hạn.
   - `capabilities/default.json`: Cấp quyền mặc định cho các cửa sổ.

4. **Kết quả kiểm tra xác thực (Test Gates):**
   - `node desktop/scripts/check_settings_ui.mjs`: **PASS**
   - `npm run build` (`tsc` + `vite build`): **PASS** (0 lỗi TypeScript, dist sạch)
   - `cargo +stable check --manifest-path desktop/src-tauri/Cargo.toml --offline`: **PASS**
   - `sh scripts/test_engine.sh`: **PASS** (1.181 tests engine upstream + 34 tests isolated runtime VXKey PASS)
   - Headless Chrome Visual Capture: **PASS** cho cả 5 màn hình, lưu tại `docs/qa/screenshots/`.

---

## 2. Yêu cầu Codex Review

1. Đối chiếu toàn bộ code frontend tại `vxkey/desktop/src/` và cấu hình `desktop/src-tauri/` với `docs/handoff/ui_handoff.md`, `docs/spec/03_ui_wireframe.md`, `docs/spec/07_implementation_contract.md`.
2. Kiểm tra tính đúng đắn của việc phân quyền theo window label (`settings`, `tray`, `history`, `onboarding`, `conversion`) tại frontend và backend `desktop/src-tauri/src/lib.rs`.
3. Đánh giá tính an toàn dữ liệu, xử lý lỗi (revision conflict, permission denied, capability locked), việc bảo lưu canonical vendor không drift.
4. Nêu các điểm mạnh, thiếu sót, edge cases tiềm ẩn hoặc khuyến nghị cải tiến cho bước tích hợp tiếp theo.
5. Định dạng: Tiếng Việt, UTF-8, không LaTeX, xuất báo cáo đánh giá khách quan.
