# 06. Demo presentation và prototype clickthrough — VXKey

> **Skill:** `_spec6_demo_presentation`  
> **Version:** 0.4.0 — **Ngày cập nhật:** 2026-10-30  
> **Trạng thái:** Đã soạn để review; chưa được user xác nhận specs ready.  
> **Đối tượng:** Dev, UI/UX, QA và AI Execution Agents.  
> **Nguồn scope:** [Tổng quan](00_system_overview.md); quyết định UI Tauri chung Mac/Windows của user.

## 1. Chuẩn bị

Demo sau khi có candidate và QA phù hợp; hiện chỉ là kịch bản. Ghi commit/hash/version/OS/arch, giữ ABC/Funput làm đường quay lại. Dùng account/fixture cô lập và text giả; không mở clipboard/password thật. Không cài đè production để quay demo. Demo UI-only phải gắn nhãn prototype, không claim native typing.

Seed text: `Xin chào Việt Nam`, multiline `Dòng 1
Dòng 2 🙂`, và sentinel `goKEEP_THIS_SUFFIX`. Macro fixture `vn` → `Việt Nam` chỉ bật trong phần demo explicit rồi tắt. Hai app A/B có exact IDs ghi trong run report; không bịa ID app fixture thành app thật.

## 2. Kịch bản 6 phút

| Mốc | Thao tác | Kết quả / case |
|---|---|---|
| 00:00–00:40 | Mở launcher, Settings, xác nhận UI Tauri/version | Metadata thật, một Settings owner; Q12 |
| 00:40–01:30 | Chọn VXKey source qua OS, gõ Telex/VNI trên TextEdit fixture | Text đúng; icon VI/EN đúng, không mất prefix/suffix; Q01/07 |
| 01:30–02:10 | Đổi mode A/B, bật bypass A rồi thử toggle | Memory độc lập, bypass giữ nguyên; Q03/04 |
| 02:10–02:50 | UI English → Tiếng Việt, theme, đóng/mở | Locale/theme persist, chế độ gõ không đổi; Q12 |
| 02:50–04:10 | Consent history session, copy multiline mới, search/preview/Enter | Exact copy-back, user tự Cmd+V; Q13/14 |
| 04:10–04:40 | Pause, copy fixture mới, Resume | Không backfill; Q13 |
| 04:40–05:20 | Chuyển ABC, mở history; đóng companion trong fixture | History dùng dưới ABC; IME độc lập khi companion dừng; Q08 |
| 05:20–06:00 | Mở lại, trình bày rollback và known limits | Không hứa Win/Intel/auto-paste/persistence chưa đạt |

Không demo encrypted persistence nếu chỉ M4a; nếu M4b đã đạt thì thêm restart/lock/key-error fixture và không lộ key. Update demo riêng cần hai signed artifacts version khác thật, không fake progress.

## 3. Prototype clickthrough

Launcher → onboarding → source status → Settings General → Typing → Applications/picker → clipboard consent → history empty → populated → paused → Settings Updates dev state. Hotspot tương ứng command trong data spec; prototype phải hiển thị mock badge, không gọi OS/clipboard thật.

## 4. Contingency và cleanup

Typing lỗi/mất chữ → dừng case, chuyển ABC bằng OS, giữ evidence fixture, không thử trên tài liệu user. IME chưa active → demo UI với nhãn giới hạn, không giả native pass. Permission denied → demo denied/recovery state, không reset TCC. Key lỗi → pause/error, không plaintext fallback. Copy race → giữ panel/retry do user, không overwrite tự động. Xong demo tắt capture, clear fixture history theo mode, trả app policy/macro fixture về trạng thái trước và ghi cleanup; không xóa dữ liệu user hoặc tự uninstall.

FAQ: Tauri dùng chung UI, engine adapter khác; VI/EN không đổi UI locale; history copy-back chưa auto-paste; session history mất khi thoát; filtering không nhận diện mọi secret; clipboard OS/Spotlight/Handoff ngoài kiểm soát VXKey.


## 5. Audience, thông điệp và talk track

Đối tượng: user duyệt sản phẩm và dev/QA nhận bàn giao. Thông điệp: UI thống nhất, bộ gõ vẫn độc lập UI, history có consent và copy-back rõ. Điểm thể hiện chính là mở cùng Settings cho cả platform adapters và IME vẫn hoạt động khi companion dừng trong fixture; không trình diễn Windows runtime khi chưa có.

| Hồi | Screen/flow | Lời thoại gợi ý | Proof cần thấy |
|---|---|---|---|
| Vấn đề | S03/FL01 | “Mình cần biết bộ gõ đang thực sự hoạt động trong app nào.” | Source readback, version/hash |
| Giải pháp | S01/S02, FL02..05 | “Chế độ gõ, ngôn ngữ UI và quy tắc bỏ qua app được quản lý riêng.” | Q03/Q04/Q12 |
| Kết quả | S05, FL06..08 | “Chỉ những bản copy mới đủ điều kiện mới vào history; Enter sao chép để mình tự dán.” | Q13/Q14 và session mode status |
| Giới hạn | S07/FL09 | “Bản này mới đạt những platform và milestone ghi trong report.” | Actual QA report, không slide PASS tự viết |

## 6. Prototype hotspot map

| Hotspot | From → to | Guard / effect | Back/cancel |
|---|---|---|---|
| Open Settings | S01→S02 | Single owner; mở tab cuối hợp lệ | Close trả focus hợp lý |
| Start onboarding | S02→S03 | Read-only status check, không chọn source tự động | Để sau trở về Settings |
| VI/EN | S01→S01 | Valid target; error variant stale_target | Không mutation khi stale |
| Add application | S02→S04 | Metadata preview exact ID | Cancel không tạo rule |
| Enable history | S02→consent→S05 | Chỉ explicit consent tạo baseline mới | Cancel giữ off |
| History item | S05 list→preview | Only selected entry data | Back list trên layout hẹp |
| Copy | S05→hidden | Chỉ sau success; fixture preview nếu mock | Error giữ S05 |
| Update CTA | S07→progress | Production real job only | Dev unsupported có explanation |

Prototype sử dụng local fixture adapter với badge “Prototype”; mỗi hotspot có variant loading/error/empty theo03, không gọi clipboard hoặc install thật. Không có Figma file/selection từ user, nên chưa có tọa độ node/hotspot Figma; mapping screen/action này là contract để dựng prototype sau.

## 7. Presentation storyboard và visual prompt

| Slide | Nội dung | Visual / tài liệu |
|---|---|---|
|1 | Gõ ổn định, UI thống nhất | Native IME + Tauri architecture từ00 |
|2 | Hành trình setup → gõ → app policy | Swimlane01 rút gọn, highlight user lane |
|3 | Settings và history | Wireframes03 light/dark, không giả ảnh runtime |
|4 | Điều kiện chất lượng | QA gates05 và support matrix thực có |
|5 | Bước tiếp | Milestones07, decisions chưa chốt |

```text
Create a clear 16:9 product walkthrough slide for VXKey. Show a shared Tauri UI layer above two distinct native input adapters, macOS first and Windows later. Use the VX navy/slate surfaces with cyan accents and restrained system typography. Highlight that typing remains inside the native input process while clipboard history and Settings belong to the companion. Label design mockups as mockups; avoid performance claims, certification badges or unsupported Windows screenshots.
```

Không cần tạo PPT/ảnh/video trong task viết spec. Demo runtime chỉ sau triển khai/QA; demo chưa phải acceptance hoặc quyền install/release.

## 8. Demo bổ sung — Current-app exclusion

Ở bước app policy: focus ChatGPT hoặc editor fixture → mở tray → chỉ rõ action mang tên app đó → click Tắt VXKey cho app → gõ fixture để thấy bypass → mở lại chọn Bật VXKey cho app. Không mở Settings ở thao tác đầu; sau đó mở Applications để xác nhận cùng rule. Nêu rõ clipboard policy vẫn độc lập và bật lại dùng ngôn ngữ đã nhớ. Native demo sau implementation đối chiếu Q25..Q27; ảnh MKey là reference UX, không proof VXKey đã có.

## 9. Demo parity và dấu chấm hai Space

Settings chọn Telex nâng cao → gõ vw ng liền thành `vwng` trong fixture để thấy `vưng`; đổi lại Telex thường thử `vuwng`. Demo extra-onsets, smart-case và flip chỉ dùng dữ liệu fixture. Chuyển mã mở S08, preview và save-copy, giữ hash originals.

Mac native period on → mở Settings VXKey thấy toggle disabled/helper macOS; user tự tắt native trong System Settings → VXKey toggle enabled và default false trên fresh profile → bật rồi thử hai Space. Bật lại native khi VXKey đang mở → khóa VXKey ngay, không chèn dấu chấm đôi. Windows demo sau backend thật: fresh profile default true. Không tự đổi setting OS chỉ để quay demo, chưa có implementation thì ghi prototype.
