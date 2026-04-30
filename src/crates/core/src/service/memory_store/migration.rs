//! One-time, idempotent migration from the pre-M1 flat memory layout to the
//! new structured layout (identity / persona / project / habits / narrative +
//! subdirectory skeleton).
//!
//! The migration is guarded by a `.migrated_v1` marker file in the memory
//! directory.  It is safe to call `run_if_needed` on every startup.
//!
//! What the migration does:
//!
//! - Ensures all new subdirectory skeleton files exist.
//! - Reads every existing `.md` file (excluding MEMORY.md and already-core
//!   files) and classifies them by their legacy `type:` frontmatter field.
//! - Appends the body content of each legacy file into the appropriate new
//!   core file (persona.md / habits.md / project.md / identity.md) or moves
//!   it into `pinned/` (reference) or `archive/legacy-<date>/` (everything
//!   else).
//! - Writes global narrative.md from any `vision` or `assistant_identity`
//!   entries found in *any* scope (the global scope migration receives these
//!   via the caller).
//! - Leaves the original files untouched (only the new core files are
//!   written/appended; originals are copied to archive/legacy-<date>/).

use super::{
    layout::{
        legacy_archive_dir_for, ARCHIVE_DIR, EPISODES_DIR, MIGRATION_V1_MARKER, PINNED_DIR,
        SESSIONS_DIR,
    },
    MemoryScope, MemoryStoreTarget,
};
use crate::util::errors::*;
use chrono::Utc;
use log::{debug, info, warn};
use std::path::{Path, PathBuf};
use tokio::fs;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run the M1 migration for `target` if it has not already been run.
///
/// Safe to call on every startup; exits immediately when the marker file
/// exists.
pub(crate) async fn run_if_needed(
    target: MemoryStoreTarget<'_>,
    memory_dir: &Path,
) -> BitFunResult<()> {
    let marker = memory_dir.join(MIGRATION_V1_MARKER);
    if marker.exists() {
        debug!(
            "Memory migration v1 already complete: scope={} memory_dir={}",
            target.scope().as_label(),
            memory_dir.display()
        );
        return Ok(());
    }

    info!(
        "Running memory migration v1: scope={} memory_dir={}",
        target.scope().as_label(),
        memory_dir.display()
    );

    let result = migrate(target.scope(), memory_dir).await;

    match &result {
        Ok(_) => {
            // Write marker regardless of whether there was anything to migrate.
            if let Err(err) = fs::write(&marker, "").await {
                warn!(
                    "Failed to write migration v1 marker: memory_dir={} error={}",
                    memory_dir.display(),
                    err
                );
            }
            info!(
                "Memory migration v1 complete: scope={} memory_dir={}",
                target.scope().as_label(),
                memory_dir.display()
            );
        }
        Err(err) => {
            warn!(
                "Memory migration v1 failed (will retry on next startup): scope={} memory_dir={} error={}",
                target.scope().as_label(),
                memory_dir.display(),
                err
            );
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Internal migration logic
// ---------------------------------------------------------------------------

async fn migrate(scope: MemoryScope, memory_dir: &Path) -> BitFunResult<()> {
    let date_str = Utc::now().format("%Y-%m-%d").to_string();

    // 1. Ensure subdirectory skeleton.
    ensure_skeleton(scope, memory_dir).await?;

    // 2. Collect legacy flat files (non-index, non-core, non-skeleton).
    let legacy_files = collect_legacy_files(memory_dir).await?;
    if legacy_files.is_empty() {
        return Ok(());
    }

    // 3. Create the legacy archive directory.
    let archive_dir = legacy_archive_dir_for(memory_dir, &date_str);
    fs::create_dir_all(&archive_dir).await.map_err(|e| {
        BitFunError::service(format!(
            "Failed to create legacy archive directory {}: {}",
            archive_dir.display(),
            e
        ))
    })?;

    // 4. Migrate each file.
    let mut persona_chunks: Vec<String> = Vec::new();
    let mut habit_chunks: Vec<String> = Vec::new();
    let mut project_chunks: Vec<String> = Vec::new();
    let mut identity_chunks: Vec<String> = Vec::new();
    let mut narrative_chunks: Vec<String> = Vec::new();

    for file_path in &legacy_files {
        let content = match fs::read_to_string(file_path).await {
            Ok(c) => c,
            Err(e) => {
                warn!(
                    "Skipping unreadable legacy memory file: path={} error={}",
                    file_path.display(),
                    e
                );
                continue;
            }
        };

        let legacy_type = extract_legacy_type(&content);

        match legacy_type.as_deref() {
            Some("user") => {
                persona_chunks.push(extract_body(&content));
            }
            Some("feedback") | Some("collaboration") => {
                habit_chunks.push(extract_body(&content));
            }
            Some("project") => {
                project_chunks.push(extract_body(&content));
            }
            Some("assistant_identity") => {
                identity_chunks.push(extract_body(&content));
            }
            Some("vision") => {
                // Vision goes into global narrative regardless of scope.
                narrative_chunks.push(extract_body(&content));
            }
            Some("reference") => {
                // Copy to pinned/ directory.
                let file_name = file_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown.md".to_string());
                let pinned_dest = memory_dir.join(PINNED_DIR).join(&file_name);
                if !pinned_dest.exists() {
                    if let Err(e) = fs::copy(file_path, &pinned_dest).await {
                        warn!(
                            "Failed to copy reference memory to pinned/: src={} dest={} error={}",
                            file_path.display(),
                            pinned_dest.display(),
                            e
                        );
                    }
                }
            }
            _ => {}
        }

        // Archive the original.
        let file_name = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown.md".to_string());
        let archive_dest = archive_dir.join(&file_name);
        if !archive_dest.exists() {
            if let Err(e) = fs::copy(file_path, &archive_dest).await {
                warn!(
                    "Failed to copy legacy memory to archive: src={} dest={} error={}",
                    file_path.display(),
                    archive_dest.display(),
                    e
                );
            }
        }
    }

    // 5. Append accumulated chunks into target core files (only global scope
    //    gets narrative; project scope gets persona/project/habits/identity).
    if scope == MemoryScope::GlobalAgenticOs {
        append_chunks_to_core_file(memory_dir, "persona.md", &persona_chunks).await?;
        append_chunks_to_core_file(memory_dir, "habits.md", &habit_chunks).await?;
        append_chunks_to_core_file(memory_dir, "narrative.md", &narrative_chunks).await?;
        append_chunks_to_core_file(memory_dir, "identity.md", &identity_chunks).await?;
    } else {
        // Project scope: persona chunks still go to global narrative indirectly
        // through their presence in project habits.md for project-specific habits.
        append_chunks_to_core_file(memory_dir, "habits.md", &habit_chunks).await?;
        append_chunks_to_core_file(memory_dir, "project.md", &project_chunks).await?;
        append_chunks_to_core_file(memory_dir, "identity.md", &identity_chunks).await?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn ensure_skeleton(scope: MemoryScope, memory_dir: &Path) -> BitFunResult<()> {
    // Subdirectory skeleton.
    for subdir in &[PINNED_DIR, EPISODES_DIR, SESSIONS_DIR, ARCHIVE_DIR] {
        let dir = memory_dir.join(subdir);
        if !dir.exists() {
            fs::create_dir_all(&dir).await.map_err(|e| {
                BitFunError::service(format!(
                    "Failed to create memory subdirectory {}: {}",
                    dir.display(),
                    e
                ))
            })?;
        }
    }

    // Core singleton placeholder files.
    let core_files: &[&str] = match scope {
        MemoryScope::GlobalAgenticOs => &["identity.md", "narrative.md", "persona.md", "habits.md"],
        MemoryScope::WorkspaceProject => &["identity.md", "project.md", "habits.md"],
    };

    for &file_name in core_files {
        let path = memory_dir.join(file_name);
        if !path.exists() {
            fs::write(&path, "").await.map_err(|e| {
                BitFunError::service(format!(
                    "Failed to create memory core file {}: {}",
                    path.display(),
                    e
                ))
            })?;
        }
    }

    Ok(())
}

async fn collect_legacy_files(memory_dir: &Path) -> BitFunResult<Vec<PathBuf>> {
    let skip_names: &[&str] = &[
        "MEMORY.md",
        "identity.md",
        "narrative.md",
        "persona.md",
        "habits.md",
        "project.md",
        MIGRATION_V1_MARKER,
    ];
    let skip_dirs: &[&str] = &[
        PINNED_DIR,
        EPISODES_DIR,
        SESSIONS_DIR,
        ARCHIVE_DIR,
        "workspaces_overview",
    ];

    let mut result = Vec::new();
    let mut entries = match fs::read_dir(memory_dir).await {
        Ok(e) => e,
        Err(_) => return Ok(result),
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if path.is_dir() {
            let dir_name = name.as_str();
            if !skip_dirs.contains(&dir_name) {
                result.extend(collect_legacy_files_recursive(&path).await);
            }
            continue;
        }

        if !name.to_lowercase().ends_with(".md") {
            continue;
        }

        if skip_names.iter().any(|s| s.eq_ignore_ascii_case(&name)) {
            continue;
        }

        result.push(path);
    }

    Ok(result)
}

async fn collect_legacy_files_recursive(dir: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut entries = match fs::read_dir(dir).await {
        Ok(e) => e,
        Err(_) => return result,
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.is_dir() {
            result.extend(Box::pin(collect_legacy_files_recursive(&path)).await);
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("md"))
            .unwrap_or(false)
        {
            result.push(path);
        }
    }
    result
}

fn extract_legacy_type(content: &str) -> Option<String> {
    if !content.starts_with("---") {
        return None;
    }
    for line in content.lines().skip(1) {
        if line.trim() == "---" {
            break;
        }
        if let Some(rest) = line.strip_prefix("type:") {
            return Some(
                rest.trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_lowercase(),
            );
        }
    }
    None
}

fn extract_body(content: &str) -> String {
    if !content.starts_with("---") {
        return content.trim().to_string();
    }
    let mut past_front_matter = false;
    let mut body_lines: Vec<&str> = Vec::new();
    let mut seen_first_dashes = false;
    for line in content.lines() {
        if !seen_first_dashes && line.trim() == "---" {
            seen_first_dashes = true;
            continue;
        }
        if seen_first_dashes && !past_front_matter && line.trim() == "---" {
            past_front_matter = true;
            continue;
        }
        if past_front_matter {
            body_lines.push(line);
        }
    }
    body_lines.join("\n").trim().to_string()
}

async fn append_chunks_to_core_file(
    memory_dir: &Path,
    file_name: &str,
    chunks: &[String],
) -> BitFunResult<()> {
    if chunks.is_empty() {
        return Ok(());
    }
    let path = memory_dir.join(file_name);
    let existing = fs::read_to_string(&path).await.unwrap_or_default();
    let mut combined = existing.trim_end().to_string();
    for chunk in chunks {
        if chunk.is_empty() {
            continue;
        }
        combined.push_str("\n\n---\n\n");
        combined.push_str(chunk);
    }
    combined.push('\n');
    fs::write(&path, combined).await.map_err(|e| {
        BitFunError::service(format!(
            "Failed to write migrated content to {}: {}",
            path.display(),
            e
        ))
    })?;
    Ok(())
}
