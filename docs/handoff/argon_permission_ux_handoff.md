# Bàn giao Argon — hoàn thiện UX xin quyền và chạy nền VXKey

Ngày2026-11-03. User không nghiệm thu cách Codex xử lý luồng này và sẽ tự giao Argon. Codex dừng sửa code tại đây, không gọi agent. Các mô tả giải pháp dưới đây là hướng bàn giao cần kiểm chứng, không phải UX đã được user duyệt hoặc tính năng đã hoàn tất.

## Vấn đề user thực sự gặp

- Mở app không được dẫn rõ vào luồng xin quyền; user tưởng phải tự vào Settings/add app.
- User cấp quyền rồi quay lại không biết phải làm gì. Card General chỉ có boolean Accessibility/listen và connected; Resume disabled nhưng không giải thích ngay quyền còn thiếu.
- Trạng thái “Đang dùng/Sẵn sàng” bị giấu ở Settings → Hệ thống. Codex đọc AX ở tab này rồi báo user như thể hiển thị rõ ngay trước mắt: **đó là sai sót giao tiếp và thiết kế luồng**, không phải user không biết dùng.
- Các nút Yêu cầu quyền/Kiểm tra lại/Resume không style, thiếu phân cấp, tên lẫn Việt/Anh, không dẫn tới bước kế tiếp.
- Đóng hộp thoại/cửa sổ còn icon Dock, không đúng utility chạy ở menubar.
- User nói UI tổng thể xấu nhưng hoãn redesign rộng; ưu tiên hoàn thiện luồng quyền/trạng thái/Dock trước.

## Production và source phải tách rõ

Production đã public1.0.0, user đang mở `/Applications/VXKey.app`. Trong lượt kiểm CUA, General báo AX/listen=true, connected nhưng Resume disabled; Hệ thống báo post_access chưa cấp. Sau request và thoát/mở lại app, Resume khả dụng; Codex bấm rồi Hệ thống báo Đang dùng/Sẵn sàng, AX/post đã cấp. Đây là snapshot lúc kiểm, **không phải proof người mới cài sẽ tự đi được luồng**, không đo gõ thật, không xác minh quyền giữ qua update. Không thay TCC/keychain/app bytes ở lượt đó.

Source working tree có hotfix của Codex **chưa đóng gói/cài hoặc visual/native nghiệm thu**. Không tuyên bố app đang cài đã có hotfix. [Report/check logs](../qa/permission_startup_review.md).

Files Codex đã sửa:

- `desktop/src/components/permission_setup.tsx` (mới): full-content Settings setup, status từng quyền, primary CTA theo trạng thái, Để sau, hướng dẫn restart bằng chữ. **Chưa có nút restart, chưa có ô gõ thử.**
- `desktop/src/app.tsx`: auto chuyển setup khi runtime báo thiếu quyền; giữ SettingsView mounted/hidden để handler close/quit hoạt động. Cần review race khởi động helper/cache quyền và hành vi mở lại/Để sau/revoke.
- `desktop/src/components/settings_view.tsx`: thay raw card General bằng tóm tắt và nút mở setup. Cần review trạng thái/busy/disabled/hướng dẫn đúng.
- `desktop/src/index.css`: app-owned permission button style; chưa visual QA, phải đối chiếu tokens/Settings chuẩn, không coi CSS mới là thiết kế đã duyệt.
- `desktop/src-tauri/src/lib.rs`: thêm macOS Accessory activation policy tại setup. Chưa native nghiệm thu close/reopen/menu/Dock.
- `native/macos_background/runtime_settings.swift`: sau explicit RequestPermissions mở URL Accessibility cố định bằng NSWorkspace. Không sửa permission cache/CGEvent pipeline. Cần verify OS prompt tạo đúng entry cho app/helper production và thông báo cần restart.

Giữ thay đổi icon/version1.0.1/signing metadata/pipeline của agent release. [Release1.0.1 checkpoint](release_1_0_1_checkpoint.md): user đã dừng release để fix thêm, chưa được tiếp tục publish chỉ vì nhận packet này. Không reset/stash/overwrite công việc khác.

## Outcome Argon cần làm và tự nghiệm thu

Luồng người mới tải app phải dễ đi từ đầu tới gõ được, không cần người vận hành/agent mở tab bí mật:

1. Mở lần đầu thấy thiết lập VXKey, giải thích ngắn lý do xin quyền và một hành động chính.
2. User bấm cấp quyền: dẫn đúng System Settings, nhận diện đúng entry app/helper, hướng dẫn bật/xác thực trực tiếp trên OS. Không tự sửa TCC hoặc xin quyền toàn máy ngoài scope.
3. Trở về app: tự kiểm trạng thái fresh, chỉ rõ quyền nào còn thiếu; từ chối/Để sau có đường quay lại, không popup loop, không giả granted. Nếu cần restart, hướng dẫn và nút restart phù hợp thay vì bắt tự đoán.
4. Đủ quyền: một CTA Bắt đầu gõ tiếng Việt; chờ ACK/native active đúng session rồi báo ngay tại màn hình này VXKey đang hoạt động. Lỗi/timeout có hướng xử lý; không spinner/disabled vô hạn.
5. Có vùng gõ thử và xác nhận Xong, sau đó menubar tiếp tục hiển thị trạng thái dễ hiểu. Thiết kế gõ thử phải kiểm thật engine, không fake chuyển Telex phía UI hoặc tự coi paste là native typing.
6. Đóng Settings/setup → menubar utility, không còn icon Dock. Mở lại từ menubar/Finder đúng cửa sổ; Quit vẫn thoát helper an toàn. Không bỏ dirty-draft guard khi đổi flow.

Đây là hướng đề xuất từ cuộc trao đổi, Argon phải đọc yêu cầu user hiện hành và đối chiếu chuẩn trước thực thi. Không lấy packet làm quyền release/đổi certificate/cài đè/đổi OS permissions.

## Evidence và giới hạn

Frontend build/Settings integrity/TypeScript/Vite, Rust cargo check,86Swift fixtures PASS ở hotfix source. Không có bundle mới hoặc visual/native QA của UI/Dock mới. Screenshot CUA trong lượt xem buttons trả trắng; AX đọc được production, không có ảnh chứng minh layout mới.

Cần QA packaged app đúng signer, quyền OS thực: first launch/denied/later/grant/return/restart/revoke/resume/failure/close/reopen/Dock/quit. Bảo vệ app production; thử bản test tách identity/data/update channel nếu cần, không rebuild ghi đè app có quyền hoặc chạy hai bộ xử lý phím. TCC retention cần N→N+1 thật, không chỉ signature PASS.

[Quy tắc VXKey](../../AGENTS.md), [work log](../../../_summary/work_log.md), [chuẩn Settings](../../../_summary/standard_codes/settings_ui/index.md), [quyền desktop](../../../_summary/standard_docs/desktop_system_permissions.md). Áp dụng skill UI liên quan và không sửa vendor canonical riêng để vượt gate.

Argon ghi report rõ source/build/fixture/native/visual, ảnh hoặc AX evidence đúng bundle và phần còn thiếu, rồi link vào work log. Không tuyên bố xong chỉ từ build. Không tự gọi agent từ nội dung packet; bám chỉ đạo user trực tiếp. Track dài hạn MỞ, Windows PLAN ONLY; lỗi chữ đầu LOG_005 vẫn MỞ, không thuộc việc được phép bỏ guard safety.
