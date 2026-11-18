//! Quy tắc nguồn/đích và kiểm tra đầu vào (FR09, FR14, BR02).
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppErrorCode};

/// Đích mặc định (A03).
pub const DEFAULT_TARGET: &str = "vi";
/// Chính sách UX/chi phí của Lexora, KHÔNG phải giới hạn API (FR14).
pub const MAX_INPUT_CHARS: usize = 5_000;

/// Mã ngôn ngữ kiểu BCP-47 rút gọn. Chỉ kiểm tra hình thức; việc ngôn ngữ có được
/// provider hỗ trợ hay không do provider quyết định.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LangCode(String);

impl LangCode {
    pub fn parse(raw: &str) -> Result<Self, AppError> {
        let ok_len = (2..=12).contains(&raw.len());
        let ok_chars = raw.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
        let ok_edges = !raw.starts_with('-') && !raw.ends_with('-') && !raw.contains("--");
        if ok_len && ok_chars && ok_edges {
            Ok(Self(raw.to_owned()))
        } else {
            Err(AppError::with_key(AppErrorCode::LanguageUnsupported, "language_unsupported")
                .with_diagnostic("invalid language code shape"))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for LangCode {
    fn default() -> Self {
        Self(DEFAULT_TARGET.to_owned())
    }
}

/// Nguồn: tự nhận diện (mặc định) hoặc chỉ định. Không auto-flip trong MVP (§6.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceLang {
    Auto,
    Explicit(LangCode),
}

/// Từ chối thay vì cắt âm thầm. Đếm theo Unicode scalar value.
pub fn validate_source_text(text: &str, max_chars: usize) -> Result<(), AppError> {
    if text.trim().is_empty() {
        return Err(AppError::with_key(AppErrorCode::InputInvalid, "input_empty"));
    }
    if text.chars().count() > max_chars {
        return Err(AppError::with_key(AppErrorCode::InputInvalid, "input_too_long"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lang_code_shapes() {
        for ok in ["vi", "en", "zh-Hans", "en-US"] {
            assert!(LangCode::parse(ok).is_ok(), "{ok}");
        }
        for bad in ["", "a", "vi_VN", "-vi", "vi-", "vi--VN", "abcdefghijklmn"] {
            assert!(LangCode::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn rejects_blank_text() {
        assert!(validate_source_text("  \n\t", MAX_INPUT_CHARS).is_err());
        assert!(validate_source_text("", MAX_INPUT_CHARS).is_err());
    }

    #[test]
    fn counts_chars_not_bytes() {
        let at_limit = "é".repeat(MAX_INPUT_CHARS);
        assert!(validate_source_text(&at_limit, MAX_INPUT_CHARS).is_ok());
        let over = "é".repeat(MAX_INPUT_CHARS + 1);
        let err = validate_source_text(&over, MAX_INPUT_CHARS).unwrap_err();
        assert_eq!(err.message_key, "error.input_too_long");
    }
}
