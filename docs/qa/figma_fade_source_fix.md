# VXKey — nguồn sửa fade/Figma canvas, 2026-11-03

## Phạm vi

User chốt chỉ sửa VXKey, không đổi setting Figma vì bộ gõ khác dùng bình thường. Không đổi Figma accessibility/screenreader/document, không viết file design, không đổi quyền OS hoặc binary đang cài. Giữ yêu cầu trước: chưa build bundle; không commit/push/publish. Các thay đổi UI/release khác trong working tree được giữ nguyên.

## Thay đổi

### LOG_008 — timeout bridge và timestamp

`crates/vxkey-runtime/src/bridge/local.rs`: read/write deadline từ1giây thành HEALTH_TIMEOUT hiện3giây, khớp ngưỡng fresh/progress. Tất cả I/O vẫn trên bridge worker, không trên callback phím. Helper lease0.5giây giữ nguyên: mất kết nối vẫn fail-open nhanh, nhưng nhịp scheduling trễ1.15giây không bị latch cố định sớm từ socket timeout1giây. Không tự xóa latch/Resume hoặc respawn khi fault. Đây là sửa mismatch đã xác nhận ở source, **chưa chứng minh là nguyên nhân incident user báo**.

Marker recovery mới thêm `occurred_at_unix_ms` lúc ghi; marker đầu tiên không bị ghi đè, incident match vẫn bắt buộc trước clear. Không sửa marker production cũ. Test thêm first cause/timestamp/cancel-clear bảo toàn và UnixStream pair đọc chậm1.15giây thành công với deadline hữu hạn3giây.

### LOG_009 — cổng AX cho canvas Figma

`native/macos_background/context_guard.swift`: khi focus element không có subrole, trước đây chỉ AXTextArea được thử đọc range/slice. Thêm admission cho roleAXDocument/AXWebArea **chỉ khi appIDcom.figma.Desktop**. Các role này vẫn phải qua focused PID/element, secure-subrole check, collapsed selection, exact owned suffix và snapshot freshness/StreamWindow trước rewrite. Không fallback nội dung buffer thay AX, không đọc full AXValue hoặc đổi API sang blind Backspace.

**Giới hạn:** CUA trước chỉ cho thấy tree dạng document, chưa có receipt raw AXRole/selected range/string-for-range trong một phiên sửa text thực. Vì vậy đây là nhánh compatibility có phạm vi hẹp cần native trial, không gọi đó là root cause confirmed hay Figma đã được sửa hoàn toàn. Nếu editor expose role khác hoặc không có slice API thì vẫn fail-open và cần tiếp tục điều tra; không mở rộng role tùy tiện để ép pass.

Thêm diagnostic state transition bootstrap/owned_slice_verified/waiting_for_owned_slice/invalid_context vào Unified Log categorycontext_probe (info level, muốn đọc phải dùng log show --info). Rate limit tối đa1record/giây, không key/text/selection offset/file/node/app design. Không I/O file từ callback phím. Không log full AX tree, không lưu nội dung canvas.

## QA

- `cargo +stable test -p vxkey-runtime --offline`: **54tests PASS**. [Log](figma_fade_runtime_tests.log).
- `sh scripts/test_background_driver.sh`: **92Swift checks PASS**, compile Rust FFI/helper boundary. [Log](figma_fade_native_tests.log), [evidence](figma_fade_swift_evidence.json).
- `cargo +stable check --manifest-path desktop/src-tauri/Cargo.toml --offline`: PASS. [Log](figma_fade_backend_check.log).
- Test admission Figma đúng scope, document app khác/unknown single-line bị chặn, missing slice không verified, exact d resolves anchor. Toàn regression empty proof, secure context/proof epoch/stream limits trước đó giữ nguyên.
- Một lần test timestamp mới FAIL do tempdir permissions chưa700; sửa **fixture permissions**, không nới secure_directory runtime; rerun54PASS. Swift sandbox vẫn có warning AX connection, không dùng fixture làm runtime permission proof.

## Phần còn mở

LOG_008/009 vẫn MỞ. Chưa native trial hoặc bundle mới; app đang cài chưa có patch. Cần candidate riêng đúng identity/data/kênh test, xác minh từng role/range/slice state không thu text và gõ tay canvas user chọn; so với TextEdit/field khác. Không sửa Figma để thích nghi VXKey. Gắn incident timestamp để phân biệt fade do timeout bridge, tap/permission fault, user pause và bypass.

Không tuyên bố giữ quyền N→N+1 hoặc production/test đã tách hoàn chỉnh từ patch này. Track bảo trì dài hạn MỞ, Windows PLAN ONLY.

## Đã thay hướng compatibility theo user

Đặc cách appIDFigma/role được gỡ ở patch tiếp theo: [compatibility_rule_review.md](compatibility_rule_review.md). Current source dùng capability-based proof cho mọi app, kèm deferred reconciliation. Không dùng đoạn mô tả whitelist phía trên làm contract hiện hành; giữ nó làm lịch sử chưa native nghiệm thu.
