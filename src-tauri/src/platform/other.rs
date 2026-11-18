//! Nền tảng ngoài phạm vi (macOS, v.v.): chỉ để `cargo check` chạy được trên máy dev.
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
    vec![Capability::untested(CapabilityId::SelectionDirect, "Nền tảng ngoài phạm vi v1.")]
}

pub fn popup_takes_focus() -> bool {
    true
}
