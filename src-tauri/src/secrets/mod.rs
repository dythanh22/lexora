//! Kho khóa (ADR-0008). Hợp đồng nằm ở `lexora_core::ports::SecretStore`.
//!
//! TRẠNG THÁI: chỉ có chế độ khóa-trong-phiên (RAM). Backend hệ điều hành
//! (Credential Manager / Secret Service) là stub cho tới khi POC-K chọn crate.
//! Không fallback plaintext, không vault mật khẩu cứng trong binary (§9.2).
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use lexora_core::capability::{Capability, CapabilityId};
use lexora_core::ports::{resolve_from_store, SecretResolver, SecretStore};
use lexora_core::{AppError, AppErrorCode, Secret, SecretRef};

fn poisoned() -> AppError {
    AppError::new(AppErrorCode::Internal).with_diagnostic("secret map poisoned")
}

#[derive(Default)]
pub struct SessionOnlyStore {
    map: Mutex<HashMap<String, String>>,
}

impl SecretStore for SessionOnlyStore {
    fn get(&self, r: &SecretRef) -> Result<Option<Secret>, AppError> {
        let map = self.map.lock().map_err(|_| poisoned())?;
        Ok(map.get(&r.0).cloned().map(Secret::new))
    }

    fn set(&self, r: &SecretRef, value: &str) -> Result<(), AppError> {
        self.map.lock().map_err(|_| poisoned())?.insert(r.0.clone(), value.to_owned());
        Ok(())
    }

    fn delete(&self, r: &SecretRef) -> Result<(), AppError> {
        self.map.lock().map_err(|_| poisoned())?.remove(&r.0);
        Ok(())
    }
}

/// Stub. TODO(POC-K): Windows Credential Manager, Linux Secret Service.
pub struct OsSecretStore;

fn unavailable() -> AppError {
    AppError::new(AppErrorCode::SecretStoreUnavailable).with_diagnostic("OS store not implemented yet")
}

impl SecretStore for OsSecretStore {
    fn get(&self, _: &SecretRef) -> Result<Option<Secret>, AppError> {
        Err(unavailable())
    }
    fn set(&self, _: &SecretRef, _: &str) -> Result<(), AppError> {
        Err(unavailable())
    }
    fn delete(&self, _: &SecretRef) -> Result<(), AppError> {
        Err(unavailable())
    }
}

/// Ưu tiên khóa-trong-phiên, sau đó mới tới kho hệ điều hành.
pub struct LayeredStore {
    os: OsSecretStore,
    session: SessionOnlyStore,
}

impl LayeredStore {
    pub fn new() -> Self {
        Self { os: OsSecretStore, session: SessionOnlyStore::default() }
    }

    /// Người dùng chọn "chỉ dùng trong phiên": không bao giờ chạm kho hệ điều hành.
    pub fn set_session(&self, r: &SecretRef, value: &str) -> Result<(), AppError> {
        self.session.set(r, value)
    }
}

impl Default for LayeredStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretStore for LayeredStore {
    fn get(&self, r: &SecretRef) -> Result<Option<Secret>, AppError> {
        if let Some(s) = self.session.get(r)? {
            return Ok(Some(s));
        }
        self.os.get(r)
    }

    fn set(&self, r: &SecretRef, value: &str) -> Result<(), AppError> {
        self.os.set(r, value)
    }

    fn delete(&self, r: &SecretRef) -> Result<(), AppError> {
        self.session.delete(r)?;
        // Xóa ở kho hệ điều hành lỗi thì báo lỗi, không im lặng (§9.2).
        self.os.delete(r)
    }
}

/// Cầu nối sang `SecretResolver` cho crate providers.
pub struct ResolverAdapter(pub Arc<LayeredStore>);

impl SecretResolver for ResolverAdapter {
    fn resolve(&self, r: &SecretRef) -> Result<Secret, AppError> {
        resolve_from_store(&*self.0, r)
    }
}

pub fn probe() -> Capability {
    Capability::untested(
        CapabilityId::SecretStore,
        "POC-K chưa chạy; hiện chỉ có chế độ khóa dùng trong phiên.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r() -> SecretRef {
        SecretRef("provider/test".into())
    }

    #[test]
    fn session_key_resolves_and_is_not_persisted_elsewhere() {
        let s = LayeredStore::new();
        s.set_session(&r(), "k").unwrap();
        assert_eq!(s.get(&r()).unwrap().unwrap().expose(), "k");
    }

    #[test]
    fn os_store_unavailable_does_not_fall_back_to_plaintext() {
        let s = LayeredStore::new();
        let err = s.set(&r(), "k").unwrap_err();
        assert_eq!(err.code, AppErrorCode::SecretStoreUnavailable);
        assert!(s.session.get(&r()).unwrap().is_none());
    }

    #[test]
    fn resolve_without_key_reports_store_problem_not_a_panic() {
        let a = ResolverAdapter(Arc::new(LayeredStore::new()));
        assert!(a.resolve(&r()).is_err());
    }
}
