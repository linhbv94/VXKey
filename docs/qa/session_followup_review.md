# Session adapter — bản vá follow-up LOG_011 & candidate revision mới

Ngày: 2026-11-04. Owner thực thi: Anti (lượt nhận từ Codex). Track dài hạn **MỞ, chỉ user đóng**.

## Tổng quan mục tiêu và phạm vi

1. **Bảo toàn bản vá LOG_010**: Cơ chế phân biệt `unknown` vs `denied` trong `PermissionCache` (`permission_cache.swift`, `session_event_driver.swift`, `runtime_settings.swift`) đã hoàn thành và tích hợp vào bản build candidate.
2. **Khắc phục lỗi LOG_011 (Flip shortcut mở completion menu)**: Khi bấm tổ hợp phím đổi kiểu gõ/đảo dấu (như `Ctrl+Option+F5`), AppKit mở popup danh sách từ gợi ý `pas`. Sửa delivery của flip sang dạng độc lập (standalone replacement delivery) và nuốt trọn cả `keyDown` lẫn `keyUp` của phím tắt vật lý.
3. **Tích hợp cải tiến UX frontend có sẵn**:
   - `permission_setup.tsx`: Bước 3 giữ trạng thái kiểm thử gõ thực tế với trình soạn thảo bên ngoài, không tự đánh dấu hoàn thành chỉ vì hook active.
   - `apps_tab.tsx`: Nhãn xóa rule hiển thị rõ hành động và ID ứng dụng.
4. **Đóng gói candidate test revision mới**:
   - Root: `build/session_followup_candidate`
   - Data root riêng: `build/sf_data`
   - Báo cáo artifacts: `docs/qa/session_followup_artifacts.json`
   - Giữ nguyên toàn bộ candidate cũ `build/session_integrated_candidate` đang có quyền TCC.
   - Không commit/push/release, không cài vào `/Applications/`, Windows giữ nguyên PLAN ONLY.

---

## Chi tiết bản vá LOG_011

### 1. Nguyên nhân gốc rễ
Trước đây, trong nhánh `flipShortcut` của `session_event_driver.swift`:
- Khi tổ hợp phím tắt được phát hiện, mã nguồn gán `event.flags = []` rồi đẩy chính sự kiện vật lý gốc (vẫn mang `keyCode: 96` của F5) qua `deliver()`.
- Tại AppKit (`NSTextView`), sự kiện phím F5 (`\uF708`) được ánh xạ mặc định tới selector `complete:` (Word Completion). Khi nhận phím F5 có chuỗi ký tự Unicode thay thế, AppKit vừa nhận ký tự vừa kích hoạt cửa sổ gợi ý từ.

### 2. Giải pháp kỹ thuật đã triển khai
- **`StreamDelivery.prepareStandalone`** (`native/macos_session/stream_delivery.swift`):
  - Nhận `EditPlan`, `CGEventSource?` và `nonce`.
  - Tiền kiểm: `backspaces` từ 0 đến 128, độ dài payload không vượt quá 8192 đơn vị UTF-16, kiểm tra các codepoint không rỗng.
  - Tổng hợp `plan.backspaces * 2` sự kiện phím xóa ảo (`kVK_Delete` / 51, gồm `[down, up]`).
  - Tổng hợp các cặp sự kiện phím ảo chèn ký tự mang `virtualKey: 0` (`kVK_ANSI_A`), `flags: []`, đánh dấu `nonce` và kiểm tra xác thực CoreGraphics chấp nhận chuỗi UTF-16 qua `keyboardGetUnicodeString`.
- **`SessionEventDriver.flipShortcut`** (`native/macos_session/session_event_driver.swift`):
  - Nếu `autorepeat != 0`: nuốt phím ngay lập tức (`consumed.insert(code); return nil`).
  - Khi `composition.flip()` trả về `.replace(plan)`: gọi `StreamDelivery.prepareStandalone`. Nếu thành công, phát trọn gói qua `proxy.tapPostEvent()` và nuốt phím vật lý (`consumed.insert(code); return nil`).
  - Ghi nhận `keyCode` vào tập `consumed` để khi sự kiện `keyUp` tương ứng đến, nó cũng bị nuốt trọn (`consumed.remove(code); return nil`), ngăn tuyệt đối phím F5 lọt tới AppKit.

---

## Bằng chứng kiểm thử tự động (Verification Gates)

| Suite | Lệnh kiểm thử | Kết quả | Ghi chú |
|---|---|:---:|---|
| Swift session adapter fixtures | `sh scripts/test_session_adapter.sh` | **111 checks PASS** | Bổ sung unit tests cho `prepareStandalone`, xác thực không post event thật, RAM fixture |
| Background bridge 2-process | `VXKEY_SESSION_ADAPTER=1 python3 scripts/test_background_bridge.py` | **6 checks PASS** | Cache timer, pending startup permission không latch, 3 chu kỳ Resume/Pause, resilience beyond lease, reconnect không cần Resume, explicit Pause |
| Rust runtime engine | `cargo +stable test -p vxkey-runtime --offline` | **59 tests PASS** | 4 unit tests, 16 bridge tests, 39 policy tests |
| UI permission decision matrix | `node scripts/test_native_permissions.mjs` | **145 checks PASS** | Ma trận quyết định quyền UI dùng chung |
| Frontend build & CSS integrity | `npm run build --prefix desktop` | **PASS** | `check_settings_ui.mjs` PASS, `tsc` + `vite build` 0 lỗi |
| Desktop Tauri backend check | `cargo +stable check --manifest-path desktop/src-tauri/Cargo.toml --features custom-protocol --offline` | **PASS** | Không có cảnh báo hay lỗi biên dịch |
| Strict codesign verification | `codesign --verify --deep --strict ...` | **PASS** | Đạt trên cả companion app và engine app |

---

## Thông tin Candidate Artifacts

- **Companion App**: `build/session_followup_candidate/desktop_candidate/VXKey.app`
  - Bundle ID: `com.vxkey.test.desktop`
  - CDHash: `77d4f88ff0abf7528e602e43b8d5a853378849a9`
  - Binary SHA256: `2de47f66a3ca57d24defc9f5ad1c2917cc88b2893e5ac748ffe0800ab7025c53`
- **Helper Engine App**: `build/session_followup_candidate/desktop_candidate/VXKey.app/Contents/Resources/VXKeyEngine.app`
  - Bundle ID: `com.vxkey.test.engine`
  - CDHash: `ad3e660132b4d1ee6622d38065daa4fc60cbf46c`
  - Binary SHA256: `76391a040853c84d2128a5761a1624fc7fe7cfd13e051f3d486dee81c4378fd6`
- **Data Root**: `build/sf_data` (chứa `bridge_pair.json` phân quyền ad-hoc pins, permissions `0o600`)
- **Tùy chọn an toàn**:
  - `autostart`: OFF
  - `clipboard`: OFF
  - `updater`: OFF
  - `input_source_required`: False

---

## Phần còn mở & Hướng dẫn Native QA
1. **Candidate cũ**: Vẫn đang chạy tại PID companion 55110 / helper 55122 trong TCC. Nếu muốn thử nghiệm candidate mới `build/session_followup_candidate`, user có thể thoát candidate cũ (`Cmd+Q`) và khởi chạy candidate mới để cấp quyền TCC Accessibility tương ứng.
2. **Kiểm tra LOG_011**: Thử bấm tổ hợp phím tắt `Ctrl+Option+F5` trên một từ tiếng Việt trong TextEdit / Figma, xác nhận chữ đảo dấu/chuyển đổi bình thường mà không bị hiện popup gợi ý từ ("pas").
3. **Kiểm tra LOG_010**: Thoát và mở lại candidate mới để xác minh không còn bị rơi vào trạng thái Pause giả do observation khởi động trễ.
