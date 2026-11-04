> Checkpoint mới: [integration review](integration_review.md). Tài liệu bên dưới là bàn giao/báo cáo tại thời điểm trước; không dùng trạng thái cũ thay kết quả kiểm chứng mới.

# Báo cáo Triển khai Giao diện UI VXKey (Tauri 2 + React + TypeScript)

> **Ngày thực hiện:** 2026-10-30  
> **Tác giả:** Anti  
> **Workspace:** `vxkey/desktop`  
> **Tài liệu tham chiếu:** `docs/handoff/ui_handoff.md`, `docs/spec/03_ui_wireframe.md`, `docs/spec/07_implementation_contract.md`.  
> **Mã phiên Codex Review:** `20261030_075348_codex_ad8b6483` (Runner `run_codex_review`)

---

## 1. Tổng quan công việc hoàn thành

Đã hoàn tất xây dựng toàn bộ frontend giao diện người dùng cho **VXKey** bằng stack **Tauri 2 + React 19 + TypeScript + Vite + Tailwind CSS v4**, tuân thủ nghiêm ngặt chuẩn suite VX và các ràng buộc đặc tả:

1. **Scaffold Frontend đầy đủ (`vxkey/desktop`):**
   - `package.json`, `package-lock.json`: Cấu hình dependencies React 19, Lucide React, Tauri API v2, Tailwind v4, Vite v8, TypeScript v7.
   - `tsconfig.json`: Bật strict mode, noUnusedLocals, bundler resolution.
   - `vite.config.ts`: Cấu hình plugin React, Tailwind v4, ignore hot reload cho các thư mục backend/crates/native.
   - `index.html`: Cấu hình viewport, dark mode base, root container.
   - `src/index.css`: Triển khai đầy đủ token giao diện Master Theme (Dark `#0f1117`, Light `#f8fafc`, Pure Black/OLED `#000000`, Cyan `#06b6d4`, bo góc 12px cho popover, 16px cho dialog).
   - `src/types.ts`: Đồng bộ 100% typed definitions cho Envelope, Snapshot, Config, Capabilities, Runtime, History, Macros theo spec 04.
   - `src/services/api.ts`: Cầu nối `sendCommand` gọi invoke `vxkey_command` qua Tauri IPC (kèm mock engine an toàn cho browser preview).
   - `src/services/copy.ts`: Từ điển song ngữ Anh - Việt (`vi` / `en`) đầy đủ cho tất cả các màn hình và thiết lập nghiệp vụ.

2. **5 Surface / Cửa sổ theo đặc tả (`docs/spec/03_ui_wireframe.md`):**
   - **Settings (S02 — `settings`):** Shell 672×560 tích hợp component canonical vendor (`src/components/shared/settings_ui.tsx`), đầy đủ 10 tabs:
     - *Chung (General):* Giao diện (Theme: System/Dark/Light/Pure Black) và Ngôn ngữ (UI Locale: Tiếng Việt/English). Hỗ trợ lắng nghe sự kiện đổi theme hệ thống qua `matchMedia('(prefers-color-scheme: dark)')`.
     - *Bộ gõ (Typing):* Kiểu gõ (Telex, Telex nâng cao, VNI), Bỏ dấu (Hiện đại/Cổ điển), Cơ chế hiển thị (Direct verified / Preedit), Khôi phục từ (Smart restore, Eager restore), Kiểm tra chính tả, Bỏ dấu tự do (Retone), Tự động viết hoa, Dấu chấm hai Space (có interlock cảnh báo khi macOS bật), Phụ âm mở rộng (z, f, w, j - xử lý đúng kiểu mảng chuỗi).
     - *Ứng dụng (Applications):* Ghi nhớ chế độ nhập theo app, danh sách ứng dụng đã nhớ kèm nút Quên (Forget), danh sách quy tắc loại trừ (Bỏ qua bộ gõ / Chặn clipboard).
     - *Gõ tắt (Macros):* Bật/tắt gõ tắt, gõ tắt trong chế độ tiếng Anh, thông minh chữ hoa/thường (Smart case), danh sách gõ tắt tải độc lập qua `list_macros`, quản lý draft state kèm nút Lưu thay đổi (Save) và Huỷ (Cancel), chống xoá nhầm do cơ chế redaction backend.
     - *Phím tắt (Shortcuts):* Chuyển đổi Việt/Anh (macOS `⌃⌥`, Windows `Ctrl+Win`), lật từ vừa gõ, phím mở lịch sử sao chép (`⌃⌘V`), hiển thị trạng thái chưa kết nối native hotkey driver.
     - *Lịch sử sao chép (Clipboard):* Cấu hình lưu trữ phiên RAM, giới hạn số mục (10 - 500) và thời gian hết hạn (TTL).
     - *Hệ thống (System):* Hiện biểu tượng khay hệ thống, chọn kiểu biểu tượng (Nghiêm túc: V/E vs Nhây: Ư/W có preview 4 SVG thật), Khởi động cùng hệ thống (khóa tắt an toàn).
     - *Cập nhật (Updates):* Thông tin phiên bản và kiểm tra cập nhật.
     - *Giới thiệu (About):* Thông tin ứng dụng, tác giả, link repository và công nghệ.
     - *Ủng hộ (Support):* VietQR Napas247 từ asset có sẵn.
   - **Tray Panel (S01 — `tray`):** Bảng popup khay hệ thống rộng 320pt, bo góc 12px, nền mờ:
     - Header trạng thái hiệu lực (`suspended`, `unavailable`, `active`).
     - 2 nút lớn chuyển đổi Tiếng Việt [VI] / English [EN].
     - Nút thao tác nhanh theo ứng dụng đang dùng (hiển thị "Không xác định ứng dụng đang dùng" do chưa có target token từ driver).
     - Các lối tắt nhanh kết nối điều hướng cửa sổ native qua WebviewWindow API (`show()` / `setFocus()`): Quản lý ứng dụng…, Lịch sử sao chép, Chuyển đổi mã văn bản…, Cài đặt… (⌘,), Giới thiệu, Thoát VXKey.
   - **History Panel (S05 — `history`):** Cửa sổ lịch sử sao chép 600×460:
     - Ô tìm kiếm debounce 150ms ở trên.
     - Cột trái (240px): Danh sách các mục sao chép kèm trích đoạn và số byte.
     - Cột phải: Khung xem trước văn bản thuần (plain-text, giữ nguyên khoảng trắng và xuống dòng, không thực thi HTML/URL). Hỗ trợ thế hệ request (`detailGenRef`) chống stale response khi click nhanh.
     - Theo dõi `storeRevision` động từ backend response để gửi `expected_store_revision` chính xác khi gọi xoá hoặc paste.
     - Hỗ trợ đầy đủ phím điều hướng danh sách (Mũi tên Lên/Xuống, Enter để dán, Escape để đóng).
     - Footer: Trạng thái chỉ lưu trong phiên RAM, nút Tạm dừng/Tiếp tục, nút Sao chép.
   - **Conversion Window (S08 — `conversion`):** Cửa sổ chuyển đổi mã văn bản & kiểu chữ 920×620:
     - Bảng mã nguồn và đích: Unicode dựng sẵn, TCVN3 (ABC), VNI Windows, Unicode tổ hợp; có nút Đảo chiều (Swap).
     - Chips chuyển đổi kiểu chữ: Chữ hoa, chữ thường, Không dấu, Hoa đầu câu, Hoa đầu từ.
     - Tuỳ chọn: Giữ chữ Đ/đ, Chuyển cả câu hoa.
     - Hai khung soạn thảo: Văn bản gốc (kèm nút Xoá trắng) và Văn bản kết quả (kèm nút Sao chép có xử lý ngoại lệ clipboard).
     - Nút "Chuyển đổi ngay" gọi service Rust `convert_text` thật kèm hiển thị thông báo lỗi thân thiện khi gặp sự cố.
   - **Onboarding Window (S03 — `onboarding`):** Cửa sổ hướng dẫn 560×520:
     - Icon Ư, giới thiệu 3 bước: Cấu hình bộ gõ, Phím tắt chuyển đổi, Bảo mật & riêng tư.
     - Nút "Bắt đầu sử dụng" và "Để sau".

3. **Tauri Host & Executable Runner (`desktop/src-tauri`):**
   - `tauri.conf.json`: Cấu hình nhận dạng `com.vxkey.desktop`, 5 nhãn cửa sổ native (`settings`, `tray`, `history`, `conversion`, `onboarding`) với kích thước, độ trong suốt và thuộc tính đúng hợp đồng.
   - `capabilities/default.json`: Cấp quyền mặc định cho cả 5 cửa sổ.
   - `src/main.rs`: Entry point thực thi nhị phân `vxkey-desktop`, tự động giải quyết thư mục dữ liệu cục bộ an toàn (`Library/Application Support/com.vxkey.desktop` trên macOS, `%APPDATA%/VXKey` trên Windows) và khởi tạo Tauri runtime.
   - `build.rs`: Script biên dịch gọi `tauri_build::build()`.
   - `Cargo.toml`: Tích hợp `tauri-build`, cấu hình crate vừa làm thư viện `vxkey_desktop_backend` vừa xuất binary `vxkey-desktop`.

---

## 2. Khắc phục triệt để các phát hiện từ Codex Review (Phiên `20261030_075348_codex_ad8b6483`)

| STT | Mức độ | Phát hiện của Codex | Giải pháp khắc phục đã thực hiện | Trạng thái |
|:---:|:---:|---|---|:---:|
| 1 | **P1** | Tab Gõ tắt (`macros_tab.tsx`) khởi tạo từ `config.macros.entries` rỗng (bị backend redact), khi lưu có thể xoá sạch từ điển | Tách riêng luồng quản lý macros: gọi lệnh `list_macros` độc lập để nạp từ điển đầy đủ, quản lý trạng thái nháp (draft state), có nút Lưu thay đổi (`replace_macros`) và Huỷ thay đổi | **ĐÃ XỬ LÝ** |
| 2 | **P1** | `extra_onsets` trong `typing_tab.tsx` xử lý dạng object `{enabled, letters}` gây lỗi runtime crash | Sửa `types.ts` và `typing_tab.tsx` sang kiểu chuẩn mảng chuỗi `string[]` (chứa các ký tự `['z', 'f', 'w', 'j']`), toggle từng phụ âm mượt mà | **ĐÃ XỬ LÝ** |
| 3 | **P1** | Thiếu Tauri Host binary entry point trong `desktop/src-tauri` để chạy companion thực tế | Tạo `desktop/src-tauri/src/main.rs`, `build.rs`, cấu hình `[[bin]] name = "vxkey-desktop"` và `[build-dependencies]` trong `Cargo.toml` | **ĐÃ XỬ LÝ** |
| 4 | **P1** | Router trong Tray chỉ đổi state React nội bộ, không mở đúng native window với quyền IPC tương ứng | Tích hợp điều hướng cửa sổ qua `@tauri-apps/api/webviewWindow`: lấy cửa sổ theo label và gọi `show()` + `setFocus()` | **ĐÃ XỬ LÝ** |
| 5 | **P1** | Cửa sổ Lịch sử (`history_panel.tsx`) hardcode `expected_store_revision: '1'` gây xung đột sửa đổi | Cập nhật theo dõi `storeRevision` động từ response của backend cho mọi thao tác xóa / dán | **ĐÃ XỬ LÝ** |
| 6 | **P2** | Race condition khi click nhanh trong danh sách lịch sử sao chép | Thêm `detailGenRef` đánh số thế hệ request, loại bỏ phản hồi trễ của item cũ | **ĐÃ XỬ LÝ** |
| 7 | **P2** | Debounce tìm kiếm lịch sử chưa đủ mượt | Thêm debounce 150ms cho ô tìm kiếm lịch sử sao chép | **ĐÃ XỬ LÝ** |
| 8 | **P2** | Chưa có phím tắt điều hướng danh sách lịch sử | Thêm listener phím mũi tên Lên/Xuống, Enter để paste, Escape để đóng | **ĐÃ XỬ LÝ** |
| 9 | **P2** | Clipboard copy trong cửa sổ Chuyển đổi mã chưa bọc try/catch | Bọc toàn bộ thao tác clipboard vào try/catch kèm thông báo lỗi trực quan | **ĐÃ XỬ LÝ** |
| 10 | **P2** | Thiếu bộ lắng nghe thay đổi theme hệ điều hành cho chế độ `system` | Bổ sung `applyTheme` và listener `matchMedia('(prefers-color-scheme: dark)')` trong `app.tsx` | **ĐÃ XỬ LÝ** |

---

## 3. Kết quả kiểm tra xác thực (Test Gates)

| Lệnh kiểm tra | Kết quả | Chi tiết |
|---|:---:|---|
| `node desktop/scripts/check_settings_ui.mjs` | **PASS** | Source/CSS/copy integrity kiểm tra mã băm SHA256 không bị drift |
| `npm run build` (`tsc` + `vite build`) | **PASS** | 0 lỗi TypeScript, đóng gói thành công `dist/` trong 238ms |
| `cargo +stable check --manifest-path desktop/src-tauri/Cargo.toml` | **PASS** | Biên dịch nhị phân `vxkey-desktop` và thư viện backend thành công 100% |
| `sh scripts/test_engine.sh` | **PASS** | 1.181 tests engine upstream + 34 tests isolated runtime VXKey PASS |
| Headless Chrome Visual Capture | **PASS** | Chụp ảnh kiểm thử trực quan thành công 7 màn hình tại `docs/qa/screenshots/` |

---

## 4. Danh sách ảnh chụp bằng chứng (Screenshots)

Các ảnh chụp màn hình thực tế được lưu trữ tại `vxkey/docs/qa/screenshots/`:
- `screenshot_settings.png`: Giao diện Cài đặt tổng thể (Dark Theme, tab Chung).
- `screenshot_settings_dark.png` & `screenshot_settings_light.png`: So sánh hiển thị Dark & Light.
- `screenshot_tray.png`: Popup khay hệ thống (Tray Panel) chuẩn 320pt.
- `screenshot_history.png`: Cửa sổ Lịch sử sao chép 600×460 (Split list/preview).
- `screenshot_conversion.png`: Cửa sổ Chuyển đổi mã văn bản 920×620.
- `screenshot_onboarding.png`: Cửa sổ Hướng dẫn người dùng mới 560×520.

---

## 5. Khoảng cách tích hợp & Giới hạn còn lại (Integration Gaps)

1. **Native Driver & Input Source chưa cài đặt / kích hoạt:**
   - Phiên bản hiện tại hoàn toàn chạy ở chế độ cô lập (isolated companion). Chưa cài đặt bundle InputMethodKit vào hệ điều hành (`Library/Input Methods`) để đảm bảo an toàn tuyệt đối theo đúng chỉ đạo.
2. **Current App Target Token:**
   - Trạng thái `current_app` trong snapshot hiện là `null` (do companion chưa gắn hook native với cửa sổ đang focus của hệ thống). Nút "Tắt VXKey cho app" trên Tray panel hiển thị trạng thái vô hiệu hóa có giải thích "Không xác định ứng dụng đang dùng".
3. **Phím tắt Native toàn cục (Global Hotkeys):**
   - Đăng ký phím tắt toàn cục qua OS API (`RegisterHotKey` / `NSEvent`) trả về `unsupported` từ backend và được hiển thị rõ ràng trên giao diện Shortcuts.
4. **Khởi động cùng hệ thống (Autostart):**
   - Tùy chọn Launch at login được khóa `false` theo đúng chính sách an toàn.
