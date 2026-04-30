//! Strength decay and lifecycle utilities for memory entries.
//!
//! Decay is computed lazily — each mid-pass reads the entry's
//! `last_seen` and `strength` frontmatter fields and applies the
//! exponential half-life formula:
//!
//!   new_strength = current_strength × 0.5 ^ (elapsed_days / half_life_days)
//!
//! Entries below `archive_threshold` transition to `status: archived`.
//!
//! Entries that must NOT decay:
//!   - `layer: identity`
//!   - `layer: narrative`
//!   - `layer: pinned`
//!   - `status: archived` (already done)
//!   - Any entry in the `pinned/` directory

use crate::service::memory_store::schema::{MemoryLayer, MemoryStatus};

/// Configuration for the decay curve.
#[derive(Debug, Clone)]
pub struct DecayConfig {
    /// Number of days for strength to halve.
    pub half_life_days: f32,
    /// Strength below which an entry should be archived.
    pub archive_threshold: f32,
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self {
            half_life_days: 21.0,
            archive_threshold: 0.15,
        }
    }
}

impl DecayConfig {
    /// Load from environment variable overrides (if set), falling back to defaults.
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(val) = std::env::var("SPARO_MEMORY_HALF_LIFE_DAYS") {
            if let Ok(f) = val.parse::<f32>() {
                cfg.half_life_days = f;
            }
        }
        if let Ok(val) = std::env::var("SPARO_MEMORY_ARCHIVE_THRESHOLD") {
            if let Ok(f) = val.parse::<f32>() {
                cfg.archive_threshold = f;
            }
        }
        cfg
    }
}

/// Compute the decayed strength after `elapsed_days` have passed.
pub fn decayed_strength(current_strength: f32, elapsed_days: f32, config: &DecayConfig) -> f32 {
    let new_strength = current_strength * 0.5_f32.powf(elapsed_days / config.half_life_days);
    new_strength.clamp(0.0, 1.0)
}

/// Returns `true` if an entry with the given layer and status should be
/// protected from decay.
pub fn is_decay_exempt(layer: &MemoryLayer, status: &MemoryStatus, relative_path: &str) -> bool {
    if matches!(status, MemoryStatus::Archived) {
        return true; // already archived
    }
    if matches!(
        layer,
        MemoryLayer::Identity | MemoryLayer::Narrative | MemoryLayer::Pinned
    ) {
        return true;
    }
    // Anything in the pinned/ directory is exempt regardless of frontmatter.
    let normalized = relative_path.replace('\\', "/");
    if normalized.starts_with("pinned/") {
        return true;
    }
    false
}

/// Returns `true` if the decayed strength falls below the archive threshold.
pub fn should_archive(strength: f32, config: &DecayConfig) -> bool {
    strength < config.archive_threshold
}

/// Apply a "hit" boost to strength (called when an entry is read/referenced).
///
/// The boost is additive but clamped to 1.0.
pub fn apply_hit(current_strength: f32, boost: f32) -> f32 {
    (current_strength + boost).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_decay_at_zero_days() {
        let cfg = DecayConfig::default();
        let strength = decayed_strength(0.8, 0.0, &cfg);
        assert!((strength - 0.8).abs() < 1e-5);
    }

    #[test]
    fn half_strength_at_half_life() {
        let cfg = DecayConfig::default(); // 21 days
        let strength = decayed_strength(1.0, 21.0, &cfg);
        assert!(
            (strength - 0.5).abs() < 1e-4,
            "expected 0.5, got {}",
            strength
        );
    }

    #[test]
    fn identity_layer_is_exempt() {
        assert!(is_decay_exempt(
            &MemoryLayer::Identity,
            &MemoryStatus::Confirmed,
            "identity.md"
        ));
    }

    #[test]
    fn pinned_dir_is_exempt() {
        assert!(is_decay_exempt(
            &MemoryLayer::Reference,
            &MemoryStatus::Confirmed,
            "pinned/linear-ingest.md"
        ));
    }

    #[test]
    fn archived_status_is_exempt() {
        assert!(is_decay_exempt(
            &MemoryLayer::Episodic,
            &MemoryStatus::Archived,
            "episodes/2026-03/2026-03-15-foo.md"
        ));
    }

    #[test]
    fn below_threshold_should_archive() {
        let cfg = DecayConfig::default();
        assert!(should_archive(0.10, &cfg));
        assert!(!should_archive(0.20, &cfg));
    }
}
