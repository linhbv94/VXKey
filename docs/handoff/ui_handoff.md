# VXKey — bàn giao UI

Ngày: 2026-10-30. Trạng thái: engine foundation có tests; native candidate compile; frontend chưa triển khai. Anti đã dừng theo user. Tài liệu này phục vụ user tự giao Anti, không tự khởi chạy CLI.

## 1. Đọc trước khi viết

1. [Index spec](../index.md), đọc toàn bộ 00–08, đặc biệt [UI](../spec/03_ui_wireframe.md), [API đích](../spec/04_api_data.md), [QA](../spec/05_qa_acceptance.md), [keyboard resilience](../spec/08_keyboard_resilience.md).
2. [Implementation report](../qa/engine_implementation_report.md): phân biệt API chạy được với API đích.
3. Chuẩn suite: [index](../../../_summary/standard_docs/index.md), [UI](../../../_summary/standard_docs/desktop_ui_standards.md), [Settings](../../../_summary/standard_docs/settings_standards.md), [theme](../../../_summary/standard_docs/master_theme.md), [song ngữ](../../../_summary/standard_docs/bilingual_standards.md), [titlebar](../../../_summary/standard_docs/titlebar_standards.md), [menu Mac](../../../_summary/standard_docs/macos_app_menu_standards.md), [Settings source](../../../_summary/standard_codes/settings_ui/index.md).
4. [Config schema](../spec/config_schema.json), [Mac example](../spec/config_example.json), [Windows example](../spec/config_example_windows.json), [defaults](../spec/platform_defaults.json). Đây là config có đủ shape, không tự thêm field vào JSON.

Spec là mục tiêu sản phẩm; mã service là hợp đồng đang thực thi. Khi hai bên khác nhau, báo gap và hiển thị trạng thái chưa khả dụng. Không sửa spec để che thiếu tích hợp.

## 2. Phạm vi và quyền sở hữu

| Khu vực | Trách nhiệm |
|---|---|
| `desktop/src`, `desktop/public`, frontend tooling/config | UI agent triển khai |
| `desktop/src/components/shared/settings_*` | Vendor chuẩn, không sửa trực tiếp; dùng props/slots |
| `desktop/src-tauri/src/lib.rs` | Backend adapter Codex đã chuẩn bị; giữ dispatch/authority |
| Tauri main/config/capabilities | UI agent thêm scaffold tối thiểu, giữ window labels/command bridge dưới đây |
| `crates/`, `native/`, `platforms/`, engine build scripts | Giữ nguyên khi làm UI; báo integration request nếu cần |
| `docs/spec` | Ground truth; không tự đổi quyết định user |

Không install/register/activate input source, không sửa TCC/keychain, không chạy autostart, không commit/push/release. Các thao tác đó không nằm trong UI handoff. Không đổi model sang Argon. User tự quyết định cách gọi Anti.

## 3. Trạng thái workspace và scaffold cần tạo

Đã có `desktop/src-tauri/Cargo.toml`, backend library, canonical Settings TSX/CSS/copy/manifest, script integrity, app icon/QR và bốn SVG. Chưa có `desktop/package.json`, React entry, Vite config, Tauri app context/main hay cửa sổ thực tế.

Dựng Tauri 2 + React + TypeScript theo chuẩn suite. Không tạo SwiftUI. Native InputMethodKit chạy riêng; UI không nằm trong đường xử lý phím. Cửa sổ nhỏ mở khi bấm icon menubar/system tray chính là tray panel trong spec.

Bốn asset có sẵn trong `desktop/public`: `menubar_serious_vi.svg`, `menubar_serious_en.svg`, `menubar_playful_vi.svg`, `menubar_playful_en.svg`. Nghiêm túc V/E, nhây Ư/W. Dùng SVG user cung cấp; chưa xác nhận raster/template rendering thực tế trên menubar.

Canonical gate:

```sh
node desktop/scripts/check_settings_ui.mjs
```

Gate chỉ chứng minh nguồn vendor không drift; không thay visual/native QA.

## 4. Gắn backend thật

Library crate tên `vxkey_desktop_backend`; gọi `builder(trusted_data_dir)` rồi `.run(tauri::generate_context!())` từ Tauri binary sau khi có config/build context. Không tạo một Builder thứ hai làm mất state/invoke handler. Không thay engine bằng mock trong bản Tauri.

Native host tự resolve data dir: macOS Application Support/VXKey trong user domain; Windows chọn app data phù hợp khi triển khai Windows. Không lấy đường dẫn từ JS/query string. QA dùng thư mục fixture riêng. Native candidate hỗ trợ `VXKEY_DATA_DIR` để trỏ fixture; frontend không được ghi biến này lên môi trường toàn máy.

Window label native bắt buộc: `settings`, `tray`, `history`, `onboarding`, `conversion`. Label xác định quyền tại Rust; `?surface=` chỉ điều hướng render, không cấp quyền. Unknown label bị `permission_denied`. Không cho JS chỉ định Surface/Developer. Mọi window load local bundle; không mở arbitrary remote URL có IPC.

```ts
import { invoke } from '@tauri-apps/api/core';
const result = await invoke<Envelope<Snapshot>>('vxkey_command', {
  request: { command: 'get_snapshot' }
});
```

Envelope thực tế:

```ts
type Envelope<T> = {
  ok: boolean;
  request_id: string | null;
  data: T | null;
  error: null | { code: string; message_key: string; retryable: boolean };
};
```

Request không gửi `request_id`; service tự tạo. Revision luôn là chuỗi decimal uint64, không convert Number. `get_snapshot` không trả macro bodies; dùng `list_macros` từ Settings. Không spread snapshot redacted lên toàn bộ config để lưu.

## 5. API có thể dùng ngay

Ground truth: [service.rs](../../crates/vxkey-runtime/src/service.rs), [config.rs](../../crates/vxkey-runtime/src/config.rs), [Tauri adapter](../../desktop/src-tauri/src/lib.rs).

| Command | Hành vi hiện tại |
|---|---|
| `get_snapshot` | Config redacted + revision + runtime/capabilities; engine unavailable/disconnected là thật |
| `patch_config` | Persist typed partial groups, revision check; không có native ack |
| `set_app_policy`, `forget_app_language` | Persist policy/memory; native target capture chưa nối |
| `list_macros`, `replace_macros` | Read/replace có validation/quota/revision |
| `list_history`, `get_history_entry`, `delete_history_entry` | Store RAM; chưa có OS capture nên launch bình thường rỗng |
| `set_capture_state` | Chỉ off/pause; bật monitoring không được hỗ trợ |
| `convert_text` | Convert charset/textcase thật từ upstream; conversion window được phép gọi |

`patch_config` chỉ nhận:

- `ui`: locale, theme, tray_icon_style.
- `typing`: input_method, tone_style, language_mode, composition_mode, smart_restore, eager_restore, spell_check, retone, auto_capitalize, extra_onsets, double_space_period.
- `apps`: remember_per_app.
- `clipboard`: max_entries, ttl_days.
- `system`: show_tray_icon.

Ví dụ persist:

```json
{"command":"patch_config","expected_revision":"1","changes":{"ui":{"tray_icon_style":"playful"}}}
```

Thành công trả `persisted_revision`, `applied_revision:"0"`, `engine_ack_state:"unavailable"`. Hiển thị “Đã lưu; chưa xác nhận áp dụng” khi chưa ack. Không biến persist thành “engine đang hoạt động”. `revision_conflict` → tải snapshot mới, giữ edit người dùng, không ghi đè mù. `durability_uncertain` nghĩa rename đã commit nhưng directory sync chưa xác nhận; refresh snapshot trước mọi retry, hiển thị lỗi lưu chưa bảo đảm durability.

Ví dụ conversion:

```json
{"command":"convert_text","text":"Tiếng Việt","from_charset":"unicode","to_charset":"unicode","transforms":["no_diacritics","upper"],"keep_d":false,"flatten_caps":false}
```

Trả `text`, `unmapped`, `normalized`; hiển thị cảnh báo mất/chuẩn hóa ký tự khi số đếm > 0. Charset: unicode/tcvn3/vni_windows/unicode_combining. Transform theo thứ tự: upper/lower/no_diacritics/sentence/title, tối đa 5, không trùng. Input tối đa 1 MiB. Chưa có conversion file/clipboard/selection native.

## 6. Capability và những nút phải khóa

Snapshot hiện có settings/session_history/conversion_text=true; native_capture/encrypted_history/native_engine/native_shortcuts/autostart/windows_history_provider/conversion_files/updater=false.

Các command chưa nối trả `unsupported`: register_hotkey, set_launch_at_login, set_windows_history_provider, set_history_mode, import/export, copy_history_entry, clear_history, reset_preferences, set_input_suspended, update_action; open_surface/show_about/open_project_url cũng chưa có host handler. `set_language` và `set_current_app_bypass` trả stale_target vì chưa có target token native. Không gọi OS bằng shell để “lấp” API thiếu.

Settings có thể chỉnh preference được hỗ trợ, nhưng native effective state phải hiển thị đúng snapshot. Shortcut custom hiển thị binding hiện có và unavailable cho Save tới khi native registration được nối. Đừng dùng patch_config để lách register_hotkey.

Hiện chưa có event emitter config_changed/runtime_changed/history_changed/update_changed. UI dùng refresh có giới hạn khi visible, dừng khi hidden/unmount và bỏ response cũ bằng generation. Events trong spec là việc tích hợp sau; không giả đã có listener.

Backend không cung cấp cửa sổ/About/URL/updater hoàn chỉnh. Nếu UI cần thêm native wrapper tối thiểu, ghi riêng integration gap, giữ token allowlist/label allowlist; không tự thêm lệnh chạy arbitrary shell/path/URL.

## 7. Các quyết định user phải giữ

| Setting | macOS | Windows tương lai |
|---|---|---|
| Đổi VI/EN | Ctrl+Option, modifier-only, nhả hết mới đổi | Ctrl+Win, cùng gesture |
| Custom shortcut | Có UI dự kiến; runtime registration chưa nối | Sau native integration |
| Autostart | OFF | OFF cho tới quyết định an toàn mới |
| History | Ctrl+Cmd+V | Win+V; chọn native/VXKey, default native |
| Double-space period | Preference OFF; native bật thì VXKey bị khóa | Preference ON |
| Tray style | serious mặc định; playful tùy chọn | Cùng quyết định |

Mac native period unknown cũng khóa VXKey, không cho toggle bật trước khi kiểm tra. Hiện service chưa nối AppKit detector nên báo checking/disabled. Không tạo hai bộ cùng thêm dấu chấm. Ctrl+V Mac không dùng làm history vì có text navigation binding.

Quick action “Tắt/Bật VXKey cho {app}” phải lấy external app trước khi mở panel, token gắn app/pid/generation; stale thì refresh. Hiện current_app=null nên disabled có giải thích. App exclusion chỉ bypass input, không tự chặn clipboard hay xóa language memory.

Telex/Telex nâng cao/VNI và các tùy chọn dùng Rust upstream. Không tự suy diễn quy tắc w trong frontend; xem engine tests/upstream. UI ngôn ngữ VI/EN tách khỏi chế độ gõ VI/EN.

## 8. Native candidate và giới hạn

Build `sh scripts/build_native_candidate.sh` chỉ tạo isolated `.app`. Candidate không chứa SwiftUI, dùng Rust FFI static. Native candidate tạm buộc preedit dù config lưu direct_verified. Settings phải phân biệt preference với effective profile; không quảng cáo direct đã nghiệm thu.

Config cache native poll 500ms là prototype read-only, chưa phải authenticated IPC/single-writer protocol. Native chưa gửi ack/status/tray mode/per-app changes trở lại backend. Backend snapshot chủ động để native_engine=false. Không suy “ready” từ file tồn tại hoặc binary build xong.

Health/period/history policy hiện có unit/fixture tests; watchdog native, fresh OS clipboard provenance, encrypted storage/keychain, native hotkey/provider và shutdown/recovery end-to-end còn gap. Không bật autostart hoặc live monitoring.

## 9. Thứ tự triển khai UI và nghiệm thu

1. Scaffold frontend + host, chạy Settings từ backend snapshot thật. Hoàn thành VI/EN/theme và canonical tabs trước.
2. Dựng Typing/Shortcuts/Apps/Macros/Clipboard/Conversion theo screen spec, dùng capability states, empty/error/loading thật.
3. Dựng tray panel đúng glyph, app exclusion disabled tới khi có target. UI không hook phím toàn cục.
4. Xác minh edit → persist → đóng/mở process → readback; conflict/validation/error không hiển thị success.
5. Chạy typecheck/frontend build, Tauri cargo check, canonical gate, browser QA light/dark + VI/EN; cung cấp screenshot và log thật. Native tray focus/z-order/window sizing/IME/clipboard phải tách là chưa test nếu chưa chạy.

```sh
sh scripts/test_engine.sh
cargo +stable check --manifest-path desktop/src-tauri/Cargo.toml --offline
node desktop/scripts/check_settings_ui.mjs
```

Frontend scripts do UI agent thêm rồi ghi lệnh chính xác vào báo cáo. Không đánh dấu Windows PASS bằng build Mac. Không gọi report/exit0 là nghiệm thu. Bàn giao diff/file list, checks đã chạy, ảnh preview, các capability còn disabled và native gaps riêng.
