//! Kiểm tra endpoint (threat model §9.3): chống gửi key nhầm host.
use std::net::IpAddr;

use lexora_core::{AppError, AppErrorCode};
use url::{Host, Url};

fn invalid(why: &str) -> AppError {
    AppError::with_key(AppErrorCode::InputInvalid, "endpoint_invalid").with_diagnostic(why)
}

fn is_loopback(host: &Host<&str>) -> bool {
    match host {
        Host::Domain(d) => d.eq_ignore_ascii_case("localhost"),
        Host::Ipv4(ip) => IpAddr::V4(*ip).is_loopback(),
        Host::Ipv6(ip) => IpAddr::V6(*ip).is_loopback(),
    }
}

/// HTTPS bắt buộc. HTTP chỉ khi `allow_loopback_http` và host là loopback
/// (preset Ollama/LM Studio sau MVP). Cấm userinfo, query, fragment.
pub fn validate_endpoint(raw: &str, allow_loopback_http: bool) -> Result<Url, AppError> {
    let url = Url::parse(raw).map_err(|_| invalid("unparsable"))?;
    if !url.username().is_empty() || url.password().is_some() {
        return Err(invalid("userinfo not allowed"));
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err(invalid("query/fragment not allowed"));
    }
    let scheme = url.scheme().to_owned();
    let loopback = match url.host() {
        Some(h) => is_loopback(&h),
        None => return Err(invalid("no host")),
    };
    match scheme.as_str() {
        "https" => Ok(url),
        "http" if allow_loopback_http && loopback => Ok(url),
        _ => Err(invalid("scheme not allowed")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_https() {
        assert!(validate_endpoint("https://api.cognitive.microsofttranslator.com", false).is_ok());
    }

    #[test]
    fn rejects_remote_http_even_if_loopback_allowed() {
        assert!(validate_endpoint("http://example.com", true).is_err());
        assert!(validate_endpoint("http://example.com", false).is_err());
    }

    #[test]
    fn loopback_http_only_when_allowed() {
        assert!(validate_endpoint("http://127.0.0.1:11434", true).is_ok());
        assert!(validate_endpoint("http://localhost:1234", true).is_ok());
        assert!(validate_endpoint("http://[::1]:1234", true).is_ok());
        assert!(validate_endpoint("http://127.0.0.1:11434", false).is_err());
    }

    #[test]
    fn rejects_userinfo_query_fragment() {
        assert!(validate_endpoint("https://user:pw@host.example", false).is_err());
        assert!(validate_endpoint("https://host.example/?key=abc", false).is_err());
        assert!(validate_endpoint("https://host.example/#x", false).is_err());
    }
}
