//! IPC (thiết kế §8.5). Quy tắc:
//!  - Rust validate lại mọi input; UI không được tin.
//!  - Không có command nào trả plaintext key về UI.
//!  - Sự kiện dịch đi qua `Channel` gắn với đúng lời gọi, không broadcast.
//!  - Không giữ khóa `Mutex` qua `.await`.
mod actions;

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use lexora_core::coordinator::{Coordinator, SubmitOutcome};
use lexora_core::ports::{EventSink, TranslationProvider};
use lexora_core::{
    AcquisitionMethod, Action, AppError, AppErrorCode, AttemptId, LangCode, ProviderEvent,
    ProviderEventKind, RequestId, SecretRef, SourceLang, TextOrigin, TranslationRequest,
    MAX_INPUT_CHARS,
};
use lexora_core::capability::Capability;
use lexora_core::ports::SecretStore;
use lexora_providers::azure::{self, AzureConfig, AzureProvider};
use lexora_providers::http::HttpPolicy;
use serde::Serialize;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};
use tokio_util::sync::CancellationToken;

use crate::config::{self, Config, CURRENT_SCHEMA};
use crate::secrets::ResolverAdapter;
use crate::state::{lock, AppState};

pub use actions::dispatch_action;

// ---------- Snapshot phiên cho UI ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotKind {
    /// Đã có nguồn hợp lệ từ capture: UI tự gửi dịch (đây chính là hành động của người dùng).
    Ready,
    /// Capture không có kết quả: hiện hướng dẫn Copy/dán; tuyệt đối không dùng clipboard cũ.
    NeedsInput,
    /// Ô nhập tay.
    Editing,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSnapshot {
    pub request_id: RequestId,
    pub kind: SnapshotKind,
    pub text: Option<String>,
    pub origin: Option<TextOrigin>,
    pub method: Option<AcquisitionMethod>,
}

#[tauri::command]
pub fn get_session_snapshot(state: State<'_, AppState>) -> Result<Option<SessionSnapshot>, AppError> {
    Ok(lock(&state.snapshot)?.clone())
}

#[tauri::command]
pub fn trigger_action(app: AppHandle, action: String) -> Result<(), AppError> {
    let action: Action = action.parse()?;
    dispatch_action(&app, action);
    Ok(())
}

// ---------- Dịch ----------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitResponse {
    pub attempt_id: Option<AttemptId>,
    pub duplicate: bool,
}

/// Chuyển sự kiện của provider tới UI sau khi coordinator xác nhận không cũ.
struct ChannelSink {
    coordinator: Arc<Mutex<Coordinator>>,
    channel: Channel<ProviderEvent>,
}

impl EventSink for ChannelSink {
    fn emit(&self, event: ProviderEvent) {
        let accepted = self.coordinator.lock().map(|mut c| c.on_event(&event)).unwrap_or(false);
        if accepted {
            let _ = self.channel.send(event);
        }
    }
}

fn build_provider(state: &AppState) -> Result<Arc<dyn TranslationProvider>, AppError> {
    let cfg = lock(&state.config)?.clone();
    if cfg.pause_network {
        return Err(AppError::new(AppErrorCode::NetworkPaused));
    }
    match cfg.provider.id.as_str() {
        azure::PROVIDER_ID => {
            let provider = AzureProvider::new(
                AzureConfig {
                    endpoint: cfg.provider.endpoint,
                    region: cfg.provider.region,
                    secret_ref: SecretRef(cfg.provider.secret_ref),
                },
                Arc::new(ResolverAdapter(state.secrets.clone())),
                HttpPolicy::default(),
            )?;
            Ok(Arc::new(provider))
        }
        _ => Err(AppError::new(AppErrorCode::ConfigMissing)),
    }
}

#[tauri::command]
pub async fn submit_translation(
    state: State<'_, AppState>,
    request_id: u64,
    text: String,
    source: SourceLang,
    target: LangCode,
    on_event: Channel<ProviderEvent>,
) -> Result<SubmitResponse, AppError> {
    let request_id = RequestId(request_id);
    let outcome = lock(&state.coordinator)?.submit(request_id, &text, MAX_INPUT_CHARS)?;
    let attempt_id = match outcome {
        SubmitOutcome::DuplicateIgnored => {
            return Ok(SubmitResponse { attempt_id: None, duplicate: true });
        }
        SubmitOutcome::Started { attempt_id } => attempt_id,
    };

    let token = CancellationToken::new();
    if let Some(old) = lock(&state.cancel)?.replace(token.clone()) {
        old.cancel();
    }

    let sink = ChannelSink { coordinator: state.coordinator.clone(), channel: on_event };
    let request = TranslationRequest {
        request_id,
        text,
        source,
        target,
        config_version: state.config_revision.load(Ordering::Relaxed),
    };

    match build_provider(&state) {
        Ok(provider) => {
            tauri::async_runtime::spawn(async move {
                provider.translate(&request, attempt_id, token, &sink).await;
            });
        }
        // Lỗi cấu hình/mạng-tạm-dừng: báo qua cùng đường sự kiện để UI có một luồng xử lý.
        Err(error) => sink.emit(ProviderEvent {
            request_id,
            attempt_id,
            sequence: 0,
            kind: ProviderEventKind::Failed { error },
        }),
    }
    Ok(SubmitResponse { attempt_id: Some(attempt_id), duplicate: false })
}

#[tauri::command]
pub fn cancel_translation(state: State<'_, AppState>) -> Result<(), AppError> {
    if let Some(token) = lock(&state.cancel)?.take() {
        token.cancel();
    }
    lock(&state.coordinator)?.close();
    *lock(&state.snapshot)? = None;
    Ok(())
}

// ---------- Năng lực, cấu hình, khóa ----------

#[tauri::command]
pub fn get_capabilities() -> Vec<Capability> {
    let mut caps = crate::platform::probe_capabilities();
    caps.push(crate::secrets::probe());
    caps
}

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<Config, AppError> {
    Ok(lock(&state.config)?.clone())
}

#[tauri::command]
pub fn update_config(state: State<'_, AppState>, config: Config) -> Result<(), AppError> {
    if let Some(err) = lock(&state.config_lock)?.clone() {
        // Không ghi đè cấu hình không đọc được hoặc có schema mới hơn (§9.2).
        return Err(err);
    }
    let mut config = config;
    config.schema_version = CURRENT_SCHEMA;
    config::validate(&config)?;
    config::save_atomic(&state.config_path, &config)?;
    *lock(&state.config)? = config;
    state.config_revision.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

fn secret_ref_for(provider_id: &str) -> Result<SecretRef, AppError> {
    let ok = (1..=64).contains(&provider_id.len())
        && provider_id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if ok {
        Ok(SecretRef(format!("provider/{provider_id}")))
    } else {
        Err(AppError::new(AppErrorCode::InputInvalid).with_diagnostic("provider id"))
    }
}

/// Ghi một chiều: UI gửi key vào Rust rồi xóa khỏi state. Không có lệnh đọc lại.
#[tauri::command]
pub fn save_secret(
    state: State<'_, AppState>,
    provider_id: String,
    value: String,
    session_only: bool,
) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Err(AppError::new(AppErrorCode::InputInvalid));
    }
    let reference = secret_ref_for(&provider_id)?;
    if session_only {
        state.secrets.set_session(&reference, &value)
    } else {
        state.secrets.set(&reference, &value)
    }
}

#[tauri::command]
pub fn delete_secret(state: State<'_, AppState>, provider_id: String) -> Result<(), AppError> {
    state.secrets.delete(&secret_ref_for(&provider_id)?)
}

// ---------- Cửa sổ / vòng đời ----------

#[tauri::command]
pub fn close_window(app: AppHandle, state: State<'_, AppState>, label: String) -> Result<(), AppError> {
    let internal = |_| AppError::new(AppErrorCode::Internal);
    match label.as_str() {
        "popup" => {
            if let Some(token) = lock(&state.cancel)?.take() {
                token.cancel();
            }
            lock(&state.coordinator)?.close();
            *lock(&state.snapshot)? = None;
            if let Some(w) = app.get_webview_window("popup") {
                w.hide().map_err(internal)?;
            }
        }
        "settings" => {
            if let Some(w) = app.get_webview_window("settings") {
                w.close().map_err(internal)?;
            }
        }
        _ => return Err(AppError::new(AppErrorCode::InputInvalid)),
    }
    Ok(())
}

/// Thoát thật sự (FR15): không phụ thuộc tray.
#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}
