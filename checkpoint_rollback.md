# VXKey — checkpoint trước khi Anti sửa và cách quay lại

Ngày: 2026-11-04. Track dài hạn vẫn MỞ. Đây là bảo vệ source/bundle, không phải phát hành hoặc nghiệm thu mọi ca gõ.

## Checkpoint đã tạo và kiểm

Thư mục: [build/checkpoints/pre_anti_2026_11_04](build/checkpoints/pre_anti_2026_11_04/).

- [Manifest](build/checkpoints/pre_anti_2026_11_04/manifest.json): HEAD/branch/trạng thái dirty, file/hash/mode, hash archive và thông tin bundle.
- `source.tar.gz`: working tree hiện tại, gồm file tracked đã sửa và file untracked Git nhìn thấy. **Không phải chỉ source ở HEAD**. Có4.004file đã khôi phục vào scratch và đối chiếu hash; không ghi đè source trong phép thử.
- `tracked_worktree.patch`, `staged.patch`: lưu chênh lệch và phần staged để tham chiếu; không apply cả patch mù quáng lên working tree đang có thay đổi khác.
- `frozen_candidate/VXKey.app`: bản sao bundle Test đã QA, gồm helper; hash khớp artifacts và codesign strict PASS. Không launch/re-sign bản sao khi tạo checkpoint.
- `test_metadata/`: chỉ config/pins/recovery marker nếu có của Test; không copy clipboard/history hoặc runtime socket.
- `reference_only/work_log.md`: snapshot để đối chiếu. **Không restore toàn bộ work log**, tránh đè tiến độ app khác.

Không gồm ignored build/cache, Git object database hoặc quyền TCC/keychain. Bundle riêng được copy như trên. Source checkpoint trước phần bổ sung hướng dẫn rollback này; giữ tài liệu rollback/handoff mới khi khôi phục code.

## Khi bản mới gõ lỗi

1. Dừng lượt sửa của Anti ở ranh giới an toàn; ghi cách tái hiện và danh sách file nó đã đổi. Không tiếp tục build đè hoặc thử trên tài liệu thật.
2. Quit riêng candidate mới bị lỗi; xác minh helper của nó đã dừng. Không kill hàng loạt theo tên executable có thể trùng production.
3. Mở lại bundle cũ ở `build/session_integrated_candidate/desktop_candidate/VXKey.app`; không cần rollback source chỉ để chạy binary cũ. Giữ một bộ đang xử lý tiếng Việt tại một thời điểm. Có thể trở về MKey sau khi Test đã dừng.
4. Bundle cũ có giới hạn đã biết: LOG_010 có thể bắt Resume sau mở lại, flip F5 thử nghiệm chưa đạt. “Quay lại” là trở về đúng checkpoint này, không phải loại hết các nợ đã ghi.

Nếu Anti đã vô tình đè bundle cũ: giữ nguyên bundle hỏng trong thư mục cách ly mới, copy `frozen_candidate/VXKey.app` vào **vị trí trống** ban đầu, kiểm hash và codesign trước mở. Không merge bản sao lên bundle hỏng vì có thể còn file dư.

## Khi source bị sửa hỏng

1. Sao lưu trạng thái lỗi/diff vào checkpoint mới để không mất bằng chứng hoặc thay đổi hữu ích.
2. So diff Anti với manifest/archive trước khi sửa. Chỉ chọn file/hunk thuộc lượt Anti; nếu có agent/user sửa cùng file sau checkpoint thì merge phần cần khôi phục.
3. Giải nén bản gốc vào scratch để đối chiếu, rồi khôi phục từng file/hunk đã chọn. File mới Anti tạo phải kiểm riêng; có thể chuyển sang thư mục cách ly, không xóa mù quáng.
4. Không dùng `git reset --hard`, `git clean`, `git restore .` hoặc checkout HEAD để rollback: baseline đang có rất nhiều sửa/untracked cần giữ.
5. Chạy lại checks đúng phần sửa và regression đã ghi trong [handoff](anti_next_handoff.md); native gate vẫn cần gõ thật. Cập nhật work log với file đã restore, checks và phần còn mở.

## Quyền hệ điều hành

Không snapshot/restore database TCC hoặc keychain. Nếu lúc thử binary mới entry Test đã thay đổi, quay về bundle cũ có thể cần bật/cấp lại riêng quyền cho đúng binary theo OS. Backup app/source không bảo đảm tự khôi phục grant. Production entry và các app khác không được thay trong thao tác rollback.

## Anti phải giữ

Checkpoint này chỉ đọc trong lượt sửa tiếp, không overwrite/cleanup. Build candidate mới bằng root/data/report riêng. Nếu phát hiện regression, ưu tiên quay về mốc đã QA và khoanh thay đổi gây lỗi; không sửa lan cả engine để che triệu chứng.
