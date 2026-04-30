use std::path::{Path, PathBuf};

/// Subdirectory for user-pinned entries.
pub(crate) const PINNED_DIR: &str = "pinned";
/// Subdirectory for episodic memories, bucket by YYYY-MM.
pub(crate) const EPISODES_DIR: &str = "episodes";
/// Subdirectory for per-session summary files.
pub(crate) const SESSIONS_DIR: &str = "sessions";
/// Subdirectory for archived (low-strength or superseded) entries.
pub(crate) const ARCHIVE_DIR: &str = "archive";
/// Subdirectory for workspace routing overviews (global scope only).
pub(crate) const WORKSPACES_OVERVIEW_DIR: &str = "workspaces_overview";

/// Marker file that signals M1 migration has already run.
pub(crate) const MIGRATION_V1_MARKER: &str = ".migrated_v1";

/// Marker file that signals the v2 bootstrap (non-empty templates) has run.
pub(crate) const BOOTSTRAP_V2_MARKER: &str = ".initialized_v2";

/// Core singleton files present in the global memory scope.
pub(crate) const GLOBAL_CORE_FILES: &[&str] =
    &["identity.md", "narrative.md", "persona.md", "habits.md"];

/// Core singleton files present in the project memory scope.
pub(crate) const PROJECT_CORE_FILES: &[&str] = &["identity.md", "project.md", "habits.md"];

// ---------------------------------------------------------------------------
// Bootstrap templates – written once when a memory store is first initialized.
// Only used when the file does not yet exist (idempotent placeholder logic).
// ---------------------------------------------------------------------------

/// MEMORY.md template for the global scope.
pub(crate) const GLOBAL_MEMORY_INDEX_TEMPLATE: &str = "\
# Memory Index

This file is the navigation directory for the global Agentic OS memory store.
Read this file first before searching for specific memories. Each section links to relevant files.

## Map

- `identity.md` — assistant identity and long-term relationship anchor
- `narrative.md` — autobiographical story of the collaboration
- `persona.md` — durable user profile and long-term preferences
- `habits.md` — cross-project collaboration style and preferences

## Active Topics

<!-- Populated by the assistant: topic → relevant files, updated as topics emerge -->

## Recent Sessions

<!-- Updated after each session summary is written to sessions/ -->

## Open Loops

<!-- Unfinished items, commitments, or pending follow-ups across sessions -->
";

/// MEMORY.md template for the project scope.
pub(crate) const PROJECT_MEMORY_INDEX_TEMPLATE: &str = "\
# Memory Index

This file is the navigation directory for this workspace's memory store.
Read this file first before searching for specific memories. Each section links to relevant files.

## Map

- `identity.md` — workspace-level rules and operating constraints
- `project.md` — project ontology, goals, and architectural decisions
- `habits.md` — project-specific collaboration preferences

## Active Topics

<!-- Populated by the assistant: topic → relevant files, updated as topics emerge -->

## Recent Episodes

<!-- Notable events in this workspace (auto-updated after episodic entries are created) -->

## Open Loops

<!-- Unfinished items, commitments, or pending follow-ups for this workspace -->
";

/// Template for `identity.md` in the global scope.
pub(crate) const GLOBAL_IDENTITY_TEMPLATE: &str = "\
# Identity

This file captures durable guidance about the Agentic OS assistant's identity, role model,
and relationship posture with this user.

Update this file only when the user explicitly defines or adjusts the assistant's
personality, capability expectations, or long-term relationship model.

<!-- Populated through explicit user direction or slow consolidation passes. -->
";

/// Template for `identity.md` in the project scope.
pub(crate) const PROJECT_IDENTITY_TEMPLATE: &str = "\
# Project Identity

This file captures workspace-level operating rules, constraints, and principles
that the assistant should follow when working in this project.

<!-- Populated through explicit user direction. -->
";

/// Template for `narrative.md` (global scope only).
pub(crate) const NARRATIVE_TEMPLATE: &str = "\
# Narrative

This is the autobiographical story of the collaboration between the user and Agentic OS.
It captures shared milestones, turning points, and the arc of the working relationship.

This file is updated only during slow consolidation passes — not during normal sessions.

<!-- Updated by slow consolidation passes. Do NOT write to this file during normal extraction. -->
";

/// Template for `persona.md` (global scope).
pub(crate) const PERSONA_TEMPLATE: &str = "\
# User Profile

This file captures durable facts about the user: their role, expertise, working style,
and long-term goals. Only record information that is explicit or high-confidence.

<!-- Populated as the assistant learns about the user across sessions. -->
";

/// Template for `project.md` (project scope).
pub(crate) const PROJECT_TEMPLATE: &str = "\
# Project

This file captures the ontology of this workspace: its purpose, goals,
architecture decisions, key context, and current focus.

<!-- Populated as the assistant learns about the project across sessions. -->
";

/// Template for `habits.md` (both scopes).
pub(crate) const HABITS_TEMPLATE: &str = "\
# Collaboration Habits

This file captures how the user prefers to collaborate: feedback style, planning rhythm,
detail level, and specific guidance about what to do or avoid.

Record both corrections (\"don't do X\") and confirmations (\"yes, keep doing Y\")
so the user does not need to repeat guidance across sessions.

<!-- Populated when the user corrects or confirms specific approaches. -->
";

/// Compute the path for an episode entry.
///
/// `date` must be in `YYYY-MM-DD` format; `slug` is a short kebab-case label.
/// Result: `<memory_dir>/episodes/YYYY-MM/YYYY-MM-DD-<slug>.md`
pub(crate) fn episode_path_for(memory_dir: &Path, date: &str, slug: &str) -> PathBuf {
    let month = &date[..7]; // "YYYY-MM"
    memory_dir
        .join(EPISODES_DIR)
        .join(month)
        .join(format!("{}-{}.md", date, slug))
}

/// Compute the path for a session summary entry.
///
/// Result: `<memory_dir>/sessions/YYYY-MM-DD-<session_id>.md`
pub(crate) fn session_summary_path_for(memory_dir: &Path, date: &str, session_id: &str) -> PathBuf {
    memory_dir
        .join(SESSIONS_DIR)
        .join(format!("{}-{}.md", date, session_id))
}

/// Compute the path for a pinned entry.
///
/// Result: `<memory_dir>/pinned/<slug>.md`
pub(crate) fn pinned_path_for(memory_dir: &Path, slug: &str) -> PathBuf {
    memory_dir.join(PINNED_DIR).join(format!("{}.md", slug))
}

/// Compute the archive path for a file being superseded.
///
/// Result: `<memory_dir>/archive/<date>-<original_name>`
pub(crate) fn archive_path_for(memory_dir: &Path, date: &str, original_name: &str) -> PathBuf {
    memory_dir
        .join(ARCHIVE_DIR)
        .join(format!("{}-{}", date, original_name))
}

/// Compute the legacy migration archive directory.
///
/// Result: `<memory_dir>/archive/legacy-<date>/`
pub(crate) fn legacy_archive_dir_for(memory_dir: &Path, date: &str) -> PathBuf {
    memory_dir
        .join(ARCHIVE_DIR)
        .join(format!("legacy-{}", date))
}

/// Classify a relative memory path (relative to the memory root) into a
/// named category for manifest grouping and cold-injection ordering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MemoryPathCategory {
    /// MEMORY.md index
    Index,
    /// One of the core singleton files (identity/narrative/persona/project/habits)
    Core,
    /// pinned/
    Pinned,
    /// episodes/
    Episode,
    /// sessions/
    Session,
    /// workspaces_overview/
    WorkspaceOverview,
    /// archive/
    Archive,
    /// Anything else
    Other,
}

pub(crate) fn classify_relative_path(rel: &str) -> MemoryPathCategory {
    let normalized = rel.replace('\\', "/");
    if normalized == "MEMORY.md" {
        return MemoryPathCategory::Index;
    }
    if GLOBAL_CORE_FILES.contains(&normalized.as_str())
        || PROJECT_CORE_FILES.contains(&normalized.as_str())
    {
        return MemoryPathCategory::Core;
    }
    if normalized.starts_with("pinned/") {
        return MemoryPathCategory::Pinned;
    }
    if normalized.starts_with("episodes/") {
        return MemoryPathCategory::Episode;
    }
    if normalized.starts_with("sessions/") {
        return MemoryPathCategory::Session;
    }
    if normalized.starts_with("workspaces_overview/") {
        return MemoryPathCategory::WorkspaceOverview;
    }
    if normalized.starts_with("archive/") {
        return MemoryPathCategory::Archive;
    }
    MemoryPathCategory::Other
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn episode_path_has_correct_structure() {
        let dir = PathBuf::from("/memory");
        let path = episode_path_for(&dir, "2026-04-29", "fix-streaming");
        assert_eq!(
            path.to_string_lossy().replace('\\', "/"),
            "/memory/episodes/2026-04/2026-04-29-fix-streaming.md"
        );
    }

    #[test]
    fn classify_relative_path_identity() {
        assert_eq!(
            classify_relative_path("identity.md"),
            MemoryPathCategory::Core
        );
        assert_eq!(
            classify_relative_path("narrative.md"),
            MemoryPathCategory::Core
        );
        assert_eq!(
            classify_relative_path("MEMORY.md"),
            MemoryPathCategory::Index
        );
        assert_eq!(
            classify_relative_path("episodes/2026-04/2026-04-29-foo.md"),
            MemoryPathCategory::Episode
        );
        assert_eq!(
            classify_relative_path("archive/legacy-2026-04-29/old.md"),
            MemoryPathCategory::Archive
        );
    }
}
