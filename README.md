# Lexora

Ứng dụng dịch nhanh, nhẹ cho Windows 11 và Ubuntu (GNOME Wayland là mục tiêu chính): bôi đen → phím tắt → cửa sổ dịch. Rust + Tauri 2 + Svelte 5 + TypeScript.

Thiết kế baseline: [`docs/design/Lexora_Reengineering_Product_Design_v1_0.md`](docs/design/Lexora_Reengineering_Product_Design_v1_0.md).

## Trạng thái: khung M0

Đây là **khung dự án**, chưa phải ứng dụng chạy được. Khi dựng, môi trường không có Rust toolchain và không có mạng, nên:

- **Chưa có dòng Rust nào được biên dịch hay chạy test.** Chưa cài/chạy được `npm`.
- Đã kiểm tra: JSON/TOML hợp lệ, khóa i18n vi/en khớp nhau, logic reducer của UI (bằng Node).
- Chưa có PoC nào chạy: mọi năng lực nền tảng đều hiển thị là *Chưa kiểm chứng*.

| Thành phần | Mức hiện tại |
|---|---|
| `crates/core`: mô hình, lỗi, coordinator, ngôn ngữ, ports | Mã thật + unit test viết sẵn |
| `crates/providers`: endpoint, HTTP policy, Azure (hàm thuần), Fake | Mã thật + unit test viết sẵn; chưa gọi dịch vụ thật |
| `src-tauri`: config, secrets (RAM), commands, window host | Mã thật; kho khóa hệ điều hành là stub |
| `src-tauri/src/platform/*` | Khung: reader trả "không hỗ trợ", mọi năng lực `Untested` |
| `src/` (Svelte): popup, settings, reducer, i18n | UI tối thiểu + test reducer/i18n |
| `poc/` | Hướng dẫn từng PoC; `poc-env-probe` chạy được (chỉ dùng std) |

## Bắt đầu

1. Icon (khung dùng ảnh giữ chỗ): `npm install && npx tauri icon src-tauri/icons/source.png`
2. Định dạng một lần: `cargo fmt --all`
3. Test lõi (không cần thư viện hệ thống): `cargo test -p lexora-core -p lexora-providers`
4. Test UI: `npm test` và `npm run check`
5. Chạy thử: `npm run tauri dev` (Linux cần `libwebkit2gtk-4.1-dev` và các gói nêu trong `.github/workflows/ci.yml`)
6. Ghi môi trường cho bằng chứng PoC: `cargo run --manifest-path poc/poc-env-probe/Cargo.toml`

Thực hiện **bước kiểm tra biên dịch đầu tiên ở M1** và sửa lỗi (nếu có) trước khi làm gì khác.

## Cấu trúc

```
crates/core        lõi độc lập (không Tauri/OS): model, coordinator, ports, capability, secret
crates/providers   adapter dịch: azure, endpoint, http, fake
src-tauri          shell Tauri: commands, config, secrets, platform/, ui_host/, capabilities/
src                UI Svelte: popup (index.html), settings (settings.html), lib/
poc                PoC H/S/W/C/K/T/R + poc-env-probe (ngoài workspace)
docs               thiết kế, ADR, truy vết, NFR, ma trận hỗ trợ, provenance
tests              corpus EN→VI, mẫu bằng chứng
```

Điều đã quyết định ở mức khung (xem `docs/adr/`): lõi đồng bộ test được; ba action (`selection|clipboard|input`) là allowlist duy nhất cho CLI/IPC; không command nào đọc lại key; sự kiện dịch mang `request_id/attempt_id/sequence` và bị lọc ở cả Rust lẫn UI; không có lịch sử dịch.

## Việc phải xác minh ở M1 (không coi là đã đúng)

- Phiên bản phụ thuộc: các ràng buộc trong `Cargo.toml`/`package.json` chỉ là khoảng rộng để phân giải. Chạy, pin, commit lockfile, kiểm giấy phép (`deny.toml`).
- Tên feature TLS của `reqwest` (khác nhau giữa các bản 0.x).
- Tên permission Tauri tự sinh (`allow-<command>`) cho `capabilities/*.json`, và các API đã dùng (`Channel`, `emit_to`, `RunEvent::ExitRequested`).
- `app identifier` (`dev.lexora.desktop`) là giữ chỗ: phải trùng desktop entry và app ID portal; kiểm tra trùng tên/nhãn hiệu "Lexora" trước khi chốt.
- Giả định `AT-SPI`, portal `GlobalShortcuts`, popup Wayland: chỉ là hướng đi, cần PoC (`poc/`).
- `tauri-plugin-global-shortcut` chỉ phục vụ Windows/X11; Wayland đi qua portal + CLI.

## Lộ trình (thiết kế §10)

M1 loại rủi ro nền tảng (POC-H/S/W/C/K/R) → M2 lát cắt dịch an toàn → M3 MVP đầy đủ → M4 phát hành → mở rộng sau. Chi tiết: thiết kế §10 và `docs/traceability.md`.
