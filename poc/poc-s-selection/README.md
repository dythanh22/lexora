# POC-S: Selection

## Cần ghi lại (theo từng ứng dụng + backend)
- Windows: UI Automation TextPattern. Linux: AT-SPI Text; PRIMARY chỉ khi chứng minh còn là lựa chọn hiện tại.
- Khoảng trống, Unicode, nhiều dòng, đổi focus giữa chừng, timeout, trường mật khẩu.
- AT-SPI: thử có/không bật cờ accessibility của phiên (`org.a11y.Status`) và ghi tác động tới ứng dụng đích.
- Ghi rõ gói cài (deb/Snap/Flatpak), native hay XWayland.

## Lưu ý
Không đạt ngưỡng thì ghi *supported-with-copy*. Không đổi mẫu thử để đạt (R01).
