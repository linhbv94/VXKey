# Tiếp nhận bản sửa Argon — w mặc định và prefix đầu từ

Ngày: 2026-11-02. Owner kiểm chứng/bàn giao: Codex. Không gọi thêm Anti, không triển khai Windows.

## Đã đọc và đối chiếu

User tự giao Argon sửa; work log bàn giao LOG_001/LOG_002. Codex đã đọc diff core, native ContextGuard/EventDriver và tests; giữ nguyên thay đổi nguồn của Argon trong lượt này.

- Telex thường và nâng cao dùng chung shortcut w/W→ư/Ư và undo ww. Brackets vẫn chỉ thuộc nâng cao. Thay đổi nghĩa của w trước nguyên âm là chủ ý theo yêu cầu user; các corpus w-permutation cũ được điều chỉnh để không coi w trước nguyên âm là cùng một modifier như trước.
- Native reset cho event đi nguyên phím có thể trì hoãn probe tức thì. Epoch rỗng bị revoked có đường re-prime.
- ACTION_NONE có thể ghi prefix phím gốc trong RAM khi chưa neo caret. Worker chỉ đọc đúng suffix có độ dài prefix đã theo dõi để neo start; không scan cả từ/document. Trước mọi batch ACTION_SEND vẫn cần verified snapshot và start>=0.
- Prefix neo bất đồng bộ vẫn có cửa sổ thời gian; không coi tests giả reader là native first-word đã hết lỗi. Trường hợp w lẻ ngay sau thao tác focus cũng phải thử thật.

## Kiểm tra độc lập

[verification log](word_handoff_verification.log): sh scripts/test_engine.sh PASS. [Rust log](engine_cargo_tests.log): tổng1.198test/doctest passed,0failed qua39summary suites trong script. [Swift evidence](background_edit_evidence.json):61assertions PASS ở lượt Codex. [Backend smoke](backend_smoke.json), Settings gate PASS.

[Build SDK27](word_pair_build.log), [artifact hashes](word_pair_artifacts.json) và strict signature verification PASS. Các kết quả này không xác nhận runtime typing. Warning HIServices trong fixture headless không được coi là native PASS.

## Candidate và dữ liệu

build/word_candidate/desktop_candidate/VXKey.app, embedded VXKeyEngine.app. Root riêng; copy config/recovery từ QA checkpoint_candidate, config26/Telex thường/tone modern/autostart OFF; không copy file driver_probe_mode. Không rebuild/ghi đè binary checkpoint_candidate đang có quyền. Không thay signer/keychain/TCC/input source hoặc app khác.

## Native gate tiếp

Sau approval đúng binary nếu thay entry Accessibility:
1. Quit candidate cũ, thay riêng entry VXKey, mở word_candidate; xác minh AX/listen/PostEvent, explicit Resume.
2. User foreground TextEdit, gõ tay w/W/thw/vwng và thwr/nhwng/chwx/vwaf; không paste.
3. Gõ casi/casch/cachs đầu dòng, ngay sau Enter, click và chọn-xóa; thử tốc độ thường và nhanh.
4. Câu dài, kéo dài chữ, tiếng Anh với ww undo; Pause chốt counters.
5. Chỉ cập nhật LOG_001/LOG_002 thành native đạt đúng các case đã thử; track bảo trì dài hạn luôn mở. WebView/focus/secure/Undo/compatibility vẫn là gate riêng.

Không cần user gõ chậm để né lỗi. Không sửa kiểu đặt dấu hiện hành; không tự đóng track. Windows vẫn PLAN ONLY.
