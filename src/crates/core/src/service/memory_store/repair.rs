//! Memory store repair and index rebuild utilities.
//!
//! This module provides two kinds of operations:
//!
//! - **Health check** — non-destructive scan that returns a list of issues.
//! - **Repair** — idempotent fixes: create missing files/dirs, fill empty
//!   templates, re-bucket mis-placed episodes.
//! - **Index rebuild** — regenerate `MEMORY.md` from the current state of the
//!   memory directory, preserving any human-authored "Active Topics" and
//!   "Open Loops" sections that already exist.

use super::{
    ensure_memory_skeleton, format_manifest_path,
    layout::{
        BOOTSTRAP_V2_MARKER, EPISODES_DIR, GLOBAL_IDENTITY_TEMPLATE, GLOBAL_MEMORY_INDEX_TEMPLATE,
        HABITS_TEMPLATE, NARRATIVE_TEMPLATE, PERSONA_TEMPLATE, PROJECT_IDENTITY_TEMPLATE,
        PROJECT_MEMORY_INDEX_TEMPLATE, PROJECT_TEMPLATE,
    },
    list_memory_files_recursive, memory_store_dir_path_for_target, MemoryScope, MemoryStoreTarget,
    MEMORY_INDEX_FILE,
};
use crate::util::errors::*;
use log::{debug, info, warn};
use std::path::{Path, PathBuf};
use tokio::fs;

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Rebuild the `MEMORY.md` index file for a memory store target.
///
/// Reads all non-archive memory files, groups them by category, and writes
/// a structured navigation document.  Any "Active Topics" or "Open Loops"
/// sections already present in the existing `MEMORY.md` are preserved.
pub(crate) async fn rebuild_memory_index(target: MemoryStoreTarget<'_>) -> BitFunResult<()> {
    let memory_dir = memory_store_dir_path_for_target(target);
    if !memory_dir.exists() {
        return Err(BitFunError::service(format!(
            "Memory directory does not exist: {}",
            memory_dir.display()
        )));
    }

    let index_path = memory_dir.join(MEMORY_INDEX_FILE);

    // Preserve user-authored sections from the existing index.
    let preserved = extract_preserved_sections(&index_path).await;

    // Collect active (non-archive) files.
    let all_files = list_memory_files_recursive(&memory_dir).await?;

    let rendered = render_rebuilt_index(target.scope(), &memory_dir, &all_files, &preserved);

    fs::write(&index_path, rendered)
        .await
        .map_err(|e| BitFunError::io(format!("Failed to write rebuilt MEMORY.md: {e}")))?;

    info!(
        "Memory index rebuilt: scope={} memory_dir={}",
        target.scope().as_label(),
        memory_dir.display()
    );

    Ok(())
}

/// Repair a memory store: ensure skeleton, write templates to empty core
/// files, re-bucket mis-placed episodes, and report what was fixed.
///
/// This is idempotent and safe to call on every startup or on demand.
pub(crate) async fn repair_memory_store(target: MemoryStoreTarget<'_>) -> RepairReport {
    let memory_dir = memory_store_dir_path_for_target(target);
    let mut report = RepairReport::default();

    if !memory_dir.exists() {
        report
            .actions
            .push("Memory directory did not exist; nothing to repair.".to_string());
        return report;
    }

    // Ensure subdirectory skeleton.
    match ensure_memory_skeleton(target.scope(), &memory_dir).await {
        Ok(_) => {}
        Err(e) => {
            report
                .errors
                .push(format!("Failed to create skeleton: {e}"));
        }
    }

    // Write templates to empty core files.
    let wrote = write_templates_to_empty_core_files(target.scope(), &memory_dir).await;
    report.actions.extend(wrote);

    // Re-bucket mis-placed episodes (flat files directly in episodes/).
    let bucketed = rebucket_episodes(&memory_dir).await;
    report.actions.extend(bucketed);

    // Mark bootstrap v2 complete if not already done.
    let marker = memory_dir.join(BOOTSTRAP_V2_MARKER);
    if !marker.exists() {
        let _ = fs::write(&marker, "").await;
        report
            .actions
            .push("Wrote bootstrap v2 marker.".to_string());
    }

    debug!(
        "Memory store repair complete: scope={} actions={} errors={}",
        target.scope().as_label(),
        report.actions.len(),
        report.errors.len()
    );

    report
}

/// Health check: returns a list of problems without modifying anything.
pub(crate) async fn check_memory_health(target: MemoryStoreTarget<'_>) -> Vec<HealthIssue> {
    let memory_dir = memory_store_dir_path_for_target(target);
    let mut issues = Vec::new();

    if !memory_dir.exists() {
        issues.push(HealthIssue {
            kind: HealthIssueKind::MissingDirectory,
            path: memory_dir.to_string_lossy().to_string(),
        });
        return issues;
    }

    // Check index.
    let index_path = memory_dir.join(MEMORY_INDEX_FILE);
    let index_empty = is_path_empty_or_missing(&index_path).await;
    if index_empty {
        issues.push(HealthIssue {
            kind: HealthIssueKind::EmptyOrMissingIndex,
            path: "MEMORY.md".to_string(),
        });
    }

    // Check core files.
    let core_files: &[&str] = match target.scope() {
        MemoryScope::GlobalAgenticOs => &["identity.md", "narrative.md", "persona.md", "habits.md"],
        MemoryScope::WorkspaceProject => &["identity.md", "project.md", "habits.md"],
    };
    for &name in core_files {
        let path = memory_dir.join(name);
        if !path.exists() {
            issues.push(HealthIssue {
                kind: HealthIssueKind::MissingCoreFile,
                path: name.to_string(),
            });
        }
    }

    // Check for episodes not bucketed in YYYY-MM.
    let episodes_dir = memory_dir.join(EPISODES_DIR);
    if episodes_dir.exists() {
        if let Ok(mut entries) = fs::read_dir(&episodes_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let p = entry.path();
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if p.is_file() && name.ends_with(".md") {
                    issues.push(HealthIssue {
                        kind: HealthIssueKind::EpisodeNotBucketed,
                        path: format!("episodes/{name}"),
                    });
                }
            }
        }
    }

    issues
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct RepairReport {
    pub actions: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug)]
pub struct HealthIssue {
    pub kind: HealthIssueKind,
    pub path: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum HealthIssueKind {
    MissingDirectory,
    EmptyOrMissingIndex,
    MissingCoreFile,
    EpisodeNotBucketed,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Extract any "Active Topics" and "Open Loops" sections from an existing
/// MEMORY.md so they can be preserved in the rebuilt version.
async fn extract_preserved_sections(index_path: &Path) -> PreservedSections {
    let content = match fs::read_to_string(index_path).await {
        Ok(c) if !c.trim().is_empty() => c,
        _ => return PreservedSections::default(),
    };

    let mut active_topics: Vec<String> = Vec::new();
    let mut open_loops: Vec<String> = Vec::new();

    let mut current_section: Option<&str> = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## Active Topics") {
            current_section = Some("active_topics");
            continue;
        }
        if trimmed.starts_with("## Open Loops") {
            current_section = Some("open_loops");
            continue;
        }
        if trimmed.starts_with("## ") {
            current_section = None;
            continue;
        }
        match current_section {
            Some("active_topics") => {
                if !trimmed.is_empty() && !trimmed.starts_with("<!--") && !trimmed.ends_with("-->")
                {
                    active_topics.push(line.to_string());
                }
            }
            Some("open_loops") => {
                if !trimmed.is_empty() && !trimmed.starts_with("<!--") && !trimmed.ends_with("-->")
                {
                    open_loops.push(line.to_string());
                }
            }
            _ => {}
        }
    }

    PreservedSections {
        active_topics,
        open_loops,
    }
}

#[derive(Default)]
struct PreservedSections {
    active_topics: Vec<String>,
    open_loops: Vec<String>,
}

/// Build the rebuilt MEMORY.md content.
fn render_rebuilt_index(
    scope: MemoryScope,
    memory_dir: &Path,
    files: &[PathBuf],
    preserved: &PreservedSections,
) -> String {
    let mut core_files: Vec<String> = Vec::new();
    let mut pinned_files: Vec<String> = Vec::new();
    let mut recent_episodes: Vec<String> = Vec::new();
    let mut recent_sessions: Vec<String> = Vec::new();

    for path in files {
        let rel = format_manifest_path(path, memory_dir);
        if rel.starts_with("archive/") || rel.starts_with("workspaces_overview/") {
            continue;
        }
        if rel == MEMORY_INDEX_FILE {
            continue;
        }
        if rel.starts_with("pinned/") {
            pinned_files.push(rel);
        } else if rel.starts_with("episodes/") {
            recent_episodes.push(rel);
        } else if rel.starts_with("sessions/") {
            recent_sessions.push(rel);
        } else if rel.ends_with(".md") {
            core_files.push(rel);
        }
    }

    // Sort and limit.
    core_files.sort();
    pinned_files.sort();
    recent_episodes.sort();
    recent_episodes.reverse();
    recent_episodes.truncate(10);
    recent_sessions.sort();
    recent_sessions.reverse();
    recent_sessions.truncate(5);

    let header = match scope {
        MemoryScope::GlobalAgenticOs => "# Memory Index\n\nThis file is the navigation directory for the global Agentic OS memory store.\nRead this file first before searching for specific memories.",
        MemoryScope::WorkspaceProject => "# Memory Index\n\nThis file is the navigation directory for this workspace's memory store.\nRead this file first before searching for specific memories.",
    };

    let mut out = format!("{}\n\n## Map\n\n", header);
    for f in &core_files {
        out.push_str(&format!("- `{f}`\n"));
    }
    if !pinned_files.is_empty() {
        out.push_str("\n**Pinned:**\n\n");
        for f in &pinned_files {
            out.push_str(&format!("- `{f}`\n"));
        }
    }

    // Active Topics — preserve user content.
    out.push_str("\n## Active Topics\n\n");
    if preserved.active_topics.is_empty() {
        out.push_str("<!-- Populated by the assistant: topic → relevant files -->\n");
    } else {
        for line in &preserved.active_topics {
            out.push_str(line);
            out.push('\n');
        }
    }

    // Recent content section name differs by scope.
    match scope {
        MemoryScope::GlobalAgenticOs => {
            out.push_str("\n## Recent Sessions\n\n");
            if recent_sessions.is_empty() {
                out.push_str("<!-- Updated after each session summary is written -->\n");
            } else {
                for f in &recent_sessions {
                    out.push_str(&format!("- `{f}`\n"));
                }
            }
        }
        MemoryScope::WorkspaceProject => {
            out.push_str("\n## Recent Episodes\n\n");
            if recent_episodes.is_empty() {
                out.push_str(
                    "<!-- Notable events (auto-updated after episodic entries are created) -->\n",
                );
            } else {
                for f in &recent_episodes {
                    out.push_str(&format!("- `{f}`\n"));
                }
            }
        }
    }

    // Open Loops — preserve user content.
    out.push_str("\n## Open Loops\n\n");
    if preserved.open_loops.is_empty() {
        out.push_str("<!-- Unfinished items, commitments, or pending follow-ups -->\n");
    } else {
        for line in &preserved.open_loops {
            out.push_str(line);
            out.push('\n');
        }
    }

    out
}

/// Write templates to core files that currently exist but are empty.
async fn write_templates_to_empty_core_files(scope: MemoryScope, memory_dir: &Path) -> Vec<String> {
    let core_files: &[(&str, &str)] = match scope {
        MemoryScope::GlobalAgenticOs => &[
            ("identity.md", GLOBAL_IDENTITY_TEMPLATE),
            ("narrative.md", NARRATIVE_TEMPLATE),
            ("persona.md", PERSONA_TEMPLATE),
            ("habits.md", HABITS_TEMPLATE),
        ],
        MemoryScope::WorkspaceProject => &[
            ("identity.md", PROJECT_IDENTITY_TEMPLATE),
            ("project.md", PROJECT_TEMPLATE),
            ("habits.md", HABITS_TEMPLATE),
        ],
    };

    let mut actions = Vec::new();
    for &(name, template) in core_files {
        let path = memory_dir.join(name);
        if is_path_empty_or_missing(&path).await {
            match fs::write(&path, template).await {
                Ok(_) => actions.push(format!("Wrote template to empty file: {name}")),
                Err(e) => warn!("Repair: failed to write template to {name}: {e}"),
            }
        }
    }

    // Also fix MEMORY.md if empty.
    let index_path = memory_dir.join(MEMORY_INDEX_FILE);
    if is_path_empty_or_missing(&index_path).await {
        let template = match scope {
            MemoryScope::GlobalAgenticOs => GLOBAL_MEMORY_INDEX_TEMPLATE,
            MemoryScope::WorkspaceProject => PROJECT_MEMORY_INDEX_TEMPLATE,
        };
        match fs::write(&index_path, template).await {
            Ok(_) => actions.push("Wrote template to empty MEMORY.md".to_string()),
            Err(e) => warn!("Repair: failed to write MEMORY.md template: {e}"),
        }
    }

    actions
}

/// Move episode files that sit directly in `episodes/` (not in a YYYY-MM
/// subdirectory) into the correct month bucket.
async fn rebucket_episodes(memory_dir: &Path) -> Vec<String> {
    let episodes_dir = memory_dir.join(EPISODES_DIR);
    if !episodes_dir.exists() {
        return Vec::new();
    }

    let mut actions = Vec::new();
    let mut entries = match fs::read_dir(&episodes_dir).await {
        Ok(e) => e,
        Err(_) => return actions,
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !path.is_file() || !name.ends_with(".md") {
            continue;
        }

        // File name must start with YYYY-MM-DD for bucketing.
        if name.len() < 10 {
            continue;
        }
        let month = &name[..7]; // "YYYY-MM"
        if !is_year_month(month) {
            continue;
        }

        let bucket = episodes_dir.join(month);
        if let Err(e) = fs::create_dir_all(&bucket).await {
            warn!("Repair: failed to create episode bucket {month}: {e}");
            continue;
        }

        let dest = bucket.join(name);
        if dest.exists() {
            continue;
        }

        match fs::rename(&path, &dest).await {
            Ok(_) => actions.push(format!(
                "Bucketed episode: {name} → episodes/{month}/{name}"
            )),
            Err(e) => warn!("Repair: failed to move episode {name}: {e}"),
        }
    }

    actions
}

fn is_year_month(s: &str) -> bool {
    // Expects "YYYY-MM"
    let bytes = s.as_bytes();
    bytes.len() == 7
        && bytes[..4].iter().all(|b| b.is_ascii_digit())
        && bytes[4] == b'-'
        && bytes[5..].iter().all(|b| b.is_ascii_digit())
}

async fn is_path_empty_or_missing(path: &Path) -> bool {
    match fs::read_to_string(path).await {
        Ok(content) => content.trim().is_empty(),
        Err(_) => true,
    }
}
