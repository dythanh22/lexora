//! Lõi độc lập của Lexora (thiết kế §8).
//!
//! Quy tắc: crate này KHÔNG phụ thuộc Tauri, UI hay API của hệ điều hành.
//! Mọi thứ chạm OS/mạng được inject qua `ports`.
#![forbid(unsafe_code)]

pub mod capability;
pub mod coordinator;
pub mod error;
pub mod language;
pub mod model;
pub mod ports;
pub mod secret;

pub use error::{AppError, AppErrorCode};
pub use language::{LangCode, SourceLang, DEFAULT_TARGET, MAX_INPUT_CHARS};
pub use model::*;
pub use secret::{Secret, SecretRef};
