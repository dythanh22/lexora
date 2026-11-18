// Danh sách command là nguồn sự thật cho quyền theo cửa sổ (capabilities/*.json).
// Thêm command mới: thêm vào đây, vào `generate_handler!` (lib.rs) và vào capability cần dùng.
// Tên permission tự sinh có dạng `allow-<command-dùng-gạch-ngang>`; kiểm lại ở M1 (thiết kế P17).
fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "trigger_action",
            "get_session_snapshot",
            "submit_translation",
            "cancel_translation",
            "get_capabilities",
            "get_config",
            "update_config",
            "save_secret",
            "delete_secret",
            "close_window",
            "quit_app",
        ]),
    ))
    .expect("failed to run tauri build");
}
