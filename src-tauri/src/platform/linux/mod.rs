//! Ubuntu: GNOME Wayland là mục tiêu chính, X11 là bổ sung (ADR-0002, §7).
use lexora_core::capability::{Capability, CapabilityId};
use lexora_core::ports::{ClipboardReader, SelectionReader};

use super::null::NullClipboardReader;

mod atspi;
mod portal;
mod session;
mod x11;

pub use session::{detect_hint, DisplayHint};

pub fn selection_reader() -> Box<dyn SelectionReader> {
    Box::new(atspi::AtSpiSelectionReader)
}

pub fn clipboard_reader() -> Box<dyn ClipboardReader> {
    // TODO(POC-S/W): đọc qua giao thức Wayland/X11 thông thường SAU khi cửa sổ có focus.
    Box::new(NullClipboardReader)
}

pub fn probe() -> Vec<Capability> {
    let hint = match detect_hint() {
        DisplayHint::Wayland => "wayland (gợi ý từ biến môi trường)",
        DisplayHint::X11 => "x11 (gợi ý từ biến môi trường)",
        DisplayHint::Unknown => "không rõ",
    };
    let mut caps: Vec<Capability> = [
        (CapabilityId::GlobalHotkey, "POC-H: portal GlobalShortcuts (Wayland) hoặc XGrabKey (X11)."),
        (CapabilityId::SelectionDirect, "POC-S: AT-SPI; phụ thuộc toolkit/ứng dụng."),
        (CapabilityId::ClipboardRead, "POC-W: đọc clipboard sau khi cửa sổ có focus."),
        (CapabilityId::SyntheticCopy, "Ngoài MVP trên Wayland (ADR-0005); X11 cần POC-C."),
        (CapabilityId::PopupNoFocus, "POC-W."),
        (CapabilityId::PopupPosition, "Wayland: compositor quyết định vị trí (ADR-0006)."),
    ]
    .into_iter()
    .map(|(id, why)| Capability::untested(id, why))
    .collect();
    // Không nhận dạng backend chỉ bằng biến môi trường (§7.5): đây mới là gợi ý.
    for c in &mut caps {
        c.backend = Some(hint.to_owned());
    }
    caps
}

pub fn popup_takes_focus() -> bool {
    // Wayland và chưa rõ: cửa sổ có focus (an toàn). X11: không lấy focus (cần POC-W).
    !matches!(detect_hint(), DisplayHint::X11)
}
