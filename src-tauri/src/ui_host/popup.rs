//! Cửa sổ kết quả/nhập. Một cửa sổ tái sử dụng (§8.2).
//!
//! Hành vi theo nền tảng (ADR-0006):
//!  - Windows/X11: mở KHÔNG lấy focus khi xem kết quả (nếu POC-W đạt).
//!  - GNOME Wayland: cửa sổ có focus; vị trí do compositor quyết định. Không hứa vị trí
//!    cạnh chuột hay always-on-top (Tao không hỗ trợ trên Wayland).
//! TODO(POC-W): WS_EX_NOACTIVATE (Windows), activation token (Wayland), nhớ vị trí/kích thước
//!   hợp lệ theo màn hình và scale. TODO(POC-R): prewarm hay tạo khi cần — quyết định bằng số đo.
use lexora_core::AppError;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use super::ui_err;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupMode {
    /// Xem kết quả, cố gắng không lấy focus.
    View,
    /// Cần nhập liệu: luôn có focus.
    Focused,
}

pub fn show(app: &AppHandle, mode: PopupMode) -> Result<(), AppError> {
    let focus = matches!(mode, PopupMode::Focused) || crate::platform::popup_takes_focus();
    if let Some(w) = app.get_webview_window("popup") {
        w.show().map_err(ui_err)?;
        if focus {
            w.set_focus().map_err(ui_err)?;
        }
        return Ok(());
    }
    WebviewWindowBuilder::new(app, "popup", WebviewUrl::App("index.html".into()))
        .title("Lexora")
        .inner_size(480.0, 360.0)
        .min_inner_size(360.0, 240.0)
        .resizable(true)
        .skip_taskbar(true)
        .focused(focus)
        .build()
        .map_err(ui_err)?;
    Ok(())
}
