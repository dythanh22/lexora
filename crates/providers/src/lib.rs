//! Adapter engine dịch (thiết kế §6, §8).
//!
//! MVP: một engine tự động duy nhất (Azure Translator, ứng viên — ADR-0004).
//! Thêm engine = thêm adapter + cấu hình + test hợp đồng; KHÔNG sửa coordinator/UI.
//! Không có plugin động (§8.1).
#![forbid(unsafe_code)]

pub mod azure;
pub mod endpoint;
pub mod fake;
pub mod http;
