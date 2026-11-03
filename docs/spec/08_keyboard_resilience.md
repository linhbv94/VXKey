# 08. Keyboard resilience — VXKey

Version: 0.8.0 — Ngày cập nhật: 2026-11-02
Ground truth: [background driver](09_background_driver.md). InputMethodKit/OS source fallback không còn dùng.

## Rủi ro và invariant KEY10

Bộ gõ có thể nuốt phím khi hook treo, giữ modifier/key-up, replay sai target, recursive injected events hoặc engine panic. Không tuyên bố không thể liệt bàn phím. Giảm phạm vi bằng helper riêng, event callback hữu hạn, lease/transport/main-loop health riêng và raw pass-through.

- Callback không disk/socket/AX RPC; context/config cache và real Rust engine trong process native.
- Bypass/secure input/unknown context thắng mọi transform/macro/retone/period. Không tự bật feature chưa có edit proof.
- Injected event nonce + producer PID tránh loop; không synth modifier; swallow key-up chỉ cho key-down đã consumed.
- Tap bị OS disable do timeout/user input: latch fault, không tự enable. Lỗi/panic không dùng lại engine state hỏng.
- Native loop chết: bridge progress watchdog làm status stale, supervisor chỉ dừng owned child. Helper mất parent tự thoát. Không broad killall, không đổi layout.
- Pause bằng native menu không phụ thuộc WebView. Disconnect không giả Pause đã ACK; marker chỉ chứng minh durable intent. Resume đòi session/readiness ACK và exact incident clear, không auto-restart.
- Autostart OFF cả Mac/Windows cho đến khi user quyết định đổi sau native safety QA. Không tự đăng ký login item.
- Không sửa TCC, keyboard source, MKey hoặc app khác để xử lý fault.

## Fault matrix bắt buộc trước daily-driver

| Case | Kỳ vọng | Bằng chứng cần |
|---|---|---|
| Thiếu/revoke AX hoặc post permission | Không nuốt phím, status thật | Native app/key readback |
| Tap timeout/main loop hang | OS tap disabled, latch, không reattach | Fault injection + raw keys |
| Companion/bridge chết | Lease hết, detach/disable; owned-child cleanup | PID/tap/status và typed text |
| Engine panic/invalid result | Quarantine, không replay text | ABI fixture + native fault |
| WebView chết | Native menu Pause vẫn dùng được | Native mouse path |
| Focus/selection/PID/app đổi | Không sửa client mới | Sentinel fixture thực nhiều app |
| Secure field/Secure Input | Raw pass-through, không AX text materialization | Password fixture không log text |
| Fast typing/lag/partial injection | Không xóa quá ownership; uncertain suspend | Native readback/latency |
| Modifier sides/repeat/key-up | Không stuck hoặc toggle lặp | Physical event fixture/native |
| Sleep/wake/logout/restart | Safe-start suspended sau fault | Native lifecycle |
| Update/re-sign | Quyền giữ hoặc UX cấp lại đúng identity | N→N+1 signed runtime; build không đủ |

Fixture transport/edit policy chỉ đóng đúng phần được chạy; không thay native tap/typing/performance acceptance. [QA05](05_qa_acceptance.md), [report hiện hành](../qa/background_driver_review.md).

## Windows và clipboard

Windows native worker phải có ownership/watchdog/fail-open tương đương, chưa nghiệm thu từ Mac build. History Win+V giữ provider native mặc định, không tranh global remapper. Clipboard worker tách key hot path; lỗi clipboard không làm mất phím. Default double-space Windows ON/Mac OFF và native Mac interlock giữ theo config/spec hiện hành, không bật capability khi chưa có driver edit proof.
