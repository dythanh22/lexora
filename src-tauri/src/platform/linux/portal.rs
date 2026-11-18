#![allow(dead_code)]
//! Portal `org.freedesktop.portal.GlobalShortcuts` (public interface, KHÔNG gọi `.impl.portal`).
//!
//! TODO(POC-H):
//!  - Dò version, tạo session, bind, đọc danh sách phím thực tế người dùng cấp.
//!  - Nhận Activated/Deactivated/ShortcutsChanged/Session Closed; không đăng ký lại mỗi lần dịch.
//!  - Cần app ID ổn định + desktop entry đúng; chuyển `activation_token` (nếu có) tới bước
//!    activate cửa sổ. Không coi `registered == usable`.
//!  - Dự phòng bắt buộc: CLI `lexora --action selection|clipboard|input` + single-instance,
//!    người dùng tự gán phím trong cài đặt hệ thống.
