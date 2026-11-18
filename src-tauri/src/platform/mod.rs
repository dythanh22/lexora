//! Platform adapters (thiết kế §7, §8.1). Mỗi HĐH cung cấp cùng một bề mặt:
//! `selection_reader`, `clipboard_reader`, `probe`, `popup_takes_focus`.
//!
//! TRẠNG THÁI: toàn bộ là KHUNG. Reader mặc định trả `Ok(None)` (không hỗ trợ) và
//! mọi năng lực báo `Untested` cho tới khi PoC tương ứng có bằng chứng (docs/support-matrix.md).
use lexora_core::capability::Capability;
use lexora_core::ports::{ClipboardReader, SelectionReader};

mod null;

#[cfg(windows)]
mod win;
#[cfg(windows)]
use win as os;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as os;

#[cfg(not(any(windows, target_os = "linux")))]
mod other;
#[cfg(not(any(windows, target_os = "linux")))]
use other as os;

pub fn selection_reader() -> Box<dyn SelectionReader> {
    os::selection_reader()
}

pub fn clipboard_reader() -> Box<dyn ClipboardReader> {
    os::clipboard_reader()
}

pub fn probe_capabilities() -> Vec<Capability> {
    os::probe()
}

/// `true` nếu cửa sổ kết quả nên nhận focus khi mở (ADR-0006).
pub fn popup_takes_focus() -> bool {
    os::popup_takes_focus()
}
