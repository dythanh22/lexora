//! Bí mật không bao giờ được in ra qua Debug/Display hay serialize.
use std::fmt;

use serde::{Deserialize, Serialize};

/// Tham chiếu không mang giá trị, ví dụ `provider/azure-translator`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SecretRef(pub String);

/// Giá trị bí mật. Cố ý không `Clone`, `Display`, `Serialize`.
/// TODO(M2): cân nhắc `zeroize` khi hủy; thiết kế không hứa xóa bộ nhớ tuyệt đối (§8.4).
pub struct Secret(String);

impl Secret {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(***)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_never_leaks() {
        let s = Secret::new("super-secret".into());
        assert_eq!(format!("{s:?}"), "Secret(***)");
    }
}
