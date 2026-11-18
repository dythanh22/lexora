//! Quản lý cửa sổ. (Đặt tên `ui_host` thay vì `windows/` của thiết kế để không trùng tên
//! với crate `windows` sẽ được thêm ở M1.)
pub mod popup;

use lexora_core::{AppError, AppErrorCode};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub(crate) fn ui_err(_: tauri::Error) -> AppError {
    AppError::new(AppErrorCode::Internal).with_diagnostic("window operation failed")
}

/// Cửa sổ cài đặt tạo khi cần, không giữ sống nền (§8.2).
pub fn show_settings(app: &AppHandle) -> Result<(), AppError> {
    if let Some(w) = app.get_webview_window("settings") {
        w.show().map_err(ui_err)?;
        w.set_focus().map_err(ui_err)?;
        return Ok(());
    }
    WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
        .title("Lexora")
        .inner_size(720.0, 520.0)
        .min_inner_size(560.0, 400.0)
        .build()
        .map_err(ui_err)?;
    Ok(())
}
