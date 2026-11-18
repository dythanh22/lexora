# PoC (thiết kế §11)

PoC là chương trình tách biệt, KHÔNG thuộc workspace ứng dụng (`exclude = ["poc"]`). Mục đích là
**tạo bằng chứng để quyết định**, không phải làm nguyên mẫu để chép vào app.

## Quy trình mỗi PoC

1. Chạy `cargo run -p` / script của PoC trên máy thật, ghi môi trường bằng `poc-env-probe`.
2. Ghi kết quả vào `tests/evidence/<poc>/<ngày>-<máy>.md` theo `tests/evidence/TEMPLATE.md`.
3. Cập nhật `docs/support-matrix.md` chỉ từ bằng chứng đã ghi.
4. Chuyển trạng thái ADR liên quan (`docs/adr/`) khi đạt tiêu chí, hoặc ghi quyết định đổi hướng.

Không đổi mẫu thử hay nới ngưỡng để "đạt": nếu không đạt thì ghi là *supported-with-copy* hoặc *không hỗ trợ*.

## Danh sách (tiêu chí gốc trích từ thiết kế §11)

| PoC | Mục tiêu | Tiêu chí đạt |
|---|---|---|
| [poc-h-hotkey](poc-h-hotkey/README.md) | Hotkey Win/X11; portal GlobalShortcuts Wayland; single-instance/CLI | NFR03; portal lỗi thì shortcut hệ thống + CLI phải chạy; không báo registered = usable |
| [poc-s-selection](poc-s-selection/README.md) | UIA / AT-SPI / PRIMARY trên ứng dụng thật | NFR04; không đạt thì ghi *supported-with-copy*, không âm thầm đổi mẫu |
| [poc-w-popup](poc-w-popup/README.md) | Popup, focus, activation | Không mất nguồn trước capture; có vị trí hữu ích; activation lỗi thì có bước mở thủ công chấp nhận được |
| [poc-c-clipboard](poc-c-clipboard/README.md) | Copy giả lập + khôi phục | 0 lần ghi đè dữ liệu mới; format không bảo toàn thì không tự Copy; không đạt thì tắt backend |
| [poc-k-secrets](poc-k-secrets/README.md) | Kho khóa | Không plaintext fallback; RAM-only dùng được; production không dùng mock |
| [poc-t-translation](poc-t-translation/README.md) | Dịch máy + hợp đồng stream bằng fake | Chất lượng đạt corpus; lỗi chuẩn hóa; fake không chứng minh tương thích dịch vụ thật |
| [poc-r-webview](poc-r-webview/README.md) | Vòng đời WebView và bộ nhớ | Chọn chiến lược cửa sổ bằng số đo thật |

Thứ tự gợi ý trong M1: **POC-H và POC-S trên GNOME Wayland trước** (khả năng no-go cao nhất, làm đổi phạm vi), rồi POC-W, POC-C, POC-K, POC-R.
