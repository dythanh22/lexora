# POC-W: Popup, focus, activation

## Cần ghi lại
- Windows/X11: mở không lấy focus; `WS_EX_NOACTIVATE`; vị trí cạnh con trỏ.
- Wayland (GNOME 50, KDE, sway): cửa sổ có hiện đúng, có bị đưa ra sau không, cần activation token không, vị trí do compositor chọn.
- Scale 100/150/200%, đa màn hình, tháo màn hình, fullscreen, ghim/đóng.

## Quyết định sau PoC
Chốt hành vi chuẩn cho Wayland (ADR-0006) và nhãn tính năng trong UI.
