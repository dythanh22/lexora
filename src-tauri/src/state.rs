use std::path::PathBuf;
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, Mutex, MutexGuard};

use lexora_core::coordinator::Coordinator;
use lexora_core::{AppError, AppErrorCode};
use tokio_util::sync::CancellationToken;

use crate::commands::SessionSnapshot;
use crate::config::{self, Config};
use crate::secrets::LayeredStore;

pub struct AppState {
    /// Không giữ khóa này qua `.await`.
    pub coordinator: Arc<Mutex<Coordinator>>,
    pub config: Mutex<Config>,
    pub config_path: PathBuf,
    /// Có giá trị khi cấu hình đọc được nhưng không được phép ghi (schema mới hơn): §9.2.
    pub config_lock: Mutex<Option<AppError>>,
    pub config_revision: AtomicU32,
    pub secrets: Arc<LayeredStore>,
    pub cancel: Mutex<Option<CancellationToken>>,
    pub snapshot: Mutex<Option<SessionSnapshot>>,
}

impl AppState {
    pub fn new(config_path: PathBuf) -> Self {
        let (cfg, lock) = match config::load(&config_path) {
            Ok(c) => (c, None),
            // Không ghi đè file gốc khi không đọc được (hỏng hoặc schema mới).
            Err(e) => (Config::default(), Some(e)),
        };
        Self {
            coordinator: Arc::new(Mutex::new(Coordinator::new())),
            config: Mutex::new(cfg),
            config_path,
            config_lock: Mutex::new(lock),
            config_revision: AtomicU32::new(0),
            secrets: Arc::new(LayeredStore::new()),
            cancel: Mutex::new(None),
            snapshot: Mutex::new(None),
        }
    }
}

pub fn lock<T>(m: &Mutex<T>) -> Result<MutexGuard<'_, T>, AppError> {
    m.lock()
        .map_err(|_| AppError::new(AppErrorCode::Internal).with_diagnostic("state poisoned"))
}
