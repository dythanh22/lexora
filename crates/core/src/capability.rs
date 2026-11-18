//! Mô hình năng lực (§7.5): không chỉ boolean.
//! Phân biệt năng lực của HĐH với kết quả đọc selection ở một ứng dụng cụ thể.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityId {
    GlobalHotkey,
    SelectionDirect,
    ClipboardRead,
    SyntheticCopy,
    PopupNoFocus,
    PopupPosition,
    SecretStore,
    Autostart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityState {
    Supported,
    Conditional,
    Unavailable,
    Denied,
    /// Mặc định cho mọi thứ chưa có bằng chứng PoC (§0: [POC] chưa phải cam kết).
    Untested,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    pub id: CapabilityId,
    pub state: CapabilityState,
    pub backend: Option<String>,
    pub condition: Option<String>,
    pub reason: Option<String>,
    pub fallback: Option<String>,
    /// Unix seconds của lần probe gần nhất.
    pub checked_at: Option<u64>,
}

impl Capability {
    pub fn untested(id: CapabilityId, reason: &str) -> Self {
        Self {
            id,
            state: CapabilityState::Untested,
            backend: None,
            condition: None,
            reason: Some(reason.to_owned()),
            fallback: None,
            checked_at: None,
        }
    }
}
