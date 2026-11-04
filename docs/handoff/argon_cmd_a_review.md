# Bàn giao Argon review — lỗi từ đầu sau Cmd+A/xóa

Ngày: 2026-11-02. User tự giao Argon xem một vòng; Codex không khởi chạy agent. Đây là lượt review, chưa phải yêu cầu sửa nguồn hay build.

## Chỉ đạo mới nhất — TODO release 1.0.0

User muốn thêm khoảng 1–2 vòng sửa rồi **khi user xác nhận thì release macOS 1.0.0 theo trạng thái lúc đó**, còn lỗi thì tiếp tục fix sau. Chưa được release ngay; review không tự tính thành một vòng sửa. Đọc mục **TODO ƯU TIÊN** trong [work log](../../../_summary/work_log.md) để giữ bộ đếm xuyên agent/chat. Chuẩn bị luồng xin quyền đầy đủ và proof auto-update N → N+1 giữ quyền, tránh trải nghiệm mất quyền user phản ánh ở Clean/Click; chưa audit hai app đó trong lượt này. Windows vẫn PLAN ONLY.

User bổ sung lỗi chữ đầu **sau Shift+Enter**, cần review cùng Cmd+A/xóa. Không đồng nhất lỗi chữ đầu với lỗi quyền khi chưa có evidence.

## Đọc trước

1. [Quy tắc VXKey](../../AGENTS.md), [track dài hạn](../../maintenance_track.md).
2. [Log lỗi user](../../typing_issue_log.md): LOG_005/006/007 và mẫu `đayas` giữ nguyên.
3. [Sửa core gần nhất](../qa/tone_order_review.md), [native evidence](../qa/tone_order_native_evidence.json), [artifact đúng binary](../qa/tone_order_pair_artifacts.json).
4. [Spec pipeline](../spec/10_checkpoint_pipeline.md), [background driver](../spec/09_background_driver.md), [latency review](../qa/latency_review.md).

## Kết quả mới nhất — ground truth từ user

Sau khi tone_order_candidate đã có quyền và Resume, user gõ tay trong ChatGPT:

```text
ddwojwc được được mượt
```

User xác nhận: “cmd A xong bị cái chữ đầu nhé. và tôi thấy nó mượt hơn hẳn đấy.”

- Từ đầu sau Cmd+A → xóa vẫn lọt raw `ddwojwc`; các từ tiếp theo đã chuyển dấu trong bài thử này. Đây là lời user xác nhận, không phải kết luận nguyên nhân hoặc trace từng phím.
- Cảm giác mượt hơn là xác nhận chủ quan đáng ghi nhận; chưa có số đo hardware-to-paint hay bằng chứng xác định nguyên nhân. Không đóng LOG_007.
- Core đã sửa được lỗi thứ tự `ddwojwc` → `đuọwc`, tái hiện độc lập trước sửa và qua regression sau sửa. Lượt sửa này chỉ đổi Rust core, không đổi Swift timing/pipeline. Lượt latency trước đó đã đưa TCC preflight sang worker; đừng quy chắc cảm giác mới cho một thay đổi riêng.
- Mẫu trước user yêu cầu giữ nguyên: `đayas`; chưa có chuỗi raw đủ xác định nguyên nhân, không tự sửa báo cáo.

## Binary và trạng thái

- App hiện hành: `build/tone_order_candidate/desktop_candidate/VXKey.app`.
- Helper nhúng: `build/tone_order_candidate/desktop_candidate/VXKey.app/Contents/Resources/VXKeyEngine.app`.
- Data root: `build/tone_order_candidate/qa_integration_data`.
- User đã duyệt thay riêng entry VXKey Accessibility; Settings UI ON. Helper xác nhận AX/listen/post=true, tap attached=true, Resume ACK applied, suspended=false, fault=null tại lúc kiểm tra. Không coi snapshot này là runtime hiện hành mãi mãi.
- Input Source không cần thêm, user dùng ABC. MKey đã được user tạm ngừng gõ Việt; không tự đổi MKey, quyền hay input source.
- Không rebuild/ghi đè binary đã có quyền. Dev ad-hoc pins không giữ TCC qua rebuild.

## Phạm vi review đề nghị

**Chỉ tham vấn/review chỉ đọc; không sửa nguồn; không gọi agent khác.**

Tìm nguyên nhân từ đầu bị bypass sau Cmd+A/xóa trong khung ChatGPT. Review vòng đời epoch/owned prefix/AX selection, bootstrap đầu từ và nhánh bỏ từ; tìm race nào khiến chờ Space mới phục hồi. Không khẳng định finding chỉ từ grep; lần theo caller và nêu evidence phản bác/giới hạn.

File cần xem:

- `native/macos_background/event_driver.swift`: `reset(immediate:)`, `abandonWord`, `verifyPending`, modifier/mouse/Backspace handling, ACTION_NONE bootstrap và ACTION_SEND guard.
- `native/macos_background/context_guard.swift`: mailbox/worker, `revoked`, snapshot epoch, focus identity, suffix verification khi start chưa neo.
- `native/macos_background/stream_window.swift`: revision ACK, timeout và giới hạn phím pending.
- `native/macos_background/permission_cache.swift`: phân biệt nhánh permission cache với mất proof theo từ.
- `native/macos_background/edit_plan.swift` và các file fixture dưới `native/tests/background_driver/`; `scripts/test_background_driver.sh`.
- Core fix hiện tại: `crates/funput-core/src/composition/uo_horn/continuation.rs`, `composition/transform/action.rs`, `uo_horn/mod.rs`; regression trong `tests/telex/w_permutations.rs` và `crates/funput-ffi/tests/advanced_telex.rs`.

Nếu tên file/path đã đổi, dùng rg để định vị; không tạo bản chuẩn mới.

## Invariant cần giữ

- Không phát Backspace/rewrite vào vùng không sở hữu, selection khác hoặc app/focus đã đổi; không bỏ proof check để chữa mất dấu.
- Không chặn bàn phím khi fault; giữ fail-open và khả năng Pause/Quit. Không quay lại global suspension cho lỗi proof một từ nếu không có lý do đã chứng minh.
- Không đọc full field/global raw keystroke log để điều tra. Probe page chỉ ghi trong fixture khi user tự Start; capture có giới hạn, synthetic events không tương đương raw hardware keys.
- Tool đã từ chối mở file:// typing_probe.html; không vòng qua localhost/Finder/CDP để bypass. User từng tự mở và gửi trace.
- Native tool có thể gửi phím vào app không thật sự foreground; kết quả đó không chứng minh per-PID hook. Physical typing user mới là evidence gần nhất.

## Checks đã có và giới hạn

- Full gate: 1199 Rust tests/doctests, backend isolated smoke, 67 Swift assertions và Settings source/CSS/copy PASS. [Log](../qa/tone_order_verification.log).
- C ABI bổ sung: 320 từ/mode × 2 modes PASS. [Log](../qa/tone_order_ffi_tests.log).
- Strict codesign candidate PASS; không thay native typing acceptance hoặc TCC retention proof.
- Chưa có test xác định tái hiện Cmd+A/xóa trên ChatGPT qua pipeline thật; fixture PASS không phủ hết race runtime.

## Output cần bàn giao lại

Một report ngắn: finding theo mức độ và file/function, bằng chứng, giả thuyết chưa xác nhận, hướng sửa nhỏ nhất giữ invariant, test tái hiện cần có. Ghi đường dẫn report và kết luận vào [work log](../../../_summary/work_log.md) để Codex đọc tiếp. Không sửa source nếu user chỉ giao review; nếu user giao khác, bám đúng phạm vi mới và ghi rõ.

Track bảo trì dài hạn luôn MỞ, chỉ user đóng. Windows PLAN ONLY; không commit/push/release/cài Applications, đổi TCC/keychain hay tự gọi agent. Giới hạn CLI global vẫn áp dụng nếu sau này thực sự được giao điều phối; bàn giao thủ công này không tự cấp thêm vòng cho các task CLI cũ.
