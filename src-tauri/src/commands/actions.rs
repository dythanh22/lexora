//! Điều phối ba action (ADR-0003). Thứ tự quan trọng:
//!   Selection: CAPTURE TRƯỚC, rồi mới mở/focus cửa sổ (BR07, §7.4).
//!   Clipboard: mở cửa sổ có focus TRƯỚC, rồi mới đọc (Wayland chỉ cho client có focus đọc).
//!   Input: mở cửa sổ nhập.
use std::time::Duration;

use lexora_core::{
    AcquisitionMethod, Action, AppError, TextOrigin, TriggerContext,
};
use tauri::{AppHandle, Emitter, Manager};

use super::{SessionSnapshot, SnapshotKind};
use crate::state::{lock, AppState};
use crate::ui_host::popup::{self, PopupMode};

/// NFR02: tổng ngân sách capture 700 ms/action; hết thì chuyển nhập tay, không spinner vô hạn.
const CAPTURE_BUDGET: Duration = Duration::from_millis(700);

pub fn dispatch_action(app: &AppHandle, action: Action) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = run_action(&app, action).await {
            // Chỉ log khóa lỗi, không log nội dung.
            eprintln!("action {:?} failed: {}", action, e.message_key);
        }
    });
}

async fn run_action(app: &AppHandle, action: Action) -> Result<(), AppError> {
    let state = app.state::<AppState>();
    let request_id = lock(&state.coordinator)?.begin(action);

    let snapshot = match action {
        Action::OpenInput => {
            popup::show(app, PopupMode::Focused)?;
            SessionSnapshot {
                request_id,
                kind: SnapshotKind::Editing,
                text: None,
                origin: Some(TextOrigin::Manual),
                method: Some(AcquisitionMethod::UserInput),
            }
        }
        Action::TranslateSelection => {
            let reader = crate::platform::selection_reader();
            let ctx = TriggerContext::now(action, reader.backend_id());
            let got = tokio::time::timeout(CAPTURE_BUDGET, reader.read_selection(&ctx, CAPTURE_BUDGET)).await;
            match got {
                Ok(Ok(Some(t))) => SessionSnapshot {
                    request_id,
                    kind: SnapshotKind::Ready,
                    text: Some(t.text),
                    origin: Some(TextOrigin::Selection),
                    method: Some(t.method),
                },
                // Không có/không hỗ trợ/lỗi/quá hạn -> hướng dẫn, KHÔNG dùng clipboard cũ (BR03).
                _ => {
                    lock(&state.coordinator)?.capture_failed(request_id);
                    SessionSnapshot {
                        request_id,
                        kind: SnapshotKind::NeedsInput,
                        text: None,
                        origin: None,
                        method: None,
                    }
                }
            }
        }
        Action::TranslateClipboard => {
            popup::show(app, PopupMode::Focused)?;
            let text = crate::platform::clipboard_reader().read_text().await.ok().flatten();
            match text {
                Some(t) => SessionSnapshot {
                    request_id,
                    kind: SnapshotKind::Ready,
                    text: Some(t),
                    origin: Some(TextOrigin::Clipboard),
                    method: Some(AcquisitionMethod::ClipboardRead),
                },
                None => {
                    lock(&state.coordinator)?.capture_failed(request_id);
                    SessionSnapshot {
                        request_id,
                        kind: SnapshotKind::NeedsInput,
                        text: None,
                        origin: None,
                        method: None,
                    }
                }
            }
        }
    };

    *lock(&state.snapshot)? = Some(snapshot.clone());
    popup::show(app, PopupMode::View)?;
    // Cửa sổ có thể chưa nạp xong: UI cũng gọi `get_session_snapshot` khi khởi động.
    let _ = app.emit_to("popup", "lexora:session", snapshot);
    Ok(())
}
