# POC-C: Copy giả lập và khôi phục clipboard

## Phạm vi
Windows (và X11 nếu cần). **Không** làm trên Wayland ở MVP (ADR-0005).

## Cần ghi lại
Text, HTML, ảnh, files, định dạng tùy biến/trễ, người dùng Copy cạnh tranh, terminal (không gửi Ctrl+C), hủy, ghi muộn.

## Tiêu chí
0 lần ghi đè dữ liệu mới của người dùng; format không bảo toàn thì không tự Copy.
