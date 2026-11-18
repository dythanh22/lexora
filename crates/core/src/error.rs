//! Lỗi ổn định, an toàn để hiển thị (thiết kế §8.3 `AppError`).
//! Không chứa raw HTTP body, key hay văn bản nguồn.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppErrorCode {
    InputInvalid,
    ConfigMissing,
    ConfigIncompatible,
    Auth,
    Quota,
    RateLimited,
    Timeout,
    Network,
    NetworkPaused,
    InvalidResponse,
    LanguageUnsupported,
    Cancelled,
    CaptureUnsupported,
    CaptureFailed,
    SecretStoreUnavailable,
    Internal,
}

impl AppErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InputInvalid => "input_invalid",
            Self::ConfigMissing => "config_missing",
            Self::ConfigIncompatible => "config_incompatible",
            Self::Auth => "auth",
            Self::Quota => "quota",
            Self::RateLimited => "rate_limited",
            Self::Timeout => "timeout",
            Self::Network => "network",
            Self::NetworkPaused => "network_paused",
            Self::InvalidResponse => "invalid_response",
            Self::LanguageUnsupported => "language_unsupported",
            Self::Cancelled => "cancelled",
            Self::CaptureUnsupported => "capture_unsupported",
            Self::CaptureFailed => "capture_failed",
            Self::SecretStoreUnavailable => "secret_store_unavailable",
            Self::Internal => "internal",
        }
    }

    /// Gợi ý cho UI có nên hiện nút "Thử lại" ngay hay không.
    /// Retry luôn do người dùng bấm (BR04); không bao giờ tự động.
    pub const fn retryable(self) -> bool {
        matches!(
            self,
            Self::Timeout | Self::Network | Self::RateLimited | Self::InvalidResponse
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[serde(rename_all = "camelCase")]
#[error("{code:?} ({message_key})")]
pub struct AppError {
    pub code: AppErrorCode,
    pub retryable: bool,
    /// Khóa i18n, ví dụ `error.quota`.
    pub message_key: String,
    /// Chẩn đoán đã lọc, tối đa 200 ký tự. Không đưa nội dung người dùng vào đây.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<String>,
}

impl AppError {
    pub fn new(code: AppErrorCode) -> Self {
        Self {
            code,
            retryable: code.retryable(),
            message_key: format!("error.{}", code.as_str()),
            diagnostic: None,
        }
    }

    pub fn with_key(code: AppErrorCode, key: &str) -> Self {
        Self { message_key: format!("error.{key}"), ..Self::new(code) }
    }

    pub fn with_diagnostic(mut self, text: impl AsRef<str>) -> Self {
        self.diagnostic = Some(text.as_ref().chars().take(200).collect());
        self
    }
}
