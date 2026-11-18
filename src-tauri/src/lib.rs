//! Lexora desktop shell. Khung M0: chưa có PoC nào được chạy (docs/support-matrix.md).
mod commands;
mod config;
mod platform;
mod secrets;
mod state;
mod ui_host;

use lexora_core::Action;
use tauri::Manager;

use crate::state::AppState;

pub fn run() {
    tauri::Builder::default()
        // single-instance phải đăng ký đầu tiên. Chỉ nhận ACTION ID từ argv (FR11);
        // không bao giờ nhận văn bản hoặc key qua tham số dòng lệnh.
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            match Action::from_cli_args(&argv) {
                Ok(Some(action)) => commands::dispatch_action(app, action),
                Ok(None) => {
                    let _ = ui_host::show_settings(app);
                }
                Err(e) => eprintln!("rejected cli args: {}", e.message_key),
            }
        }))
        // Plugin này chỉ hoạt động trên Windows/X11. Wayland: portal + CLI (POC-H).
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let dir = app.path().app_config_dir()?;
            app.manage(AppState::new(dir.join("config.json")));

            // Cold start qua shortcut hệ thống: `lexora --action selection`.
            let argv: Vec<String> = std::env::args().collect();
            match Action::from_cli_args(&argv) {
                Ok(Some(action)) => commands::dispatch_action(app.handle(), action),
                _ => {
                    let _ = ui_host::show_settings(app.handle());
                }
            }
            // TODO(POC-H): đăng ký hotkey (Win/X11) hoặc bind portal (Wayland).
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::trigger_action,
            commands::get_session_snapshot,
            commands::submit_translation,
            commands::cancel_translation,
            commands::get_capabilities,
            commands::get_config,
            commands::update_config,
            commands::save_secret,
            commands::delete_secret,
            commands::close_window,
            commands::quit_app,
        ])
        .build(tauri::generate_context!())
        .expect("không khởi tạo được Lexora")
        .run(|app, event| {
            // Chạy nền chỉ khi người dùng bật; `Quit` (exit code Some) luôn thoát thật.
            if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
                if code.is_none() {
                    let keep = app
                        .try_state::<AppState>()
                        .and_then(|s| s.config.lock().ok().map(|c| c.run_in_background))
                        .unwrap_or(false);
                    if keep {
                        api.prevent_exit();
                    }
                }
            }
        });
}
