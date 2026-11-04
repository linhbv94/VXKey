# Session tap spike — P0/P1

Ngày: 2026-11-03. Owner: Codex. Track VXKey **MỞ, chỉ user được đóng**.

**Hiện hành: v6 return-original đã qua P1 native và P2 boundary/sentinel cơ bản theo user.** Spike đã Pause/Quit; MKey mở lại. Chưa full parity/P2 đầy đủ/P3/P4, chưa thay production. [Evidence](session_spike_evidence.json), [artifact v6](session_spike_v6_artifacts.json), [trace v6](session_spike_v6_return_original_trace.json). Các revision phía dưới là quá trình điều tra, không phải app đang chạy.

**Quyết định docs2026-11-04:** chọn route v6 để triển khai tiếp theo MAC_ADAPTER_002 trong [spec11](../spec/11_mac_driver_replan.md). Backlog MAC_ADAPT_01…05 có đầu ra/gate rõ; chưa đổi source hoặc runtime trong lượt cập nhật quyết định này.

User giao triển khai hướng đã ghi trong [replan11](../spec/11_mac_driver_replan.md). Lượt này tự triển khai, không gọi Anti/Argon hoặc đặt lại bộ đếm CLI. Không triển khai Windows, promote driver, publish hoặc thay app trong Applications.

## Phạm vi source đã thực hiện

- [Session composition](../../native/macos_session_spike/session_composition.swift): Funput C ABI trong cùng process, Telex/modern, giữ suffix từ event history; chuẩn bị edit hữu hạn trước khi chặn phím. Đây không phải document ownership đã xác minh.
- [Session driver](../../native/macos_session_spike/session_driver.swift): active session tap, batch Backspace/Unicode qua callback proxy, own-event nonce, chặn keyUp tương ứng; thử `nil` suppression riêng trên session tap. Không bê kết luận `.null` của per-PID tap cũ.
- Không dùng ContextGuard, AX caret/text/range RPC, bridge/React/clipboard/socket hoặc lease trong đường gõ. TCC preflight chạy worker, callback đọc cache; reply chậm không tự tạo lease hết hạn. Fresh negative grant/tap-disable vẫn dừng/bypass; lỗi không tự Resume.
- Reset khi đổi app/layout, click/drag/scroll, Command/Control/Option, navigation, Enter/Shift+Enter, Backspace, autorepeat hoặc Unicode ngoài scope. Ký tự tiếp theo khởi động composer ngay.
- [App native thử](../../native/macos_session_spike/main.swift): một process tự sở hữu hook, single-instance lock, khởi động **paused**, native menu Pause/Quit/hướng dẫn. Quit làm mất hook; không có orphan helper, updater hoặc login item. Đây là harness P1, chưa tích hợp lifecycle production.
- P1 chỉ bật cho **TextEdit/Figma nháp** nhằm giới hạn QA, không phải app whitelist/role rule sẽ đưa vào sản phẩm. Không load policy, macro, history hoặc config của production. Hard bypass/per-app memory/hotkey/retone/VNI/double-space phải giữ trong sản phẩm khi tới P4; chưa tuyên bố parity cho harness này.
- Không copy mã MKey GPL. Chỉ tham khảo kiến trúc đã ghi trong replan; dùng wrapper/core và edit planner sẵn có của VXKey.

## Build và baseline

- [Build script](../../scripts/build_session_spike.sh), [fixture runner](../../scripts/test_session_spike.sh), [manifest reader](../../scripts/session_spike_manifest.py).
- Candidate hiện hành **v6**: `build/session_spike_candidate_v6/vxkey_session_spike.app`, version0.1.5/build6, ID `com.vxkey.spike.session`, executable `vxkey_session_spike`; app hiển thị VXKey Session Spike. Menu `VX S‖` = paused, `VX S` = đang thử. OS Accessibility hiển thị tên executable `vxkey_session_spike`. V1–v5 giữ bytes làm evidence, đã Quit; v6 cũng đã Quit sau QA. Không chạy lại mặc định.
- Dev ad-hoc identity riêng có chủ đích, không fallback từ signer production. Không cam kết retention khi rebuild; script từ chối ghi đè bundle đã tồn tại. **Bundle hiện đã được user cấp quyền: giữ nguyên bytes/path.**
- Data thử hiện hành chỉ `build/session_spike_candidate_v6/ss_data`: lock và diagnostics theo lệnh export/Quit. Diagnostics thường chứa counters/capability/state/timing, không raw key/text/app document/clipboard. Mode engine-fixture chỉ nhận đúng chuỗi kê sẵn `as af aa`; trace core/index/keycode chỉ thuộc chuỗi đó, dừng khi mismatch/boundary. Các folder revision cũ giữ snapshot riêng.
- [Baseline trước build](session_spike_baseline.json) và [artifact sau build](session_spike_artifacts.json) có HEAD, dirty-source fingerprint, OS/architecture/SDK, executable hashes và codesign output. Fingerprint được chụp sau khi viết source spike, trước build; không giả là snapshot trước mọi sửa nguồn trong lượt này.
- App production và candidate compatibility cũ có executable hash **không đổi**. Production strict check ở môi trường hiện tại trả `CSSMERR_TP_NOT_TRUSTED` trước và sau build; không xử lý trust/keychain trong task này hoặc ghi là production verify PASS. Test candidate cũ và spike strict verify exit0.
- Process baseline chỉ thấy MKey, không VXKey đang chạy. Sau launch spike không còn thấy process MKey; không suy ra đây là Codex đã chuyển mode MKey. Chưa tự mở lại MKey. App production/candidate cũ không được launch.

## Checks và native gate

1. [Fixture log](session_spike_fixture.log): real core + edit policy + event construction **476 assertions PASS**. Corpus `aa/ngaan/ngaan hangf/ddwojwc/casch/cachs/w`, reset, Unicode sentinel và thứ tự down/up. Không tạo tap, không post event; không phải native typing PASS. Có cảnh báo `hiservices-xpcservice Connection invalid` của event construction trong môi trường fixture; không che hoặc tính như native success.
2. [Build log](session_spike_build.log): compile optimized arm64/deployment26/SDK27, ad-hoc signing riêng, strict verify và artifact SDK gate PASS.
3. User duyệt chạy/cấp quyền riêng và tạm chuyển MKey sang English nếu còn chạy. Startup alert đã kiểm trực tiếp: paused, hướng dẫn nháp, Pause/Quit. macOS đã bật **riêng Trợ năng** cho `vxkey_session_spike` sau user xác thực. Không thêm/bật Input Monitoring cho spike.
4. Công cụ CUA truy cập app chỉ có menubar sau khi đóng alert bị timeout dù process còn chạy. Chưa sửa/rebuild app đã cấp quyền để chữa automation. Nhờ user thao tác menu native và gõ tay; không coi lỗi AX của công cụ là lỗi keyboard driver.
5. **v1 gate lỗi đã xác định:** user báo probe mờ và đọc `AX=true, post=false`. Harness đã dùng post preflight khóa cả probe không hề phát phím. V2 bỏ gate post; chỉ thử tạo tap khi AX true, post/listen chỉ được quan sát. Chưa chứng minh proxy-posting/Telex hoạt động từ sửa gate này.
6. V2 có cửa sổ điều khiển native đọc được bằng CUA, hết phụ thuộc vào automation menu windowless. Đã Quit đúng PIDv1 bằng Activity Monitor→Quit, kiểm không còn VXKey/MKey trước launch v2. Giữ cùng identifier/purpose/quyền thử đã duyệt; chỉ thay entry spike sang exact v2, không thay entry production. Process v2 đọc **AX=true/post=false/listen=false**, bấm Probe thành `running`, `tap_attached=true`; [runtime snapshot](session_spike_evidence.json). Không request Input Monitoring/PostEvent.
7. **Physical filter PASS**: user xác nhận gõ tay `azx` thành `aX` cả TextEdit và Figma canvas nháp. Snapshot sau thử: tap attached, seen29/suppressed12, không post/listen preflight. Những count này bao gồm các phím khác trong lượt thử, không phải trace riêng mỗi `azx`. Callback153samples p50=9.167µs/p95=17.416µs/p99=57.375µs; đây là filter/mixed bypass timing, không phải end-to-end typing latency hoặc composer benchmark. Sau gate này mới chuyển cùng binary sang modecompose và nhờ gõ corpus Telex.

V2 [build log](session_spike_v2_build.log), [fixture log](session_spike_v2_fixture.log), [artifact manifest](session_spike_v2_artifacts.json). Cùng476assertions PASS, SDK27/strict test signing PASS. Không rebuild/re-sign v2 đã cấp quyền.

Evidence hiện hành đọc ở [session_spike_evidence.json](session_spike_evidence.json). P2/P3/P4 còn mở.

## Playbook tiếp tục

1. Không chạy đồng thời MKey/VXKey khác đang xử lý tiếng Việt. Không rebuild/re-sign spike đã cấp quyền.
2. Menu `VX S‖` → **2. Probe azx → aX**. Click nháp TextEdit, gõ từng phím `azx` chữ thường. Kết quả phải `aX`; thử Figma canvas nháp tương tự. Nếu z lọt hoặc X lặp, dừng composer, menu Pause rồi ghi kết quả thật.
3. Khi probe đạt cả hai, menu → **3. Gõ Telex**. Thử `aa`, `ngaan`, `ngaan hangf`, `ddwojwc`, `mwojwt`, `casch cachs`, `w`, `casi`. Kết quả đọc trong app thật, không từ ô demo engine.
4. P2: nháp có prefix/suffix sentinel, gõ ở giữa; Cmd+A→Delete→gõ ngay, Shift+Enter→gõ ngay, click/drag/selection, focus, paste/undo/autocomplete, nhanh/chậm. Một lỗi sửa nhầm sentinel là blocker. Không lấy dữ liệu thật làm phép thử.
5. Export diagnostics bằng cửa sổ hoặc menu rồi đọc `build/session_spike_candidate_v6/ss_data/diagnostics.json`. Callback timing không phải end-to-end latency, và counters không thay readback text.
6. Pause/Quit khi chưa được yêu cầu dùng tiếp. Không tự bật lại production/MKey khác trong khi spike active. Tới P3 mới fault-inject bridge/helper production; spike không có bridge nên không thể chứng minh production lifecycle đã sửa.

## Giới hạn cần giữ rõ

Không đọc AX đồng nghĩa không quan sát được mọi caret/selection do app đổi bằng code. Session event target PID có thể0; focus cache + notification không phải atomic document transaction. Secure Input được bypass nhưng không bảo đảm mọi password field đều bật Secure Input. P1 nháp chưa phải bằng chứng đủ cho dữ liệu thật hoặc daily use. Event pairs được dựng đúng chưa chứng minh client xử lý thứ tự/chunks đúng. Không tự đóng LOG_005/007/008/009, không đổi KEY01/BG04/BG06 hoặc spec09/10 production từ fixture/build.

## Native Telex FAIL v2 và thí nghiệm v3 đang chạy

User xác nhận corpus ở **cả TextEdit lẫn Figma** cho `â ngân ngân hng được cch cch ư`, thay vì `â ngân ngân hàng được cách cách ư`. V2 đã Pause/Quit, chưa đủ gate P1. Counters trước Pause:22replacements/seen119/suppressed56; không engine/tap fault được ghi, không coi counts là text correctness. Oracle core của fixture cho `hangf`→delete3+`àng`, `casch`→delete1+`á` tại phím s, `cachs`→delete3+`ách`; toàn corpus này chỉ ACTION_NONE/SEND, không phải ACTION_RESTORE.

Thử CUA `as` trong tài liệu TextEdit mới `Untitled 7` trả raw`as`, nhưng seen không tăng (119→119). Công cụ gửi vào editor mà không đi qua session tap/foreground thực, **không dùng làm evidence native PASS/FAIL**. Không tiếp tục giả lập hàng loạt hoặc ghi nội dung tài liệu cũ.

Một khác biệt có cơ chế cụ thể: batch v2 dùng nil/default event source; Apple SDK `CGEventSource.h` mô tả private source giữ independent state table; MKey cũng dùng private source (chỉ đọc tham khảo, không copy code). **Chưa chứng minh đây là nguyên nhân mất nguyên âm.**

V3 `build/session_spike_candidate_v3/vxkey_session_spike.app` version0.1.2/build3, cùng ID/quyền thử, có checkbox source A/B chỉ đổi khi paused. Mặc định private. Core, backspace count, Unicode payload/chunk, batch order và nonce/PID guard giữ nguyên. Thay đổi được thử duy nhất ở transport là source tạo synthetic events; có thể chuyển về default trong cùng binary, không rebuild mỗi thử. V3 đã nhận riêng Trợ năng, AX=true/post=false/listen=false, modecompose/private đang chạy; đang chờ TextEdit `as af aa ngaan hangf casch cachs ddwojwc`. **Không chạy v1/v2 song song, không promote.**

- [Artifact v3](session_spike_v3_artifacts.json), [build SDK27/strict signing](session_spike_v3_build.log), [fixture479checks](session_spike_v3_fixture.log), [evidence hiện hành](session_spike_evidence.json).
- Fixture sandbox ban đầu không tạo được private source, hiservices XPC invalid và trap; rerun cùng fixture được quyền WindowServer PASS, không tap/posting. Không che lỗi môi trường hoặc tính lần trap là PASS.
- Data/diagnostics hiện hành `build/session_spike_candidate_v3/ss_data`; giữ frozen bundle v3 sau cấp quyền.
- P2 source gap đã thấy: harness chỉ áp ACTION_SEND, chưa xử lý ACTION_RESTORE như full adapter; thêm restore/English parity trước tích hợp, không quy lỗi corpus hiện tại cho gap này vì oracle không phát ACTION_RESTORE ở các mẫu đó.

## Probe transport v4 — PASS; composer vẫn chưa được nghiệm thu

V3 private-source Telex: user báo **vẫn mất á/à** trên TextEdit. Không có exact output mới, không tự điền lại chuỗi cũ thành raw trace. V3 đã Pause rồi Quit; dữ liệu/candidate cũ giữ nguyên.

V4version0.1.3/build4 thêm ba probe cố định, không dùng Funput, gộp thành một lượt gõ `x x ax`:

1. D1 đổi Unicode của event x gốc thành á, trả event đó.
2. D2 chặn x và post một Unicode á mới (down/up).
3. D3 chỉ khi vừa thấy a thường, chặn x và post một Backspace pair + á pair. Sau ba x, không tiếp tục thay chữ.

Observer ở annotated-session tail là tap **chỉ quan sát/pass-through**, không phải composer thứ hai. Chỉ thu event mang nonce của probe fixed; nội dung khác không ghi. Metadata chỉ type/keycode/flags/ownPID và lớp payload knowná/a/Backspace/empty/unknown, không AX/text/clipboard. Không request thêm quyền.

User xác nhận `á á á` trong TextEdit. [Trace](session_spike_v4_fixed_transport_trace.json): expected/observed7events giống nhau, observerattached, AX=true/post=false/listen=false. Syntheticevents giữ producerPID đúng process sau khi qua stream. AppKit diễn giải keyDown á đúng; keyUp có AppKitclassplain_a nhưng đây cũng có trong expected và không làm probe mất á. Không tự gọi đó là bug. [Build](session_spike_v4_build.log), [fixture480checks](session_spike_v4_fixture.log), [artifacts](session_spike_v4_artifacts.json).

Kết quả này thu hẹp đường điều tra: Backspace+Unicode tối thiểu hoạt động không engine; chưa chứng minh mọi batch hoặc Telex đúng. Bước tiếp là so actual input/core callback/inline C result/batch bằng **known fixture `as af aa`**, có GUI-side oracle cùng process. Chỉ cho trace fixtuređúngchuỗi; mismatch/boundary giữa chuỗi dừng ghi, không thu phím ngoài scope. Chưa quay lại thử câu dài hoặc thêm rule từng app.

## V5/v6 — core đúng, thay delivery mới qua native trial

V5version0.1.4/build5 có known fixture `as af aa`: user vẫn báo mất á/à. [Trace v5](session_spike_v5_known_engine_trace.json) nhận đủ8input, abort=null; callbackUTF8/inlineC/GUIoracle đều trả đúng á/à/â, backspaces1 ở từng modifier. Expected/observed12events giống nhau; post-pair keycode0 đi tới annotated tail với đúng payload. Đây bác bỏ hướng quy riêng case này cho output engine hoặc packet mất giữa các tap, **không chứng minh text client đã hiển thị đúng**.

V6version0.1.5/build6 thêm deliveryA/B, core/edit counts không đổi:

- `return_original` (mặc định): chuẩn bị toàn bộ synthetic Backspace trước, đặt Unicode output lên chính event vật lý gốc và trả nó; giữ keycode/type/flags và cho keyUp vật lý đi qua. Không post thêm Unicode down/up keycode0.
- `post_pair` giữ baseline cũ, có thể chọn khi paused trên cùng v6. ReverseA/B trên cùng binary chưa chạy; so sánh native hiện có là v5post-pair FAIL → v6return-original PASS. Không gọi đây là chứng minh cơ chế nội bộ macOS tuyệt đối.
- P1 returnedpacket cap20UTF16, reject trước post khi vượt cap/empty/invalid. Chưa có full long-expansion strategy. Giữ ACTION_RESTORE/parity gap cho P2; không promote bản cắt tính năng.

[Fixture v6](session_spike_v6_fixture.log):484assertions PASS, gồm giữ identity/flags phímS, batch chỉ Backspace và reject oversized trước post. [Build](session_spike_v6_build.log)/SDK27/strict ad-hoctest signing PASS. [Trace v6](session_spike_v6_return_original_trace.json): đủ8knownkeys, core đúng, expected/observed9events giống nhau; Unicode đi trên S/F/A gốc thay vì12events baseline. User xác nhận `á à â`. User báo sau Enter ra raw, nhưng fixture được thiết kế dừng sau8phím; không dùng báo cáo đó để đóng/mở lỗi Enter, đã chuyển modecompose liên tục để thử lại.

**Physical trial modecompose (user xác nhận):**

- TextEdit: `as af aa ngaan hangf casch cachs ddwojwc w` đúng.
- Enter rồi `ddwojwc casi`, Shift+Enter rồi `ddwojwc mwojwt`, Cmd+A→xóa→gõ ngay `ddwojwc casi`: đúng ngay từ đầu.
- Figma canvas nháp: `ngaan hangf casch cachs` đúng.
- TextEdit sentinel: `PRE 🧪  SUF`→gõ giữa `cachs`→`PRE 🧪 cách SUF`; mouse-select riêng từ và gõ`ddwojwc`→`PRE 🧪 được SUF`. User xác nhận prefix/emoji/suffix giữ nguyên.

Kết luận hẹp: **P1 và subsetP2 này đạt**, route return-original là hướng có bằng chứng để tiếp tục. Chưa đo end-to-end latency hoặc usage dài ngày, chưa test mọi layout/uppercase/autocomplete/terminal/password/undo case. Chưa tích hợp policy/config/macros/VNI/retone/fullrestore hoặc helper/bridge lifecycle thật. Không đóng LOG_005/007/008/009 production từ kết quả harness.

## Cleanup và bước tiếp bắt buộc

V6 đã Pause rồi Quit, tap_attached=false; `ps` sau cleanup chỉ thấy MKey tại app cũ. MKey được mở lại qua CUA, app windowless làm getApp timeout nhưng ps xác nhận launch; không đổi mode/setting/TCC hoặc claim mode được kiểm. Production VXKey không launch/thay. Giữ tất cả bundle/hashes đã grant; không overwrite/re-sign v6.

Bước tiếp: P2 remaining matrix và parity (ACTION_RESTORE, VNI/advanced/retone/macros, output>20UTF16, shortcuts/policy); P3 lifecycle/helper-owner/control disconnect và quyền; P4 tích hợp cache config/policy đã auth và thống nhất readiness UI. Không chỉ chép harness làm default rồi mất tính năng/exclusion. Đồng bộ spec00/02/04/05/07/08/09/10 ở P4; Windows vẫn PLAN ONLY, release1.0.1 vẫn dừng. Không tự mở lại spike cho dữ liệu thật hoặc chạy đồng thời MKey.
