//! Deterministic consolidation executor.
//!
//! Runs before each mid-pass fork agent so that purely mechanical operations
//! (age-based archive moves, index rebuild) happen in Rust rather than being
//! delegated to the fork's prompt instructions. After this executor completes,
//! the mid-pass fork is given a summary of what was done so it can focus
//! exclusively on semantic tasks: clustering, conflict detection, abstraction
//! proposals, and narrative updates.

use super::decay::{is_archive_exempt, should_archive_by_age, LifecycleConfig};
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
pub struct LifecyclePassSummary {
    pub total_scanned: usize,
    pub archived: usize,
    pub errors: usize,
}

impl LifecyclePassSummary {
    /// Render a short human-readable summary suitable for inclusion in the
    /// mid-pass fork prompt so the agent knows what has already been done.
    pub fn to_prompt_note(&self) -> String {
        DETERMINISTIC_PRE_PASS_NOTE
            .replace("{total_scanned}", &self.total_scanned.to_string())
            .replace("{archived}", &self.archived.to_string())
            .replace("{errors}", &self.errors.to_string())
    }
}

/// Run the deterministic lifecycle pass for the given target.
///
/// For each entry in the memory store (excluding archive, identity, narrative,
/// pinned):
/// 1. Read `last_seen` (or `created`) from front matter.
/// 2. If the entry is older than `archive_after_days`, move it to `archive/`
///    and set `status: archived`.
///
/// After the pass, rebuilds the `MEMORY.md` index.
pub async fn run_lifecycle_pass(
    target: MemoryStoreTarget<'_>,
    config: &LifecycleConfig,
) -> BitFunResult<LifecyclePassSummary> {
    let memory_dir = memory_store_dir_path_for_target(target);
    if !memory_dir.exists() {
        return Ok(LifecyclePassSummary::default());
    }

    let all_files = list_memory_files_recursive(&memory_dir).await?;
    let mut summary = LifecyclePassSummary::default();
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
                debug!(
                    "Lifecycle pass: failed to read file: path={} error={}",
                    rel, e
                );
                summary.errors += 1;
                continue;
            }
        };

        let mut parsed = parse_entry(&raw);
        let layer = &parsed.front_matter.layer;
        let status = &parsed.front_matter.status;

        if is_archive_exempt(layer, status, &rel) {
            continue;
        }

        let elapsed_days = compute_elapsed_days(
            parsed
                .front_matter
                .last_seen
                .as_ref()
                .or(parsed.front_matter.created.as_ref()),
            &now,
        );

        if !should_archive_by_age(elapsed_days, config) {
            continue;
        }

        match archive_file(path, &memory_dir, &mut parsed, &rel).await {
            Ok(_) => {
                summary.archived += 1;
                info!(
                    "Lifecycle pass: archived stale entry: rel={} age_days={:.1}",
                    rel, elapsed_days
                );
            }
            Err(e) => {
                warn!(
                    "Lifecycle pass: failed to archive entry: rel={} error={}",
                    rel, e
                );
                summary.errors += 1;
            }
        }
    }

    if let Err(e) = rebuild_index(target).await {
        warn!("Lifecycle pass: index rebuild failed: error={}", e);
    }

    info!(
        "Lifecycle pass complete: scope={} scanned={} archived={} errors={}",
        target.scope().as_label(),
        summary.total_scanned,
        summary.archived,
        summary.errors
    );

    Ok(summary)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn compute_elapsed_days(anchor: Option<&String>, now: &DateTime<Utc>) -> f32 {
    let Some(anchor_str) = anchor else {
        return 0.0;
    };
    let Ok(anchor_dt) = DateTime::parse_from_rfc3339(anchor_str) else {
        return 0.0;
    };
    let elapsed = *now - anchor_dt.with_timezone(&Utc);
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
