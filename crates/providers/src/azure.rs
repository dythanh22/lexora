//! Azure Translator Text v3 (ứng viên MVP, ADR-0004).
//!
//! TRẠNG THÁI: khung + hàm thuần có test. Chưa gọi dịch vụ thật.
//! POC-T phải xác minh: mã lỗi thật (tài liệu P12: 403001 = hết free quota),
//! ánh xạ HTTP 400, header region, chất lượng EN→VI trên corpus.
use std::sync::Arc;

use lexora_core::ports::{BoxFuture, EventSink, SecretResolver, TranslationProvider};
use lexora_core::{
    AppError, AppErrorCode, AttemptId, AuthMode, Detected, ProviderCapabilities, ProviderEvent,
    ProviderEventKind, SecretRef, SourceLang, TranslationRequest,
};
use reqwest::header::HeaderValue;
use serde::Deserialize;
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::endpoint::validate_endpoint;
use crate::http::{build_client, map_transport_error, HttpPolicy};

pub const DEFAULT_ENDPOINT: &str = "https://api.cognitive.microsofttranslator.com";
pub const API_VERSION: &str = "3.0";
pub const PROVIDER_ID: &str = "azure-translator";

#[derive(Debug, Clone)]
pub struct AzureConfig {
    pub endpoint: String,
    /// Bắt buộc với tài nguyên theo vùng/multi-service; để trống nếu không dùng.
    pub region: Option<String>,
    pub secret_ref: SecretRef,
}

pub struct AzureProvider {
    endpoint: Url,
    region: Option<String>,
    secret_ref: SecretRef,
    secrets: Arc<dyn SecretResolver>,
    client: reqwest::Client,
    policy: HttpPolicy,
}

impl AzureProvider {
    pub fn new(
        cfg: AzureConfig,
        secrets: Arc<dyn SecretResolver>,
        policy: HttpPolicy,
    ) -> Result<Self, AppError> {
        let endpoint = validate_endpoint(&cfg.endpoint, false)?;
        let client = build_client(&policy)?;
        Ok(Self {
            endpoint,
            region: cfg.region.filter(|r| !r.trim().is_empty()),
            secret_ref: cfg.secret_ref,
            secrets,
            client,
            policy,
        })
    }

    async fn call(
        &self,
        req: &TranslationRequest,
    ) -> Result<(String, Option<Detected>), AppError> {
        let key = self.secrets.resolve(&self.secret_ref)?;
        let mut key_header = HeaderValue::from_str(key.expose())
            .map_err(|_| AppError::new(AppErrorCode::Auth).with_diagnostic("key not header-safe"))?;
        key_header.set_sensitive(true);

        let from = match &req.source {
            SourceLang::Auto => None,
            SourceLang::Explicit(l) => Some(l.as_str()),
        };
        let url = build_url(&self.endpoint, from, req.target.as_str())?;

        let mut rb = self
            .client
            .post(url)
            .header("Ocp-Apim-Subscription-Key", key_header)
            .json(&serde_json::json!([{ "Text": req.text }]));
        if let Some(region) = &self.region {
            rb = rb.header("Ocp-Apim-Subscription-Region", region.as_str());
        }

        let resp = rb.send().await.map_err(map_transport_error)?;
        let status = resp.status();
        let bytes = resp.bytes().await.map_err(map_transport_error)?;
        // TODO(POC-T): giới hạn khi đọc dần thay vì sau khi đã nhận đủ.
        if bytes.len() > self.policy.max_response_bytes {
            return Err(AppError::new(AppErrorCode::InvalidResponse)
                .with_diagnostic("response too large"));
        }
        let body = String::from_utf8_lossy(&bytes);
        if status.is_success() {
            parse_success(&body)
        } else {
            Err(map_error(status.as_u16(), &body))
        }
    }
}

impl TranslationProvider for AzureProvider {
    fn id(&self) -> &'static str {
        PROVIDER_ID
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            auto_detect: true,
            streaming: false,
            max_input_chars: None,
            auth: AuthMode::ApiKey,
        }
    }

    fn translate<'a>(
        &'a self,
        request: &'a TranslationRequest,
        attempt: AttemptId,
        cancel: CancellationToken,
        sink: &'a dyn EventSink,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let mut seq = 0u32;
            let mut emit = |kind: ProviderEventKind| {
                sink.emit(ProviderEvent {
                    request_id: request.request_id,
                    attempt_id: attempt,
                    sequence: seq,
                    kind,
                });
                seq += 1;
            };
            emit(ProviderEventKind::Started);
            let outcome = tokio::select! {
                _ = cancel.cancelled() => None,
                r = self.call(request) => Some(r),
            };
            match outcome {
                // Đã hủy: không phát gì; coordinator đã loại phiên (BR05).
                None => {}
                Some(Ok((text, detected))) => emit(ProviderEventKind::Completed { text, detected }),
                Some(Err(error)) => emit(ProviderEventKind::Failed { error }),
            }
        })
    }
}

/// `{endpoint}/translate?api-version=3.0&to=vi[&from=en]`
pub fn build_url(base: &Url, from: Option<&str>, to: &str) -> Result<Url, AppError> {
    let mut url = base.clone();
    url.path_segments_mut()
        .map_err(|_| AppError::new(AppErrorCode::Internal).with_diagnostic("endpoint cannot be base"))?
        .pop_if_empty()
        .push("translate");
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("api-version", API_VERSION);
        q.append_pair("to", to);
        if let Some(f) = from {
            q.append_pair("from", f);
        }
    }
    Ok(url)
}

#[derive(Deserialize)]
struct Item {
    #[serde(rename = "detectedLanguage")]
    detected_language: Option<Det>,
    translations: Vec<Tr>,
}

#[derive(Deserialize)]
struct Det {
    language: String,
    score: Option<f32>,
}

#[derive(Deserialize)]
struct Tr {
    text: String,
}

pub fn parse_success(body: &str) -> Result<(String, Option<Detected>), AppError> {
    let bad = || AppError::new(AppErrorCode::InvalidResponse);
    let items: Vec<Item> = serde_json::from_str(body).map_err(|_| bad())?;
    let first = items.into_iter().next().ok_or_else(bad)?;
    let tr = first.translations.into_iter().next().ok_or_else(bad)?;
    let detected = first
        .detected_language
        .map(|d| Detected { language: d.language, confidence: d.score });
    Ok((tr.text, detected))
}

#[derive(Deserialize)]
struct ErrBody {
    error: Option<ErrInner>,
}

#[derive(Deserialize)]
struct ErrInner {
    code: Option<serde_json::Value>,
}

fn error_code(body: &str) -> Option<u64> {
    let inner = serde_json::from_str::<ErrBody>(body).ok()?.error?;
    match inner.code? {
        serde_json::Value::Number(n) => n.as_u64(),
        serde_json::Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

/// Ánh xạ lỗi HTTP -> AppError. KHÔNG đưa raw body vào kết quả.
pub fn map_error(status: u16, body: &str) -> AppError {
    use AppErrorCode as C;
    let code = error_code(body);
    let mapped = match (status, code) {
        (_, Some(403001)) => C::Quota,
        (401 | 403, _) => C::Auth,
        (429, _) => C::RateLimited,
        (408 | 504, _) => C::Timeout,
        (500..=599, _) => C::Network,
        // TODO(POC-T): xác minh mã 400xxx (ví dụ ngôn ngữ không hỗ trợ) theo P12.
        _ => C::Internal,
    };
    AppError::new(mapped).with_diagnostic(format!("http {status} code {code:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Url {
        Url::parse(DEFAULT_ENDPOINT).unwrap()
    }

    #[test]
    fn url_without_source_lets_service_detect() {
        let u = build_url(&base(), None, "vi").unwrap();
        assert_eq!(
            u.as_str(),
            "https://api.cognitive.microsofttranslator.com/translate?api-version=3.0&to=vi"
        );
    }

    #[test]
    fn url_with_explicit_source() {
        let u = build_url(&base(), Some("en"), "vi").unwrap();
        assert!(u.as_str().ends_with("&to=vi&from=en"));
    }

    #[test]
    fn parses_translation_and_detection() {
        let body = r#"[{"detectedLanguage":{"language":"en","score":0.98},
                        "translations":[{"text":"xin chào","to":"vi"}]}]"#;
        let (text, det) = parse_success(body).unwrap();
        assert_eq!(text, "xin chào");
        assert_eq!(det.unwrap().language, "en");
    }

    #[test]
    fn parses_without_detection_field() {
        let body = r#"[{"translations":[{"text":"chào","to":"vi"}]}]"#;
        let (text, det) = parse_success(body).unwrap();
        assert_eq!(text, "chào");
        assert!(det.is_none());
    }

    #[test]
    fn malformed_or_empty_is_invalid_response() {
        for body in ["", "not json", "[]", r#"[{"translations":[]}]"#] {
            assert_eq!(parse_success(body).unwrap_err().code, AppErrorCode::InvalidResponse);
        }
    }

    #[test]
    fn error_mapping() {
        let quota = r#"{"error":{"code":403001,"message":"x"}}"#;
        assert_eq!(map_error(403, quota).code, AppErrorCode::Quota);
        assert_eq!(map_error(401, "").code, AppErrorCode::Auth);
        assert_eq!(map_error(403, "{}").code, AppErrorCode::Auth);
        assert_eq!(map_error(429, "").code, AppErrorCode::RateLimited);
        assert_eq!(map_error(503, "").code, AppErrorCode::Network);
        assert_eq!(map_error(400, "").code, AppErrorCode::Internal);
    }

    #[test]
    fn error_diagnostic_never_contains_body() {
        let body = r#"{"error":{"code":401000,"message":"secret-text-from-server"}}"#;
        let e = map_error(401, body);
        assert!(!e.diagnostic.unwrap().contains("secret-text"));
    }
}
