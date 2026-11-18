//! Chính sách HTTP dùng chung (NFR08, §9.3).
use std::time::Duration;

use lexora_core::{AppError, AppErrorCode};

#[derive(Debug, Clone)]
pub struct HttpPolicy {
    /// Khởi điểm theo NFR08; chốt sau số đo thật.
    pub connect_timeout: Duration,
    pub total_timeout: Duration,
    pub max_response_bytes: usize,
}

impl Default for HttpPolicy {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            total_timeout: Duration::from_secs(20),
            max_response_bytes: 1024 * 1024,
        }
    }
}

/// Không theo redirect: tránh chuyển header xác thực sang host khác (§9.3).
/// Không bật cache đĩa: ứng dụng không lưu phản hồi HTTP (§5.2).
pub fn build_client(policy: &HttpPolicy) -> Result<reqwest::Client, AppError> {
    reqwest::Client::builder()
        .connect_timeout(policy.connect_timeout)
        .timeout(policy.total_timeout)
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(concat!("Lexora/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|_| AppError::new(AppErrorCode::Internal).with_diagnostic("http client build"))
}

/// Lỗi vận chuyển -> AppError, luôn bỏ URL (có thể chứa thông tin nhạy cảm).
pub fn map_transport_error(err: reqwest::Error) -> AppError {
    let err = err.without_url();
    if err.is_timeout() {
        AppError::new(AppErrorCode::Timeout)
    } else {
        AppError::new(AppErrorCode::Network)
    }
}
