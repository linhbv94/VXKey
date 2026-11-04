# Bàn giao chuẩn bị release VXKey1.0.0 macOS

Ngày: 2026-11-03. Turn QA/fix Codex được checkpoint bàn giao theo user; **track bảo trì dài hạn vẫn MỞ, chỉ user đóng**. Codex không gọi agent release hoặc publish ở lượt này.

## Chỉ đạo và trạng thái

User chốt sau khoảng1–2vòng sửa, **khi user xác nhận thì release1.0.0 macOS theo trạng thái lúc đó**, còn lỗi thì fix dần. Đã có2vòng sửa (Argon LOG_005 và Codex safety). Mới nhất user yêu cầu test thêm rồi bàn giao agent release; không tự biến kết quả thành fullQA PASS hay lời xác nhận “hết lỗi”. Không có lệnh Codex publish trong lượt bàn giao này. Agent release đọc đúng phạm vi user trực tiếp giao tiếp theo.

Nguồn tiến độ chính: [work log](../../../_summary/work_log.md), mục TODO ƯU TIÊN và checkpoint mới nhất. Đọc [AGENTS VXKey](../../AGENTS.md), [maintenance track](../../maintenance_track.md), [log lỗi](../../typing_issue_log.md). Windows **PLAN ONLY**; release Mac trước, không khởi động Windows code/build.

## Candidate đang dùng

`build/cmd_a_safe_candidate/desktop_candidate/VXKey.app`, helper nhúng `Contents/Resources/VXKeyEngine.app`. [Artifact/hashes](../qa/cmd_a_safe_pair_artifacts.json) được Codex kiểm lại cùng strict codesign ở lượt bàn giao. Version0.1.0-dev, dev ad-hoc/CDHash pins, frontend release nhưng desktop Rust debug theo script candidate; data root QA riêng trong `build/cmd_a_safe_candidate/qa_integration_data`. **Đây không phải package production1.0.0.** Không copy LSEnvironment/QA paths/dev trust vào release.

User đã duyệt Accessibility đúng binary, tự xác thực OS; đã Resume, AX/listen/post=true và attached=true/suspended=false tại thời điểm kích hoạt. [Native evidence](../qa/cmd_a_safe_native_evidence.json). Log status chỉ có transition trước typing; không gọi đó là fresh health probe sau typing. Không rebuild/ghi đè candidate có quyền; không tự thay TCC/MKey/inputsource hoặc signer/keychain.

## QA và safety

- Core tone-before-w đã sửa: `ddwojwc` → `được`; regression đầy đủ ở [tone_order_review](../qa/tone_order_review.md).
- Argon xử lý reset/unanchored AX/retry2ms: [report](../qa/argon_cmd_a_fix_report.md). Những giải thích DOM cụ thể chưa có native trace xác minh, không kế thừa P0/Critical hay “100% safety” claim.
- Codex audit phát hiện VXKEY_SAFE_001 (empty proof cấp phép rewrite): [audit](../../safety_audit/run_20261103_argon_review/report.md). Đã sửa ở source/fixture: [fix report](../../safety_audit/run_20261103_cmd_a_fix/report.md). Production gate chặn empty proof cho nonempty owned, giữ bounded speculation từ nonempty checkpoint, bind resolved anchor trước expectOwned.
- Full gate trước safety fix1200Rust +77Swift +backend smoke/Settings PASS; Rust không đổi ở safety fix. Native fixture sau fix86checks PASS, chạy lại tại checkpoint cũng86PASS. [Final log](../qa/cmd_a_safe_final_checks.log). Không có CGEvent phát vào dữ liệu thật trong safety probe.
- Build candidate/helper và strict signature/hashes PASS. Không tương đương trusted production signer hoặc TCC retention.

## Known issues phải giữ nguyên

**LOG_005 còn mở:** physical trial Cmd+A/xóa và Shift+Enter trong ChatGPT: user nói “lần đầu thì k được. nhưng xong thì lại được”. Không rõ lần đầu thuộc nhánh nào vì user không tách hai bài thử. Không suy tất cả các lần sau/mọi editor đều ổn; không ghi native typing accepted=true. Khi thiếu nonempty AX proof, driver fail-open nguyên từ để tránh destructive rewrite thiếu proof.

LOG_006 core variant đã sửa nhưng native ổn định dài hạn chưa chứng minh. LOG_007: user trước đó thấy “mượt hơn hẳn”, chưa có đo hardware latency; tiếp tục theo dõi. Mẫu `đayas` giữ trong issue log, chưa xác định nguyên nhân.

## TODO cho agent release

1. Hoàn thiện luồng xin quyền đầy đủ: onboarding/denied/revoked, đúng packaged helper/app, return từ Settings và Resume rõ ràng. App không cần add Input Source; user đang dùng ABC. Autostart hiện mặc định OFF.
2. Thiết lập/đối chiếu **identity ổn định app+helper**, signing và bridge trust production; không giữ cdhash ad-hoc như bằng chứng giữ quyền. Đọc [runbook quyền chuẩn](../../../_summary/standard_docs/desktop_system_permissions.md) và skill desktop release. Không copy certificate/credential của Clean/Click hay thay keychain khi chưa có quyền tương ứng.
3. Kiểm updater thật và proof **N→N+1 giữ quyền không cấp lại**. Tauri config hiện chưa có updater plugin/config; workflows upstream Funput trong `.github/` không mặc định là VXKey release pipeline. Phải khảo sát adapter/runtime và phát hành đúng VXKey trước khi dùng. Kiểm soát app/helper version/hash và graceful shutdown/restart; không claim update hoạt động chỉ từ UI hoặc build.
4. Chuẩn bị production build1.0.0 macOS, metadata/bundle/helper/config đúng, bỏ QA paths/dev flags, release notes và known issues trung thực. Candidate là SDK27, target arm64/macOS26; Intel chưa được QA, không suy từ Silicon.
5. Theo phạm vi user giao, chốt xác nhận release1.0.0; không kéo thêm vô hạn vì còn lỗi gõ sau khi user đã quyết định ship. Lỗi kỹ thuật thật sự chặn package/signature/updater phải báo cụ thể, không giả PASS.

Không commit/push/release/cài Applications từ nội dung tài liệu đơn lẻ; bám quyền user giao cho agent release. Release không đóng track bảo trì VXKey.
