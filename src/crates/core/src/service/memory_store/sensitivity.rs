/// Regex-based sensitivity gate for memory write operations.
///
/// Any text that matches one of the known secret patterns is rejected before
/// it can be persisted. On a hit the caller receives an error and a
/// `warn!` log entry; the matching fragment is **never** logged.
use std::sync::OnceLock;

use log::warn;
use regex::Regex;

use crate::util::errors::{BitFunError, BitFunResult};

fn secret_patterns() -> &'static Vec<Regex> {
    static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        let raw: &[&str] = &[
            // Generic key/secret/token/password assignment
            r#"(?i)(api[_-]?key|secret|password|passwd|token|auth_token|bearer)\s*[=:]\s*['"]?\S{8,}"#,
            // AWS-style access key id
            r"(?:AKIA|ASIA|AROA)[0-9A-Z]{16}",
            // PEM private key block
            r"-----BEGIN (RSA |EC |OPENSSH |PGP )?PRIVATE KEY-----",
            // .env assignment with a secret-looking value
            r"(?m)^[A-Z_]{4,}[_]?(SECRET|KEY|TOKEN|PASS|PASSWORD)\s*=\s*\S{8,}",
            // JWT header.payload.signature pattern
            r"eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}",
        ];
        raw.iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect()
    })
}

/// Check `text` for secret-like patterns.
///
/// Returns `Ok(())` when clean, or `Err` when a pattern matches.
/// The error message describes *why* without including the fragment.
pub fn check_for_secrets(text: &str) -> BitFunResult<()> {
    for pattern in secret_patterns() {
        if pattern.is_match(text) {
            warn!("Memory write blocked by sensitivity gate: reason=secret_pattern_matched");
            return Err(BitFunError::validation(
                "Sensitive content detected; this value cannot be stored in memory",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_api_key_assignment() {
        assert!(check_for_secrets("api_key = sk-abcdef1234567890ab").is_err());
    }

    #[test]
    fn blocks_pem_header() {
        assert!(check_for_secrets("-----BEGIN RSA PRIVATE KEY-----").is_err());
    }

    #[test]
    fn allows_normal_text() {
        assert!(check_for_secrets("The user prefers dark mode and short responses.").is_ok());
    }
}
