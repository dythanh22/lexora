//! Windows 11 x64 (nền tảng chính). Xem thiết kế §7.2.
//!
//! TODO(POC-S): UI Automation `TextPattern.GetSelection` trên worker riêng có COM apartment
//!   đúng; không chạy trên UI thread; kiểm tra selection rỗng và trường mật khẩu.
//! TODO(POC-C): Copy giả lập qua `SendInput` + snapshot/restore clipboard (chịu UIPI;
//!   không tự nâng quyền; không gửi Ctrl+C vào terminal).
//! TODO(POC-H): `RegisterHotKey` qua plugin global-shortcut.
//! TODO(POC-W): `WS_EX_NOACTIVATE` để mở popup không lấy focus.
use lexora_core::capability::{Capability, CapabilityId};
use lexora_core::ports::{ClipboardReader, SelectionReader};

use super::null::{NullClipboardReader, NullSelectionReader};

pub fn selection_reader() -> Box<dyn SelectionReader> {
    Box::new(NullSelectionReader)
}

pub fn clipboard_reader() -> Box<dyn ClipboardReader> {
    Box::new(NullClipboardReader)
}

pub fn probe() -> Vec<Capability> {
    const WHY: &str = "Chưa có PoC trên Windows.";
    [
        CapabilityId::GlobalHotkey,
        CapabilityId::SelectionDirect,
        CapabilityId::ClipboardRead,
        CapabilityId::SyntheticCopy,
        CapabilityId::PopupNoFocus,
        CapabilityId::PopupPosition,
    ]
    .into_iter()
    .map(|id| Capability::untested(id, WHY))
    .collect()
}

/// Windows: mục tiêu là không lấy focus khi mở (nếu POC-W đạt).
pub fn popup_takes_focus() -> bool {
    false
}
