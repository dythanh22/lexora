//! Cấu hình không bí mật: có version, ghi atomic, migration (FR10, §9.2).
//! Secret không bao giờ nằm ở đây, chỉ có `secret_ref`.
use std::fs;
use std::io::Write;
use std::path::Path;

use lexora_core::{AppError, AppErrorCode, LangCode, DEFAULT_TARGET};
use lexora_providers::azure;
use lexora_providers::endpoint::validate_endpoint;
use serde::{Deserialize, Serialize};

pub const CURRENT_SCHEMA: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ProviderConfig {
    pub id: String,
    pub endpoint: String,
    pub region: Option<String>,
    pub secret_ref: String,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            id: azure::PROVIDER_ID.to_owned(),
            endpoint: azure::DEFAULT_ENDPOINT.to_owned(),
            region: None,
            secret_ref: format!("provider/{}", azure::PROVIDER_ID),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
    pub schema_version: u32,
    pub ui_language: String,
    pub target_language: String,
    pub provider: ProviderConfig,
    /// Tạm dừng mọi gửi dịch (FR13).
    pub pause_network: bool,
    pub run_in_background: bool,
    /// Mặc định tắt (§4.1).
    pub autostart: bool,
    pub popup_size: Option<[f64; 2]>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA,
            ui_language: "vi".to_owned(),
            target_language: DEFAULT_TARGET.to_owned(),
            provider: ProviderConfig::default(),
            pause_network: false,
            run_in_background: false,
            autostart: false,
            popup_size: None,
        }
    }
}

fn incompatible(why: &str) -> AppError {
    AppError::new(AppErrorCode::ConfigIncompatible).with_diagnostic(why)
}

/// Chạy trước khi khởi tạo provider. Schema mới hơn bị từ chối, không đoán.
pub fn migrate(value: serde_json::Value) -> Result<Config, AppError> {
    let version = value.get("schemaVersion").and_then(|v| v.as_u64()).unwrap_or(1);
    if version > u64::from(CURRENT_SCHEMA) {
        return Err(incompatible("config schema newer than this build"));
    }
    // v1 -> v1: chưa có bước migrate. Thêm `match version { .. }` khi nâng schema.
    let mut cfg: Config =
        serde_json::from_value(value).map_err(|_| incompatible("config unreadable"))?;
    cfg.schema_version = CURRENT_SCHEMA;
    Ok(cfg)
}

pub fn load(path: &Path) -> Result<Config, AppError> {
    let raw = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Config::default()),
        Err(_) => return Err(incompatible("config not readable")),
    };
    let value: serde_json::Value =
        serde_json::from_str(&raw).map_err(|_| incompatible("config is not valid json"))?;
    migrate(value)
}

/// Ghi tạm rồi đổi tên để không để lại file dở dang.
pub fn save_atomic(path: &Path, cfg: &Config) -> Result<(), AppError> {
    let io_err = |_| AppError::new(AppErrorCode::Internal).with_diagnostic("config write failed");
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(io_err)?;
    }
    let tmp = path.with_extension("json.tmp");
    let json = serde_json::to_vec_pretty(cfg)
        .map_err(|_| AppError::new(AppErrorCode::Internal).with_diagnostic("config serialize"))?;
    {
        let mut f = fs::File::create(&tmp).map_err(io_err)?;
        f.write_all(&json).map_err(io_err)?;
        f.sync_all().map_err(io_err)?;
    }
    fs::rename(&tmp, path).map_err(io_err)
}

/// Kiểm tra nghiệp vụ trước khi lưu (Rust luôn validate lại input từ UI — §8.5).
pub fn validate(cfg: &Config) -> Result<(), AppError> {
    LangCode::parse(&cfg.target_language)?;
    if !matches!(cfg.ui_language.as_str(), "vi" | "en") {
        return Err(AppError::new(AppErrorCode::InputInvalid).with_diagnostic("ui language"));
    }
    validate_endpoint(&cfg.provider.endpoint, false)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("lexora-{}-{nanos}", std::process::id())).join(name)
    }

    #[test]
    fn missing_file_gives_defaults() {
        let cfg = load(&temp_path("none.json")).unwrap();
        assert_eq!(cfg, Config::default());
        assert_eq!(cfg.target_language, "vi");
        assert!(!cfg.autostart);
    }

    #[test]
    fn roundtrip_atomic() {
        let path = temp_path("config.json");
        let cfg = Config { pause_network: true, ..Config::default() };
        save_atomic(&path, &cfg).unwrap();
        assert_eq!(load(&path).unwrap(), cfg);
        assert!(!path.with_extension("json.tmp").exists());
    }

    #[test]
    fn newer_schema_is_rejected_not_overwritten() {
        let v = serde_json::json!({ "schemaVersion": CURRENT_SCHEMA + 1 });
        assert_eq!(migrate(v).unwrap_err().code, AppErrorCode::ConfigIncompatible);
    }

    #[test]
    fn corrupt_file_is_an_error() {
        let path = temp_path("bad.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "{ not json").unwrap();
        assert!(load(&path).is_err());
    }

    #[test]
    fn config_never_contains_a_secret_field() {
        let json = serde_json::to_string(&Config::default()).unwrap().to_lowercase();
        assert!(!json.contains("apikey") && !json.contains("\"key\""));
    }

    #[test]
    fn validation_rejects_bad_endpoint_and_language() {
        let bad_endpoint = Config {
            provider: ProviderConfig { endpoint: "http://evil.example".into(), ..Default::default() },
            ..Config::default()
        };
        assert!(validate(&bad_endpoint).is_err());
        let bad_lang = Config { target_language: "vi_VN".into(), ..Config::default() };
        assert!(validate(&bad_lang).is_err());
        assert!(validate(&Config::default()).is_ok());
    }
}
