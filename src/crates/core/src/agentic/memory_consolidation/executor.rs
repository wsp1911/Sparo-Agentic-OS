//! Deterministic consolidation executor.
//!
//! Runs before each mid-pass fork agent so that purely mechanical operations
//! (strength decay, archive-threshold moves, index rebuild, episode
//! re-bucketing) happen in Rust rather than being delegated to the fork's
//! prompt instructions.  This makes consolidation more reliable and reduces
//! the prompt complexity the fork agent must handle.
//!
//! After this executor completes, the mid-pass fork is given a summary of
//! what was done so it can focus exclusively on semantic tasks: clustering,
//! conflict detection, abstraction proposals, and narrative updates.

use super::decay::{decayed_strength, is_decay_exempt, should_archive, DecayConfig};
use crate::service::memory_store::{
    api::rebuild_index,
    format_manifest_path, list_memory_files_recursive, memory_store_dir_path_for_target,
    schema::{parse_entry, render_entry, MemoryStatus},
    MemoryStoreTarget, MEMORY_INDEX_FILE,
};
use crate::util::errors::BitFunResult;
use chrono::{DateTime, Utc};
use log::{debug, info, warn};
use std::path::Path;
use tokio::fs;

const DETERMINISTIC_PRE_PASS_NOTE: &str =
    include_str!("../../service/memory_store/prompts/agent_memory_deterministic_pre_pass_note.md");

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Summary of what the deterministic pass did.
#[derive(Debug, Default)]
pub struct DecayPassSummary {
    pub total_scanned: usize,
    pub decayed: usize,
    pub archived: usize,
    pub errors: usize,
}

impl DecayPassSummary {
    /// Render a short human-readable summary suitable for inclusion in the
    /// mid-pass fork prompt so the agent knows what has already been done.
    pub fn to_prompt_note(&self) -> String {
        DETERMINISTIC_PRE_PASS_NOTE
            .replace("{total_scanned}", &self.total_scanned.to_string())
            .replace("{decayed}", &self.decayed.to_string())
            .replace("{archived}", &self.archived.to_string())
            .replace("{errors}", &self.errors.to_string())
    }
}

/// Run the deterministic decay pass for the given target.
///
/// For each entry in the memory store (excluding archive, identity, narrative,
/// pinned):
/// 1. Read `strength` and `last_seen` from front matter.
/// 2. Compute decayed strength.
/// 3. If strength changed, write the updated entry back.
/// 4. If the decayed strength is below `archive_threshold`, move the file to
///    the `archive/` directory and set `status: archived`.
///
/// After the pass, rebuilds the `MEMORY.md` index.
pub async fn run_decay_pass(
    target: MemoryStoreTarget<'_>,
    config: &DecayConfig,
) -> BitFunResult<DecayPassSummary> {
    let memory_dir = memory_store_dir_path_for_target(target);
    if !memory_dir.exists() {
        return Ok(DecayPassSummary::default());
    }

    let all_files = list_memory_files_recursive(&memory_dir).await?;
    let mut summary = DecayPassSummary::default();
    let now: DateTime<Utc> = Utc::now();

    for path in &all_files {
        let rel = format_manifest_path(path, &memory_dir);

        // Skip the index file and archived directory.
        if rel == MEMORY_INDEX_FILE || rel.starts_with("archive/") {
            continue;
        }

        summary.total_scanned += 1;

        let raw = match fs::read_to_string(path).await {
            Ok(r) => r,
            Err(e) => {
                debug!("Decay pass: failed to read file: path={} error={}", rel, e);
                summary.errors += 1;
                continue;
            }
        };

        let mut parsed = parse_entry(&raw);
        let layer = &parsed.front_matter.layer;
        let status = &parsed.front_matter.status;

        if is_decay_exempt(layer, status, &rel) {
            continue;
        }

        let current_strength = parsed.front_matter.strength.unwrap_or(1.0);
        let elapsed_days = compute_elapsed_days(&parsed.front_matter.last_seen, &now);
        let new_strength = decayed_strength(current_strength, elapsed_days, config);

        if (new_strength - current_strength).abs() < 1e-4 {
            continue; // no meaningful change
        }

        parsed.front_matter.strength = Some(new_strength);
        summary.decayed += 1;

        if should_archive(new_strength, config) {
            // Move to archive/.
            match archive_file(path, &memory_dir, &mut parsed, &rel).await {
                Ok(_) => {
                    summary.archived += 1;
                    info!(
                        "Decay pass: archived low-strength entry: rel={} strength={:.3}",
                        rel, new_strength
                    );
                }
                Err(e) => {
                    warn!(
                        "Decay pass: failed to archive entry: rel={} error={}",
                        rel, e
                    );
                    summary.errors += 1;
                    // Still write back the updated strength even if archive failed.
                    let _ = write_updated_entry(path, &parsed).await;
                }
            }
        } else {
            if let Err(e) = write_updated_entry(path, &parsed).await {
                debug!(
                    "Decay pass: failed to write updated entry: rel={} error={}",
                    rel, e
                );
                summary.errors += 1;
            }
        }
    }

    // Rebuild the index after the pass so the fork has an accurate picture.
    if let Err(e) = rebuild_index(target).await {
        warn!("Decay pass: index rebuild failed: error={}", e);
    }

    info!(
        "Decay pass complete: scope={} scanned={} decayed={} archived={} errors={}",
        target.scope().as_label(),
        summary.total_scanned,
        summary.decayed,
        summary.archived,
        summary.errors
    );

    Ok(summary)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn compute_elapsed_days(last_seen: &Option<String>, now: &DateTime<Utc>) -> f32 {
    let Some(last_seen_str) = last_seen else {
        return 0.0;
    };
    let Ok(last_seen_dt) = DateTime::parse_from_rfc3339(last_seen_str) else {
        return 0.0;
    };
    let elapsed = *now - last_seen_dt.with_timezone(&Utc);
    elapsed.num_seconds().max(0) as f32 / 86_400.0
}

async fn archive_file(
    path: &Path,
    memory_dir: &Path,
    parsed: &mut crate::service::memory_store::schema::ParsedMemoryEntry,
    rel: &str,
) -> BitFunResult<()> {
    use crate::util::errors::BitFunError;

    let archive_dir = memory_dir.join("archive");
    fs::create_dir_all(&archive_dir)
        .await
        .map_err(|e| BitFunError::io(format!("Failed to create archive dir: {e}")))?;

    // Flatten the path inside archive/: replace / with _ so episodes/2026-01/foo.md
    // becomes archive/episodes_2026-01_foo.md.
    let flat_name = rel.replace('/', "_");
    let dest = archive_dir.join(&flat_name);

    parsed.front_matter.status = MemoryStatus::Archived;
    let content = render_entry(parsed);

    fs::write(&dest, &content)
        .await
        .map_err(|e| BitFunError::io(format!("Failed to write archive file: {e}")))?;
    fs::remove_file(path)
        .await
        .map_err(|e| BitFunError::io(format!("Failed to remove original after archive: {e}")))?;

    Ok(())
}

async fn write_updated_entry(
    path: &Path,
    parsed: &crate::service::memory_store::schema::ParsedMemoryEntry,
) -> BitFunResult<()> {
    let content = render_entry(parsed);
    fs::write(path, content)
        .await
        .map_err(|e| crate::util::errors::BitFunError::io(format!("Failed to write entry: {e}")))
}
