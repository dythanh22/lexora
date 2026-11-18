//! Gợi ý backend hiển thị từ biến môi trường. Chỉ là GỢI Ý (thiết kế §7.5):
//! cần dò thêm runtime (display thật của GDK, interface portal, kết quả probe).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayHint {
    Wayland,
    X11,
    Unknown,
}

pub fn detect_hint_with(get: impl Fn(&str) -> Option<String>) -> DisplayHint {
    let non_empty = |k: &str| get(k).is_some_and(|v| !v.is_empty());
    let session = get("XDG_SESSION_TYPE").map(|s| s.to_ascii_lowercase());
    if session.as_deref() == Some("wayland") || non_empty("WAYLAND_DISPLAY") {
        DisplayHint::Wayland
    } else if session.as_deref() == Some("x11") || non_empty("DISPLAY") {
        DisplayHint::X11
    } else {
        DisplayHint::Unknown
    }
}

pub fn detect_hint() -> DisplayHint {
    detect_hint_with(|k| std::env::var(k).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn hint(pairs: &[(&str, &str)]) -> DisplayHint {
        let m: HashMap<String, String> =
            pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        detect_hint_with(|k| m.get(k).cloned())
    }

    #[test]
    fn wayland_wins_over_xwayland_display() {
        let h = hint(&[("XDG_SESSION_TYPE", "wayland"), ("WAYLAND_DISPLAY", "wayland-0"), ("DISPLAY", ":0")]);
        assert_eq!(h, DisplayHint::Wayland);
    }

    #[test]
    fn plain_x11_and_unknown() {
        assert_eq!(hint(&[("XDG_SESSION_TYPE", "x11"), ("DISPLAY", ":0")]), DisplayHint::X11);
        assert_eq!(hint(&[]), DisplayHint::Unknown);
    }
}
