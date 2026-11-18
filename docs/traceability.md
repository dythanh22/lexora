# Truy vết yêu cầu

Cột "Trạng thái khung" mô tả đúng hiện trạng repo: *khung* = có cấu trúc/stub, *test viết sẵn* = có unit test nhưng chưa chạy được trong môi trường dựng khung, *—* = chưa bắt đầu.

| FR | Yêu cầu | Milestone | Test | Vị trí mã | Trạng thái khung |
|---|---|---|---|---|---|
| FR01 | Dịch vùng chọn | M3 | T01, T02 | `src-tauri/src/platform/*`, `commands/actions.rs` | khung (reader trả `Ok(None)`) |
| FR02 | Dịch clipboard | M2–M3 | T03 | `commands/actions.rs`, `platform/*::clipboard_reader` | khung |
| FR03 | Nhập/chỉnh sửa văn bản | M2 | T08 | `src/lib/components/SourceInput.svelte` | UI khung |
| FR04 | Hotkey + dự phòng CLI | M1, M3 | T04 | `platform/linux/portal.rs`, `lib.rs` (single-instance + CLI) | CLI parse có test; hotkey chưa |
| FR05 | Cửa sổ kết quả | M1–M3 | T05 | `ui_host/popup.rs`, `src/popup/` | khung |
| FR06 | Dịch máy | M2 | T06, T11 | `crates/providers/src/azure.rs` | hàm thuần + test viết sẵn |
| FR07 | Điều phối phiên | M2 | T07 | `crates/core/src/coordinator.rs`, `src/lib/session-reducer.ts` | test viết sẵn |
| FR08 | Sao chép kết quả | M2 | T03, T07 | `Popup.svelte` | UI khung |
| FR09 | Ngôn ngữ nguồn/đích | M2 | T06, T11 | `crates/core/src/language.rs` | test viết sẵn |
| FR10 | Cấu hình và khóa | M2 | T09 | `src-tauri/src/config`, `secrets` | config có test viết sẵn; OS store là stub |
| FR11 | Single-instance / CLI | M1, M3 | T04, T10 | `lib.rs`, `core/model.rs::Action::from_cli_args` | test viết sẵn |
| FR12 | Khả năng hệ thống | M1, M3 | T01, T04 | `core/capability.rs`, `platform/*::probe` | khung (mọi thứ `Untested`) |
| FR13 | Quyền riêng tư | M2 | T09, T12 | `config.pause_network`, `build_provider` | khung |
| FR14 | Giới hạn đầu vào | M2 | T06 | `language::validate_source_text` | test viết sẵn |
| FR15 | Vòng đời desktop | M3, M4 | T10 | `lib.rs` (run event), `quit_app` | khung |
| FR16 | Phát hành | M4 | T09, T10, T12 | `.github/workflows/ci.yml`, `tauri.conf.json` | khung |

## Test T01–T12 (định nghĩa theo thiết kế §12.1) và nơi đặt

| Test | Loại | Nơi đặt |
|---|---|---|
| T01 | Platform integration: đúng app/selection, không selection cũ, timeout | `poc/poc-s-selection`, `tests/evidence/` |
| T02 | Desktop race: đổi focus trước injection, bấm liên tục, mở settings giữa capture | `poc/poc-c-clipboard`, `poc/poc-w-popup` |
| T03 | Clipboard integration: Copy cạnh tranh, format không hỗ trợ, Copy result | `poc/poc-c-clipboard` |
| T04 | Desktop lifecycle: xung đột/lặp phím, portal denied/restart, cold/warm | `poc/poc-h-hotkey` |
| T05 | UI/native: focus, scale, đa màn hình, fullscreen, pin/close | `poc/poc-w-popup` |
| T06 | HTTP contract: độ dài, ngôn ngữ, lỗi JSON, auth, quota, 429, timeout | `crates/providers` (mock; thêm `wiremock` ở M2) |
| T07 | Core unit + integration: request mới vs kết quả cũ, retry stale, cancel, trigger lặp | `crates/core/src/coordinator.rs`, `src/lib/session-reducer.test.ts` |
| T08 | IME thật + UI: IBus/Fcitx5, IME Windows, Enter khi composition, chỉ dùng bàn phím | Kịch bản thủ công trên máy thật (`tests/evidence/`) |
| T09 | Security/config: secret không có trong file/log/args, store thiếu, migration, xóa | `src-tauri/src/config`, `secrets`, `poc/poc-k-secrets` |
| T10 | Install/lifecycle: .deb/installer, app ID, đường dẫn có dấu/khoảng trắng, quit, nâng cấp | M4 |
| T11 | Product UAT: corpus chất lượng, hiểu nhãn nguồn/provider | `poc/poc-t-translation`, `tests/corpus/` |
| T12 | Privacy/performance: mạng khi idle, redaction, CPU/RAM soak, không gọi provider ngầm | M2–M4 |

## Khoảng trống đã biết giữa khung và thiết kế (cần xử lý ở M2/M3)

- FR13: thiết kế yêu cầu người dùng chọn provider TRƯỚC lần gửi đầu tiên. Khung hiện mặc định Azure và chỉ báo `config_missing` khi thiếu key. Cần màn hình xác nhận lần đầu.
- FR12: cần nút "Thử lại" probe năng lực trong tab Khả năng hệ thống (khung hiện probe lúc mở tab).
- FR04: chưa có thao tác "thử phím" và hiển thị phím thực mà portal trả về.
- FR15: chưa có tray; cửa sổ Settings và `lexora --action input` là đường mở hiện có.
