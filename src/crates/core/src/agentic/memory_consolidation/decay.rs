//! Lifecycle utilities for memory entries.
//!
//! Strength was removed in favor of age-based archiving. The mid-pass reads
//! `last_seen` (falling back to `created`) and archives entries older than
//! `archive_after_days` unless they are exempt.
//!
//! Entries that must NOT auto-archive:
//!   - `layer: identity`
//!   - `layer: narrative`
//!   - `layer: pinned`
//!   - `status: archived` (already done)
//!   - Any entry in the `pinned/` directory

use crate::service::memory_store::schema::{MemoryLayer, MemoryStatus};

/// Configuration for the lifecycle pass.
#[derive(Debug, Clone)]
pub struct LifecycleConfig {
    /// Entries older than this many days (by `last_seen`, fallback to `created`)
    /// are moved to `archive/` unless exempt.
    pub archive_after_days: f32,
}

impl Default for LifecycleConfig {
    fn default() -> Self {
        Self {
            archive_after_days: 90.0,
        }
    }
}

impl LifecycleConfig {
    /// Load from environment variable overrides (if set), falling back to defaults.
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(val) = std::env::var("SPARO_MEMORY_ARCHIVE_AFTER_DAYS") {
            if let Ok(f) = val.parse::<f32>() {
                cfg.archive_after_days = f;
            }
        }
        cfg
    }
}

/// Returns `true` if an entry with the given layer/status/path is protected
/// from automatic archiving.
pub fn is_archive_exempt(layer: &MemoryLayer, status: &MemoryStatus, relative_path: &str) -> bool {
    if matches!(status, MemoryStatus::Archived) {
        return true;
    }
    if matches!(
        layer,
        MemoryLayer::Identity | MemoryLayer::Narrative | MemoryLayer::Pinned
    ) {
        return true;
    }
    let normalized = relative_path.replace('\\', "/");
    if normalized.starts_with("pinned/") {
        return true;
    }
    false
}

/// Returns `true` if an entry whose freshness anchor is `elapsed_days` ago
/// should be archived.
pub fn should_archive_by_age(elapsed_days: f32, config: &LifecycleConfig) -> bool {
    elapsed_days >= config.archive_after_days
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_entries_are_not_archived() {
        let cfg = LifecycleConfig::default();
        assert!(!should_archive_by_age(0.0, &cfg));
        assert!(!should_archive_by_age(30.0, &cfg));
    }

    #[test]
    fn entries_past_threshold_are_archived() {
        let cfg = LifecycleConfig::default();
        assert!(should_archive_by_age(91.0, &cfg));
    }

    #[test]
    fn identity_layer_is_exempt() {
        assert!(is_archive_exempt(
            &MemoryLayer::Identity,
            &MemoryStatus::Confirmed,
            "identity.md"
        ));
    }

    #[test]
    fn pinned_dir_is_exempt() {
        assert!(is_archive_exempt(
            &MemoryLayer::Reference,
            &MemoryStatus::Confirmed,
            "pinned/linear-ingest.md"
        ));
    }

    #[test]
    fn archived_status_is_exempt() {
        assert!(is_archive_exempt(
            &MemoryLayer::Episodic,
            &MemoryStatus::Archived,
            "episodes/2026-03/2026-03-15-foo.md"
        ));
    }
}
