# 09. Bộ gõ chạy nền — hợp đồng macOS

> 2026-11-02: [checkpoint pipeline](10_checkpoint_pipeline.md) supersedes per-key verification fault policy; native acceptance pending.

Version: 0.8.0 — Ngày cập nhật: 2026-11-02
Skill: `_spec0_system`, `_spec4_api_data`, `_spec5_qa_acceptance`.
Trạng thái: quyết định kiến trúc của user; chưa phải nghiệm thu runtime hoặc specs ready.

## Quyết định và phạm vi

VXKey là tiện ích chạy nền với một tray và Tauri Settings. Người dùng giữ nguyên ABC hoặc nguồn nhập đang dùng, không thêm/chọn VXKey trong Input Sources. Rust Funput core/engine/C ABI được giữ; thay InputMethodKit bằng Swift/CoreGraphics keyboard driver. Không dùng JavaScript để xử lý phím. Windows giữ hướng hook native, dùng chung engine/config/UI khi triển khai sau.

InputMethodKit là hướng thử đã hủy. Các hướng dẫn cài `VXKeyInput.app` vào Library/Input Methods, TISRegister/Enable/Select và fallback chọn ABC không còn là workflow sản phẩm. Các báo cáo IME trước ngày này là bằng chứng lịch sử, không nghiệm thu driver mới. Không đăng ký nguồn nhập, không thay keyboard layout, không dừng MKey hoặc bộ gõ khác.

## Cấu trúc và ownership

`VXKey.app` (com.vxkey.desktop) chứa `Contents/Resources/VXKeyEngine.app` (com.vxkey.engine). Companion tự khởi chạy đúng helper được đóng gói; không yêu cầu user mở hai app. Helper giữ lock một instance cho mỗi data root. Companion quản lý config/persistence/clipboard; helper giữ composer và bộ đệm từ trong RAM, không gửi phím/văn bản/AX content qua bridge hoặc log.

Status `driver` chứa accessibility/listen_access/post_access/attached; `active` chỉ external target hợp lệ, `driver_status` mới phản ánh tap. Tauri `vxkey_request_permissions` chỉ Settings; ACK không đồng nghĩa đã cấp quyền. Bridge protocol 2 dùng component `engine`, `engine_session_id`, PID và focus_generation. Peer authentication vẫn dùng Security.framework và identity/pins; IME cũ bị từ chối. Không nới trust để chạy helper mới. Ad-hoc development pins không chứng minh quyền được giữ qua update.

## State và luồng

Launch → suspended → nhận config hợp lệ → user Resume → kiểm quyền và tạo tap → readiness ACK → host xóa durable latch → tick xác thực mới → chạy. Thiếu quyền: giữ suspended, hiển thị hướng dẫn quyền. Không tự bật lại sau timeout, lỗi engine, mất bridge hoặc restart sau fault. Autostart mặc định OFF.

Active filtering tap yêu cầu Accessibility/post access; listen access được đọc riêng, không tự xin quyền không cần thiết. Driver dùng per-application event tap (CGEventTapCreateForPid) và main run loop; callback chỉ dùng cache, engine và edit plan hữu hạn. Không disk/socket/AX IPC hoặc NSWorkspace query đồng bộ trong callback. Tap đăng ký trực tiếp cho PID external hiện hành và được gắn lại khi focus đổi; không đòi eventTargetUnixProcessID vì field này có thể bằng0 ngay trên annotated tap. Callback chỉ xử lý nếu registration PID khớp cached target, AX worker còn xác minh system-wide focused app. Unknown/mismatch chuyển nguyên phím. Focus cache phải publish PID trước khi prime AX context. Native UI/tray không giữ runtime mutex khi dispatch main-thread API. AX worker đọc focused element/selection và đúng phạm vi từ do VXKey sở hữu, không đọc toàn document hay clipboard. Snapshot cũ, focus đổi, selection khác, secure field, Secure Input, app bị bypass hoặc VXKey UI: chuyển nguyên phím và reset ownership. Bypass đứng trước macro/retone/engine.

Phím có Command/Control/Option thông thường và navigation được chuyển nguyên trạng. Ctrl+Option mặc định là gesture nhả hết modifier, bị hủy nếu có phím khác; custom shortcut vẫn theo config. Vi/En và nhớ app độc lập hard bypass. Foreground target lấy từ NSWorkspace, giữ last external app khi mở tray để loại trừ app đó.

Khi engine yêu cầu rewrite, chỉ xóa phần suffix đã xác minh thuộc từ hiện tại, caret collapsed và snapshot còn hạn. Không suy số Backspace từ UTF-16; chỉ chấp nhận scalar/grapheme tương ứng. Chuẩn bị đủ event batch trước khi nuốt phím gốc. Event tự phát có nonce/PID riêng và gửi đúng PID đích. Không phát modifier tổng hợp. Sau rewrite cần AX readback; không chắc chắn → suspend, không replay mù. Các app không cung cấp AX range đáng tin được pass-through, phải ghi coverage thật.

## Quyền và phục hồi

Kiểm Accessibility và quyền listen/post theo API macOS. Accessibility và PostEvent phải kiểm/hiển thị riêng: AXIsProcessTrusted không thay CGPreflightPostEventAccess. Nút quyền gọi AXIsProcessTrustedWithOptions và CGRequestPostEventAccess; request PostEvent chạy ngoài main input run loop. Chỉ request bằng hành động rõ ràng của user; quyền trong System Settings do user cấp. Không sửa TCC DB. Agent chỉ thay/bật đúng entry qua Settings UI khi user đã xác nhận rõ đúng candidate và quyền; approval cho binary trước không tự chuyển sang binary mới. Trên máy QA hiện tại macOS liệt kê entry **VXKey**, không phải VXKey Engine; UI/hướng dẫn phải dùng tên app OS thực tế, không yêu cầu user tìm entry không tồn tại. UI phải phân biệt helper connected, permissions và tap attached; không gọi chúng là Input Source active. Pause invalidate/tháo tap, không disable rồi reuse port; process chết khiến tap không còn xử lý. Companion chỉ được quản lý child do chính nó khởi chạy. Không đổi layout như phương án phục hồi.

## Acceptance contract

- BG01: Given ABC đang chọn, when mở VXKey, then không thêm/chọn input source và Settings/tray hoạt động.
- BG02: Given chưa cấp quyền, when Resume, then không nuốt phím, không báo ready; có trạng thái thiếu quyền.
- BG03: Given driver đã hoạt động, when gõ `tieengs Vieetj`, then đúng text thực trong TextEdit và app WebView; kiểm bằng phím vật lý/automation từng phím, không paste.
- BG04: Given prefix/suffix sentinel, emoji/combining/selection/focus đổi, then không sửa ngoài ownership; chưa chứng minh thì literal pass-through.
- BG05: Given app bypass hoặc secure input, then macro/retone/double-space đều không chạy.
- BG06: Given tap timeout, bridge/companion/engine mất hoặc sleep/wake, then raw keyboard vẫn dùng được, latch giữ suspended và chỉ explicit Resume mới phục hồi.
- BG07: Given Pause/quit/relaunch, then không duplicate hook, không chặn key-up/modifier; không ảnh hưởng MKey/layout.
- BG08: Kiểm latency và fast typing; fixture không thay native acceptance. Double-space vẫn Mac OFF/Win ON và native interlock như spec cũ; chưa có edit proof thì không bật capability.

## Demo và giới hạn

Mở một app → mở tray → đúng tên app, loại trừ ngay → bật lại → thử VI/EN. Trong Settings/System xem connection/permissions/driver; cấp quyền bằng nút/hướng dẫn rõ ràng, Resume rồi thử gõ và Pause. Không hướng dẫn user vào Input Sources. QA native, permission retention, production signing và Windows phải có evidence riêng trước daily-driver/release.

Try-lock busy của bridge chỉ giữ lease cũ còn hạn tối đa0.5s, không giả disconnect hoặc làm reset composer từng tick; contention quá hạn → suspend/latch.

### Delta native edit stream — 2026-11-02

Batch thay thế phải post qua proxy của callback (`CGEvent.tapPostEvent`), tại cùng điểm trong stream; không gọi postToPid cho rewrite của callback hiện hành. Không giữ proxy để sử dụng ngoài callback. Sau batch vẫn cần AX proof; nếu phím vật lý tiếp theo tới mà batch trước chưa verify, suspend và pass phím đó, không dùng tiếp owned word cũ. Timeout verification phải có reason riêng. Đây là contract source đang sửa; chưa có native acceptance cho stream_candidate.
