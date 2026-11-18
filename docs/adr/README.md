# Architecture Decision Records

Mỗi quyết định có điều kiện xem lại. Chỉ chuyển trạng thái sang **Đã chốt** khi có bằng chứng PoC (xem `docs/support-matrix.md` và `tests/evidence/`).

| ADR | Quyết định | Trạng thái |
|---|---|---|
| [0001](0001.md) | Tauri 2 + Svelte + TypeScript, lõi Rust độc lập | Đề xuất |
| [0002](0002.md) | Hai nền tảng chính: Windows 11 và Ubuntu 26.04 GNOME Wayland | Đề xuất |
| [0003](0003.md) | Ba điểm vào: selection, clipboard, nhập tay | Đề xuất |
| [0004](0004.md) | Một engine tự động duy nhất ở MVP (ứng viên: Azure Translator) | Chờ POC-T |
| [0005](0005.md) | Không giả lập input trên Wayland ở MVP | Đề xuất |
| [0006](0006.md) | Popup trên GNOME Wayland: cửa sổ có focus, không hứa vị trí hay always-on-top | Đề xuất |
| [0007](0007.md) | Không lưu lịch sử dịch | Đề xuất |
| [0008](0008.md) | Kho khóa hệ điều hành, không fallback plaintext | Chờ POC-K |
| [0009](0009.md) | Capabilities Tauri giới hạn command theo cửa sổ; không có command đọc lại key | Đề xuất |
| [0010](0010.md) | Clean-room, giấy phép chưa chốt | Cần quyết định |
