# VXKey — native Settings / tray / About review

Ngày: 2026-10-30. User đã resume bằng “Tiếp nào”. **Đã sửa và kiểm native subset bên dưới; chưa nghiệm thu toàn bộ bộ gõ.** Checkpoint dừng cũ được giữ tại [checkpoint lịch sử](ui_window_checkpoint.json); trạng thái hiện tại và fingerprint nằm trong [evidence mới](ui_window_evidence.json).

## Kết quả sửa

- Settings là toàn bộ client area của một native window. Bỏ modal/backdrop/gutter/border/shadow/header/X bên trong; native titlebar và close là frame duy nhất. Sidebar10tab và footer cố định; content cuộn riêng.
- App-scoped adapter trong `desktop/src/index.css`; canonical Settings vendor không sửa và integrity gate PASS. Nhịp paragraph/notice/section sửa rõ ràng, Typing notice không còn chạm heading. Macro toolbar/actions, app-ID form và policy actions bố trí lại; text fields tương ứng tắt autocorrection.
- Settings/About dùng Keyboard SVG tượng trưng; native About dùng artwork app RGBA512 với logical size512, version một lần, author/summary/GitHub/copyright. Credits centered, font11, word wrap; dùng standard NSApplication About panel từ menu và tray.
- Tray có native window/background/WebView/document trong suốt, bỏ native shadow chữ nhật; chỉ một border bo góc. ResizeObserver gọi host `vxkey_fit_tray`, chỉ tray, width320 và height280..640; quan sát thực tế320×376.
- Host ẩn tray trước khi mở native About; không đợi JavaScript continuation của hidden WebView. Escape và native Focused(false) đi qua hide. Panel refresh snapshot khi nhận native focus, tránh dùng locale/theme cũ đến nhịp polling3giây.
- Pure Black áp dụng dark control tokens; copy đã rút gọn, bỏ tuyên bố an toàn tuyệt đối/feature chưa có. Autostart vẫn OFF và unavailable.

Đã đồng bộ [spec03 v0.6](../spec/03_ui_wireframe.md), [contract07](../spec/07_implementation_contract.md), [index](../index.md), chuẩn suite [Settings v1.4](../../../_summary/standard_docs/settings_standards.md) và [desktop UI v1.14](../../../_summary/standard_docs/desktop_ui_standards.md). Specs ready vẫn chờ user.

## Bằng chứng native

Candidate development: `build/desktop_candidate/VXKey.app`, SDK27, QA data riêng `build/qa_integration_data`. Không phải signed production installer. SHA256 binary và danh sách ảnh đúng bản cuối nằm trong [evidence](ui_window_evidence.json). Các ảnh không có prefix `final_` là evidence ở lượt trước trong cùng đợt chỉnh UI; sau đó chỉ bổ sung native-focus refresh. Không gộp chúng thành một fingerprint.

| Case | Kết quả / ảnh |
| --- | --- |
| Startup bình thường | Mở Settings vi/system; không nested modal. QA override đã gỡ khỏi Info.plist khi bàn giao. |
| 10tab vi/dark bản cuối | Đã xem tất cả: [General](ui_window_native/resume/final_general_vi_dark.jpg), [Typing](ui_window_native/resume/final_typing_vi_dark.jpg), [Apps](ui_window_native/resume/final_apps_vi_dark.jpg), [Macros](ui_window_native/resume/final_macros_vi_dark.jpg), [Shortcuts](ui_window_native/resume/final_shortcuts_vi_dark.jpg), [Clipboard](ui_window_native/resume/final_clipboard_vi_dark.jpg), [System](ui_window_native/resume/final_system_vi_dark.jpg), [Updates](ui_window_native/resume/final_updates_vi_dark.jpg), [About SVG](ui_window_native/resume/final_about_settings_vi_dark.jpg), [Support](ui_window_native/resume/final_support_vi_dark.jpg). |
| Macro draft | [Dirty toolbar](ui_window_native/resume/macros_dirty_toolbar_vi_dark.jpg): Cancel/Save không vỡ từng chữ; Cancel rồi navigation thành công,124entries vẫn còn. |
| Populated app list | [Long ID](ui_window_native/resume/apps_populated_vi_dark.jpg): identity truncate, policy controls hàng riêng, content scroll không đẩy footer. Hai app-ID là fixture giả, không app thực. |
| English/light + min size | [System640×520](ui_window_native/resume/system_en_light_min640x520.jpg), [Typing top](ui_window_native/resume/typing_en_light_min640x520.jpg), [Typing scrolled](ui_window_native/resume/typing_en_light_min_scrolled.jpg). Sidebar/footer vẫn truy cập, lower controls hiện qua scroll. |
| Pure Black | [General](ui_window_native/resume/general_en_black.jpg), [Macros](ui_window_native/resume/macros_en_black.jpg): dark control colors giữ contrast. |
| Tray native | [vi/dark](ui_window_native/resume/final_tray_vi_dark.jpg), [en/light](ui_window_native/resume/final_tray_en_light.jpg):320×376, không outer rectangle. Screenshot là composited JPEG, không có bằng chứng alpha-channel; kết luận ở mức visual. |
| Tray refresh | Sau đổi English/light ở Settings, đóng Settings rồi reopen tray: state đầu tiên là English/light trên bản cuối. |
| Escape/About hide | Escape trả noWindowsAvailable; [host log](ui_window_native/resume/host_visibility.jsonl) ghi close_surface visible=false. before_about visible=false trước native panel. Focus_lost hide thấy khi mở Settings/About. |
| Native About | [vi](ui_window_native/resume/about_native_vi_final.jpg), [en bản cuối](ui_window_native/resume/final_about_native_en.jpg); từ tray và app menu, metadata rõ/version1lần. Panel dùng appearance dark của OS, không đổi OS setting. CUA reopen có thể chủ động show QA tray khi lấy state sau đóng panel; host log mới là bằng chứng hide. |

## Checks và trạng thái bàn giao

- `scripts/build_desktop_candidate.sh`: frontend integrity/TypeScript/Vite + offline Tauri build + SDK artifact gate PASS. [Build log](ui_window_build.log).
- `node --experimental-strip-types --test desktop/tests/macro_loader.test.ts`:2/2PASS, gồm pagination >50entries và không save partial dictionary khi page/revision lỗi.
- Không chạy lại full engine suites vì thay đổi UI; kết quả1185Cargo/38VXKey trước đó vẫn là evidence lịch sử trong [integration review](integration_review.md).
- App QA đã quit; developer service get_snapshot mở fixture thành công, xác nhận writer lock được giải phóng. Fixture đã trả vi/system, serious, autostartfalse;124macros và2policy giả giữ để tái hiện visual.
- Debug-only `VXKEY_QA_SURFACE` cần `VXKEY_DATA_DIR`, chỉ chọn native surface khi startup/reopen; release không có override. Bounded probe tối đa64KiB chỉ ghi event/label/visibility, không gõ phím/clipboard. Info.plist bàn giao đã bỏ selector.
- Không gọi Anti, không cài/activate IME, không đổi TCC/keychain, không commit/push/release, không cài /Applications.

## Còn mở

Chưa full locale × theme × tab matrix; native About light chưa thử; chưa thao tác trực tiếp icon SystemUIServer/click-outside sang app khác. Tool SystemUIServer bị timeout, không lặp lại hoặc đổi quyền; fixture mở đúng tray native thay thế chỉ cho kiểm surface. Focus-lost path đã quan sát nhưng không suy thành mọi tình huống click-outside.

Windows chưa chạy. IME bridge/authenticated IPC, current-app tokens, watchdog/recovery, global shortcuts, clipboard provenance/capture và daily-driver typing vẫn là các gate M2 trong integration review; visual companion không đóng các gate đó.
