//! Cold-injection pyramid for memory context.
//!
//! The pyramid has two layers assembled in order:
//!
//! **Global layer** (injected for every session):
//!   1. global `identity.md`  — core identity anchor
//!   2. global `narrative.md` — autobiographical story ("our story")
//!   3. global `persona.md`   — user profile
//!   4. global `habits.md`    — cross-project collaboration style
//!   5. global `MEMORY.md`    — main index (≤120 lines)
//!   6. most recent 3 global `sessions/*.md` titles + one-line summary
//!
//! **Project layer** (appended when the session is workspace-scoped):
//!   7. project `identity.md` — project-level rules anchor (may be empty)
//!   8. project `project.md`  — project ontology
//!   9. project `habits.md`   — project-specific style
//!  10. project `MEMORY.md`   — project index (≤120 lines)
//!  11. most recent 3 project `sessions/*.md` titles + one-line summary
//!
//! Episodes are intentionally NOT injected; they appear only in the manifest
//! so agents can retrieve them via agentic search (Read/Grep/Glob) on demand.

use super::{
    ensure_memory_store_for_target, format_path_for_prompt, memory_store_dir_path_for_target,
    render_global_main_memory_prompt, render_workspace_main_memory_prompt, MemoryScope,
    MemoryStoreTarget, MEMORY_INDEX_FILE,
};
use crate::infrastructure::get_path_manager_arc;
use crate::util::errors::*;
use log::debug;
use tokio::fs;

/// Prefix prepended to cold-injected memory file excerpts (not the main memory policy block).
const MEMORY_ACTIVATION_PREFACE: &str = include_str!("prompts/agent_memory_inject_preface.md");

// ---------------------------------------------------------------------------
// Hard limits
// ---------------------------------------------------------------------------

/// Max lines to show from a core singleton file (identity/narrative/persona/project/habits).
const CORE_FILE_MAX_LINES: usize = 200;
/// Max lines to show from MEMORY.md index.
const INDEX_MAX_LINES: usize = 120;
/// Number of recent session summaries to include.
const RECENT_SESSIONS_COUNT: usize = 3;
/// Max chars to read from each session summary (title + first content line).
const SESSION_SUMMARY_MAX_CHARS: usize = 200;

// ---------------------------------------------------------------------------
// Public interface (signatures unchanged from pre-M2)
// ---------------------------------------------------------------------------

pub(crate) async fn build_memory_prompt_for_target(
    target: MemoryStoreTarget<'_>,
) -> BitFunResult<String> {
    ensure_memory_store_for_target(target).await?;
    let memory_dir = memory_store_dir_path_for_target(target);
    let memory_dir_display = format_path_for_prompt(&memory_dir);
    Ok(match target {
        MemoryStoreTarget::WorkspaceProject(_) => render_workspace_main_memory_prompt(
            &memory_dir_display,
            MEMORY_INDEX_FILE,
        ),
        MemoryStoreTarget::GlobalAgenticOs => {
            render_global_main_memory_prompt(&memory_dir_display, MEMORY_INDEX_FILE)
        }
    })
}

/// Build the cold-injection context that is prepended to every session's
/// system prompt.
///
/// For a `GlobalAgenticOs` target, only the global layer is assembled.
/// For a `WorkspaceProject` target, both layers are assembled (global first,
/// then project).
pub(crate) async fn build_memory_files_context_for_target(
    target: MemoryStoreTarget<'_>,
) -> BitFunResult<Option<String>> {
    ensure_memory_store_for_target(target).await?;

    let mut sections: Vec<String> = Vec::new();

    // --- Global layer (always included) ---
    let global_dir = get_path_manager_arc().agentic_os_memory_dir();
    if global_dir.exists() {
        let global_section = build_global_layer(&global_dir).await?;
        if !global_section.trim().is_empty() {
            sections.push(global_section);
        }
    }

    // --- Project layer (only for workspace-scoped targets) ---
    if let MemoryStoreTarget::WorkspaceProject(_) = target {
        let project_dir = memory_store_dir_path_for_target(target);
        if project_dir.exists() {
            let project_section = build_project_layer(&project_dir).await?;
            if !project_section.trim().is_empty() {
                sections.push(project_section);
            }
        }
    }

    if sections.is_empty() {
        Ok(None)
    } else {
        // Lead with a short activation-discipline preface so the main agent
        // treats the injected memory as silent background, not as a script
        // to recite back at the user. We only prepend it when there is
        // actual memory content to discipline.
        let body = sections.join("\n\n");
        Ok(Some(format!("{}\n\n{}", MEMORY_ACTIVATION_PREFACE, body)))
    }
}

// ---------------------------------------------------------------------------
// Layer builders
// ---------------------------------------------------------------------------

async fn build_global_layer(global_dir: &std::path::Path) -> BitFunResult<String> {
    let mut parts: Vec<String> = Vec::new();

    // identity → narrative → persona → habits (canonical order)
    for file_name in &["identity.md", "narrative.md", "persona.md", "habits.md"] {
        let content = read_core_file(global_dir, file_name).await;
        if !content.trim().is_empty() {
            parts.push(format!("## Memory: {} (global)\n\n{}", file_name, content));
        }
    }

    // Global MEMORY.md index
    let index_content = read_index_file(global_dir).await;
    if !index_content.trim().is_empty() {
        parts.push(format!("## Memory Index (global)\n\n{}", index_content));
    }

    // Recent global sessions
    let sessions_summary =
        build_recent_sessions_summary(global_dir, MemoryScope::GlobalAgenticOs).await;
    if !sessions_summary.trim().is_empty() {
        parts.push(format!(
            "## Recent sessions (global)\n\n{}",
            sessions_summary
        ));
    }

    Ok(parts.join("\n\n"))
}

async fn build_project_layer(project_dir: &std::path::Path) -> BitFunResult<String> {
    let mut parts: Vec<String> = Vec::new();

    // identity → project → habits (canonical order for project scope)
    for file_name in &["identity.md", "project.md", "habits.md"] {
        let content = read_core_file(project_dir, file_name).await;
        if !content.trim().is_empty() {
            parts.push(format!(
                "## Memory: {} (workspace)\n\n{}",
                file_name, content
            ));
        }
    }

    // Project MEMORY.md index
    let index_content = read_index_file(project_dir).await;
    if !index_content.trim().is_empty() {
        parts.push(format!("## Memory Index (workspace)\n\n{}", index_content));
    }

    // Recent project sessions
    let sessions_summary =
        build_recent_sessions_summary(project_dir, MemoryScope::WorkspaceProject).await;
    if !sessions_summary.trim().is_empty() {
        parts.push(format!(
            "## Recent sessions (workspace)\n\n{}",
            sessions_summary
        ));
    }

    Ok(parts.join("\n\n"))
}

// ---------------------------------------------------------------------------
// File readers
// ---------------------------------------------------------------------------

async fn read_core_file(memory_dir: &std::path::Path, file_name: &str) -> String {
    let path = memory_dir.join(file_name);
    match fs::read_to_string(&path).await {
        Ok(content) if !content.trim().is_empty() => truncate_lines(&content, CORE_FILE_MAX_LINES),
        _ => String::new(),
    }
}

async fn read_index_file(memory_dir: &std::path::Path) -> String {
    let path = memory_dir.join(MEMORY_INDEX_FILE);
    match fs::read_to_string(&path).await {
        Ok(content) if !content.trim().is_empty() => truncate_lines(&content, INDEX_MAX_LINES),
        _ => String::new(),
    }
}

fn truncate_lines(content: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() <= max_lines {
        return content.to_string();
    }
    let truncated = lines[..max_lines].join("\n");
    format!("{}\n[... truncated at {} lines]", truncated, max_lines)
}

async fn build_recent_sessions_summary(memory_dir: &std::path::Path, scope: MemoryScope) -> String {
    let sessions_dir = memory_dir.join("sessions");
    if !sessions_dir.exists() {
        return String::new();
    }

    let mut session_files: Vec<std::path::PathBuf> = Vec::new();
    let mut entries = match fs::read_dir(&sessions_dir).await {
        Ok(e) => e,
        Err(_) => return String::new(),
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("md"))
            .unwrap_or(false)
        {
            session_files.push(path);
        }
    }

    // Sort descending (most recent first) by file name (YYYY-MM-DD-... prefix).
    session_files.sort();
    session_files.reverse();
    session_files.truncate(RECENT_SESSIONS_COUNT);
    session_files.reverse(); // oldest-first for reading order

    let mut summaries: Vec<String> = Vec::new();
    for path in &session_files {
        let content = match fs::read_to_string(path).await {
            Ok(c) => c,
            Err(_) => continue,
        };
        let title = extract_session_title(&content, path);
        let first_line = extract_first_content_line(&content);
        let entry = if first_line.is_empty() {
            format!("- {}", title)
        } else {
            format!("- {} — {}", title, first_line)
        };
        // Truncate the combined entry.
        if entry.chars().count() > SESSION_SUMMARY_MAX_CHARS {
            summaries.push(
                entry
                    .chars()
                    .take(SESSION_SUMMARY_MAX_CHARS)
                    .collect::<String>(),
            );
        } else {
            summaries.push(entry);
        }
    }

    debug!(
        "Built recent sessions summary: scope={} count={}",
        scope.as_label(),
        summaries.len()
    );

    summaries.join("\n")
}

fn extract_session_title(content: &str, path: &std::path::Path) -> String {
    // Try H1 heading first.
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("# ") {
            return heading.trim().to_string();
        }
    }
    // Fall back to file stem.
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Session")
        .to_string()
}

fn extract_first_content_line(content: &str) -> String {
    // Skip front matter and headings; return the first non-empty prose line.
    let mut in_front_matter = false;
    let mut past_first_dashes = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            if !past_first_dashes {
                in_front_matter = true;
                past_first_dashes = true;
                continue;
            } else if in_front_matter {
                in_front_matter = false;
                continue;
            }
        }
        if in_front_matter || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        return trimmed.to_string();
    }
    String::new()
}
