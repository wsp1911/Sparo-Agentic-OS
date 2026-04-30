/// CRUD and control operations for the memory store.
///
/// All write paths run through the sensitivity gate in `sensitivity.rs` before
/// touching the filesystem. Archive operations move entries to the `archive/`
/// sub-directory rather than deleting them, preserving history.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

use chrono;
use log::{debug, info, warn};

use super::{
    layout::ARCHIVE_DIR,
    paths::memory_store_dir_path_for_target,
    repair,
    schema::{parse_entry, render_entry, MemorySensitivity, MemoryStatus},
    sensitivity::check_for_secrets,
    MemoryStoreTarget,
};

use crate::util::errors::{BitFunError, BitFunResult};

// ---------------------------------------------------------------------------
// DTO types (serialisable, used by api-layer and Tauri commands)
// ---------------------------------------------------------------------------

/// Which consolidation pass to trigger manually.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConsolidationKind {
    Mid,
    SlowGlobal,
    SlowProject,
}

/// Filter for `list_entries`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListEntriesFilter {
    /// Optional glob pattern to match against relative paths.
    pub path_glob: Option<String>,
    /// If true, include archived entries.
    pub include_archived: bool,
    /// Filter by tag (exact match).
    pub tag: Option<String>,
}

/// Lightweight summary of a memory entry (no body text).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryEntrySummary {
    pub relative_path: String,
    pub title: String,
    pub layer: String,
    pub status: String,
    pub sensitivity: String,
    pub tags: Vec<String>,
    pub source_session: Option<String>,
    pub updated_ms: Option<u64>,
}

/// Full entry including body text.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryEntryDetail {
    pub summary: MemoryEntrySummary,
    pub content: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn normalize_entry_relative_path(relative_path: &str) -> BitFunResult<String> {
    let normalized = relative_path.replace('\\', "/");
    if normalized.trim().is_empty() {
        return Err(BitFunError::validation(
            "memory relative path cannot be empty",
        ));
    }
    if normalized.starts_with('/') || normalized.starts_with("~/") || normalized.contains("://") {
        return Err(BitFunError::validation(
            "memory relative path must stay inside the memory store",
        ));
    }

    let mut parts = Vec::new();
    for part in normalized.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." || part.contains('\0') || part.contains(':') {
            return Err(BitFunError::validation(
                "memory relative path must stay inside the memory store",
            ));
        }
        parts.push(part);
    }

    if parts.is_empty() {
        return Err(BitFunError::validation(
            "memory relative path cannot be empty",
        ));
    }

    Ok(parts.join("/"))
}

fn resolve_entry_path(
    target: MemoryStoreTarget<'_>,
    relative_path: &str,
) -> BitFunResult<(PathBuf, String)> {
    let normalized = normalize_entry_relative_path(relative_path)?;
    Ok((
        memory_store_dir_path_for_target(target).join(&normalized),
        normalized,
    ))
}

fn first_heading(body: &str) -> String {
    for line in body.lines() {
        let trimmed = line.trim_start_matches('#').trim();
        if !trimmed.is_empty() {
            return trimmed.to_owned();
        }
    }
    String::new()
}

async fn read_entry_detail(
    full_path: &std::path::Path,
    relative_path: &str,
) -> BitFunResult<MemoryEntryDetail> {
    let raw = fs::read_to_string(full_path)
        .await
        .map_err(|e| BitFunError::io(format!("read memory entry: {e}")))?;

    let parsed = parse_entry(&raw);
    let fm = &parsed.front_matter;

    let title = fm
        .id
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| first_heading(&parsed.body));

    let updated_ms = fs::metadata(full_path)
        .await
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64);

    Ok(MemoryEntryDetail {
        summary: MemoryEntrySummary {
            relative_path: relative_path.to_owned(),
            title,
            layer: fm.layer.as_str().to_owned(),
            status: fm.status.as_str().to_owned(),
            sensitivity: fm.sensitivity.as_str().to_owned(),
            tags: fm.tags.clone(),
            source_session: fm.source_session.clone(),
            updated_ms,
        },
        content: raw,
    })
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// List all memory entries for `target`, applying `filter`.
pub async fn list_entries(
    target: MemoryStoreTarget<'_>,
    filter: ListEntriesFilter,
) -> BitFunResult<Vec<MemoryEntrySummary>> {
    let memory_dir = memory_store_dir_path_for_target(target);

    let mut entries: Vec<MemoryEntrySummary> = Vec::new();

    let mut stack = vec![memory_dir.clone()];
    while let Some(dir) = stack.pop() {
        let mut read_dir = match fs::read_dir(&dir).await {
            Ok(rd) => rd,
            Err(_) => continue,
        };

        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if path.is_dir() {
                // Skip archive unless explicitly requested
                if name == ARCHIVE_DIR && !filter.include_archived {
                    continue;
                }
                stack.push(path);
                continue;
            }

            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            let relative_path = path
                .strip_prefix(&memory_dir)
                .ok()
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();

            // Path glob filter
            if let Some(ref glob) = filter.path_glob {
                if !relative_path.contains(glob.as_str()) {
                    continue;
                }
            }

            let raw = match fs::read_to_string(&path).await {
                Ok(r) => r,
                Err(_) => continue,
            };
            let parsed = parse_entry(&raw);
            let fm = &parsed.front_matter;

            // Skip archived if not requested
            if !filter.include_archived && fm.status == MemoryStatus::Archived {
                continue;
            }

            // Tag filter
            if let Some(ref tag) = filter.tag {
                if !fm.tags.iter().any(|t| t == tag) {
                    continue;
                }
            }

            let title = fm
                .id
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| first_heading(&parsed.body));

            let updated_ms = fs::metadata(&path)
                .await
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64);

            entries.push(MemoryEntrySummary {
                relative_path,
                title,
                layer: fm.layer.as_str().to_owned(),
                status: fm.status.as_str().to_owned(),
                sensitivity: fm.sensitivity.as_str().to_owned(),
                tags: fm.tags.clone(),
                source_session: fm.source_session.clone(),
                updated_ms,
            });
        }
    }

    entries.sort_by(|a, b| b.updated_ms.cmp(&a.updated_ms));
    Ok(entries)
}

/// Read a single memory entry by relative path.
pub async fn read_entry(
    target: MemoryStoreTarget<'_>,
    relative_path: &str,
) -> BitFunResult<MemoryEntryDetail> {
    let (full_path, normalized_relative_path) = resolve_entry_path(target, relative_path)?;
    read_entry_detail(&full_path, &normalized_relative_path).await
}

/// Update the content of an existing entry.  Runs the sensitivity gate before writing.
pub async fn update_entry(
    target: MemoryStoreTarget<'_>,
    relative_path: &str,
    content: &str,
) -> BitFunResult<()> {
    check_for_secrets(content)?;
    let (full_path, normalized_relative_path) = resolve_entry_path(target, relative_path)?;
    fs::write(&full_path, content)
        .await
        .map_err(|e| BitFunError::io(format!("write memory entry: {e}")))?;
    debug!(
        "Memory entry updated: relative_path={}",
        normalized_relative_path
    );
    Ok(())
}

/// Move an entry to `archive/` and update its `status` frontmatter field.
pub async fn archive_entry(target: MemoryStoreTarget<'_>, relative_path: &str) -> BitFunResult<()> {
    let (full_path, normalized_relative_path) = resolve_entry_path(target, relative_path)?;
    let raw = fs::read_to_string(&full_path)
        .await
        .map_err(|e| BitFunError::io(format!("read memory entry for archive: {e}")))?;

    let mut parsed = parse_entry(&raw);
    parsed.front_matter.status = MemoryStatus::Archived;

    let updated = render_entry(&parsed);
    if normalized_relative_path.starts_with("archive/") {
        fs::write(&full_path, updated)
            .await
            .map_err(|e| BitFunError::io(format!("write archived entry: {e}")))?;
    } else {
        let archive_dir = memory_store_dir_path_for_target(target).join(ARCHIVE_DIR);
        fs::create_dir_all(&archive_dir)
            .await
            .map_err(|e| BitFunError::io(format!("create memory archive dir: {e}")))?;

        let flat_name = normalized_relative_path.replace('/', "_");
        let mut archive_path = archive_dir.join(&flat_name);
        if archive_path.exists() {
            let stem = flat_name.trim_end_matches(".md");
            archive_path = archive_dir.join(format!(
                "{}-{}.md",
                stem,
                chrono::Utc::now().timestamp_millis()
            ));
        }

        fs::write(&archive_path, updated)
            .await
            .map_err(|e| BitFunError::io(format!("write archived entry: {e}")))?;
        fs::remove_file(&full_path).await.map_err(|e| {
            BitFunError::io(format!("remove original memory entry after archive: {e}"))
        })?;
    }

    info!(
        "Memory entry archived: relative_path={}",
        normalized_relative_path
    );
    Ok(())
}

/// Hard-delete a memory entry from disk.
pub async fn delete_entry(target: MemoryStoreTarget<'_>, relative_path: &str) -> BitFunResult<()> {
    let (full_path, normalized_relative_path) = resolve_entry_path(target, relative_path)?;
    fs::remove_file(&full_path)
        .await
        .map_err(|e| BitFunError::io(format!("delete memory entry: {e}")))?;
    info!(
        "Memory entry deleted: relative_path={}",
        normalized_relative_path
    );
    Ok(())
}

/// Record a memory hit: bump `last_seen` to now.
///
/// Call this whenever a memory entry is explicitly read or referenced during
/// a session so the lifecycle pass treats it as freshly used and defers
/// auto-archive.
pub async fn record_memory_hit(
    target: MemoryStoreTarget<'_>,
    relative_path: &str,
) -> BitFunResult<()> {
    let (full_path, normalized_relative_path) = resolve_entry_path(target, relative_path)?;
    let raw = match fs::read_to_string(&full_path).await {
        Ok(r) => r,
        Err(e) => {
            // Non-fatal: the file may have been removed or the path is wrong.
            debug!(
                "record_memory_hit: could not read entry: relative_path={} error={}",
                normalized_relative_path, e
            );
            return Ok(());
        }
    };

    if !raw.trim_start().starts_with("---") {
        debug!(
            "record_memory_hit: skipped file without front matter: relative_path={}",
            normalized_relative_path
        );
        return Ok(());
    }

    let mut parsed = parse_entry(&raw);

    if matches!(parsed.front_matter.status, MemoryStatus::Archived) {
        debug!(
            "record_memory_hit: skipped archived entry: relative_path={}",
            normalized_relative_path
        );
        return Ok(());
    }

    if matches!(parsed.front_matter.sensitivity, MemorySensitivity::Secret) {
        debug!(
            "record_memory_hit: skipped secret entry: relative_path={}",
            normalized_relative_path
        );
        return Ok(());
    }

    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    parsed.front_matter.last_seen = Some(now);

    let updated = render_entry(&parsed);
    fs::write(&full_path, updated)
        .await
        .map_err(|e| BitFunError::io(format!("record_memory_hit write: {e}")))?;

    debug!(
        "Memory hit recorded: relative_path={}",
        normalized_relative_path
    );

    Ok(())
}

/// Rebuild the `MEMORY.md` index for `target` from the current file state.
///
/// Preserves user-authored "Active Topics" and "Open Loops" sections.
pub async fn rebuild_index(target: MemoryStoreTarget<'_>) -> BitFunResult<()> {
    repair::rebuild_memory_index(target).await
}

/// Run the repair pass for `target`: fill empty templates, rebucket episodes,
/// write bootstrap marker.  Returns a summary of what changed.
pub async fn run_repair(target: MemoryStoreTarget<'_>) -> repair::RepairReport {
    repair::repair_memory_store(target).await
}

/// Archive all entries that carry `tag`.
pub async fn forget_by_tag(target: MemoryStoreTarget<'_>, tag: &str) -> BitFunResult<usize> {
    let filter = ListEntriesFilter {
        tag: Some(tag.to_owned()),
        include_archived: false,
        ..Default::default()
    };
    let entries = list_entries(target, filter).await?;
    let count = entries.len();
    for entry in &entries {
        if let Err(e) = archive_entry(target, &entry.relative_path).await {
            warn!(
                "Failed to archive entry during forget_by_tag: relative_path={} error={}",
                entry.relative_path, e
            );
        }
    }
    info!("forget_by_tag completed: tag={} count={}", tag, count);
    Ok(count)
}
