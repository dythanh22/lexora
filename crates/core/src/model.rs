//! Mô hình dữ liệu trung lập (thiết kế §8.3).
use std::str::FromStr;
use std::sync::OnceLock;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppErrorCode};
use crate::language::{LangCode, SourceLang};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RequestId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AttemptId(pub u32);

/// Ba điểm vào (ADR-0003). Đây là allowlist duy nhất cho CLI/IPC (FR11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    TranslateSelection,
    TranslateClipboard,
    OpenInput,
}

impl Action {
    pub const fn cli_name(self) -> &'static str {
        match self {
            Self::TranslateSelection => "selection",
            Self::TranslateClipboard => "clipboard",
            Self::OpenInput => "input",
        }
    }

    /// Tìm `--action <tên>` hoặc `--action=<tên>` trong argv.
    /// `Ok(None)` nếu không có cờ; `Err` nếu giá trị không thuộc allowlist.
    /// Tuyệt đối không nhận văn bản hay key qua tham số dòng lệnh.
    pub fn from_cli_args(args: &[String]) -> Result<Option<Self>, AppError> {
        let mut it = args.iter();
        while let Some(a) = it.next() {
            if let Some(v) = a.strip_prefix("--action=") {
                return v.parse::<Self>().map(Some);
            }
            if a == "--action" {
                return match it.next() {
                    Some(v) => v.parse::<Self>().map(Some),
                    None => Err(AppError::new(AppErrorCode::InputInvalid)),
                };
            }
        }
        Ok(None)
    }
}

impl FromStr for Action {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "selection" => Ok(Self::TranslateSelection),
            "clipboard" => Ok(Self::TranslateClipboard),
            "input" => Ok(Self::OpenInput),
            _ => Err(AppError::new(AppErrorCode::InputInvalid).with_diagnostic("unknown action")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextOrigin {
    Selection,
    Clipboard,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcquisitionMethod {
    Uia,
    AtSpi,
    Primary,
    SyntheticCopy,
    ClipboardRead,
    UserInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcquiredText {
    pub text: String,
    pub origin: TextOrigin,
    pub method: AcquisitionMethod,
    /// Chỉ khi nền tảng xác minh được; không suy đoán từ nội dung văn bản.
    pub source_app: Option<String>,
}

/// Ngữ cảnh lúc kích hoạt. `activation_token` sống ngắn, không log, không export.
#[derive(Debug, Clone)]
pub struct TriggerContext {
    pub action: Action,
    pub backend: String,
    pub monotonic_ms: u64,
    pub activation_token: Option<String>,
}

impl TriggerContext {
    pub fn now(action: Action, backend: &str) -> Self {
        static START: OnceLock<Instant> = OnceLock::new();
        let start = START.get_or_init(Instant::now);
        Self {
            action,
            backend: backend.to_owned(),
            monotonic_ms: start.elapsed().as_millis() as u64,
            activation_token: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationRequest {
    pub request_id: RequestId,
    pub text: String,
    pub source: SourceLang,
    pub target: LangCode,
    /// Tăng mỗi lần cấu hình provider thay đổi.
    pub config_version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMode {
    None,
    ApiKey,
    Bearer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderCapabilities {
    pub auto_detect: bool,
    pub streaming: bool,
    pub max_input_chars: Option<usize>,
    pub auth: AuthMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Detected {
    pub language: String,
    pub confidence: Option<f32>,
}

/// Hợp đồng sự kiện (§8.3): Started, Delta, Completed, Failed.
/// `Completed` luôn mang toàn văn cuối cùng; UI thay thế, không nối thêm.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProviderEventKind {
    Started,
    Delta { text: String },
    Completed { text: String, detected: Option<Detected> },
    Failed { error: AppError },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderEvent {
    pub request_id: RequestId,
    pub attempt_id: AttemptId,
    /// Tăng dần trong một attempt; sự kiện không tăng bị bỏ.
    pub sequence: u32,
    #[serde(flatten)]
    pub kind: ProviderEventKind,
}

impl ProviderEvent {
    pub fn is_terminal(&self) -> bool {
        matches!(self.kind, ProviderEventKind::Completed { .. } | ProviderEventKind::Failed { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn cli_action_allowlist() {
        let a = Action::from_cli_args(&args(&["lexora", "--action", "selection"])).unwrap();
        assert_eq!(a, Some(Action::TranslateSelection));
        let b = Action::from_cli_args(&args(&["lexora", "--action=input"])).unwrap();
        assert_eq!(b, Some(Action::OpenInput));
        assert_eq!(Action::from_cli_args(&args(&["lexora"])).unwrap(), None);
        assert!(Action::from_cli_args(&args(&["lexora", "--action", "rm -rf"])).is_err());
        assert!(Action::from_cli_args(&args(&["lexora", "--action"])).is_err());
    }

    #[test]
    fn event_json_shape_is_flat_and_camel_case() {
        let ev = ProviderEvent {
            request_id: RequestId(7),
            attempt_id: AttemptId(1),
            sequence: 2,
            kind: ProviderEventKind::Delta { text: "xin".into() },
        };
        let json = serde_json::to_string(&ev).unwrap();
        assert!(json.contains("\"requestId\":7"));
        assert!(json.contains("\"kind\":\"delta\""));
        assert!(json.contains("\"text\":\"xin\""));
    }
}
