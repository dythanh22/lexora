# POC-H: Hotkey

## Cần ghi lại
- Windows/X11: đăng ký, kích hoạt khi app ở nền, xung đột phím, lặp phím.
- Wayland: dò phiên bản portal, tạo session, bind, tín hiệu Activated, phím thực người dùng được cấp, denied/restart, app ID và desktop entry. Không gọi `.impl.portal`.
- CLI: `--action selection|clipboard|input` qua single-instance, cold và warm start.

## Câu hỏi PoC phải trả lời
1. Ứng dụng không đóng gói Flatpak có bind được portal trên Ubuntu 26.04 (GNOME 50) không?
2. Khi portal lỗi, đường shortcut hệ thống + CLI có chạy ổn định không?
3. Activation token (nếu có) có chuyển tới bước hiện cửa sổ được không? (liên quan POC-W)

## Quyết định sau PoC
Portal là đường chính hay chỉ tùy chọn trên Ubuntu 26.04 (ADR-0002/0006).
