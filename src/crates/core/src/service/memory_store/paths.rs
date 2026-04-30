use super::{
    ensure_markdown_placeholder,
    global_overview::ensure_global_memory_overview_files,
    layout::{
        ARCHIVE_DIR, BOOTSTRAP_V2_MARKER, EPISODES_DIR, GLOBAL_IDENTITY_TEMPLATE,
        GLOBAL_MEMORY_INDEX_TEMPLATE, HABITS_TEMPLATE, NARRATIVE_TEMPLATE, PERSONA_TEMPLATE,
        PINNED_DIR, PROJECT_IDENTITY_TEMPLATE, PROJECT_MEMORY_INDEX_TEMPLATE, PROJECT_TEMPLATE,
        SESSIONS_DIR,
    },
    migration, MemoryScope, MemoryStoreTarget, MEMORY_DIR_NAME, MEMORY_INDEX_FILE,
};
use crate::infrastructure::get_path_manager_arc;
use crate::util::errors::*;
use log::{debug, info};
use std::path::PathBuf;
use tokio::fs;

pub(crate) fn memory_store_dir_path_for_target(target: MemoryStoreTarget<'_>) -> PathBuf {
    let path_manager = get_path_manager_arc();
    let path = match target {
        MemoryStoreTarget::WorkspaceProject(workspace_root) => {
            path_manager.project_memory_dir(workspace_root)
        }
        MemoryStoreTarget::GlobalAgenticOs => path_manager.agentic_os_memory_dir(),
    };
    debug!(
        "Resolved memory store directory: scope={} memory_dir={} storage_subdir={}",
        target.scope().as_label(),
        path.display(),
        MEMORY_DIR_NAME
    );
    path
}

pub(crate) async fn ensure_memory_store_for_target(
    target: MemoryStoreTarget<'_>,
) -> BitFunResult<()> {
    let memory_dir = memory_store_dir_path_for_target(target);
    if !memory_dir.exists() {
        fs::create_dir_all(&memory_dir).await.map_err(|e| {
            BitFunError::service(format!(
                "Failed to create memory directory {}: {}",
                memory_dir.display(),
                e
            ))
        })?;
    }

    // Choose the correct MEMORY.md template based on scope.
    let index_template = match target.scope() {
        MemoryScope::GlobalAgenticOs => GLOBAL_MEMORY_INDEX_TEMPLATE,
        MemoryScope::WorkspaceProject => PROJECT_MEMORY_INDEX_TEMPLATE,
    };
    let created_memory_index =
        ensure_markdown_placeholder(&memory_dir.join(MEMORY_INDEX_FILE), index_template).await?;

    // Ensure the full directory skeleton and core files.
    ensure_memory_skeleton(target.scope(), &memory_dir).await?;

    // Run the one-time M1 migration (idempotent, guarded by marker file).
    migration::run_if_needed(target, &memory_dir).await?;

    // Run the one-time v2 bootstrap that upgrades any previously-empty core
    // files to the meaningful templates.
    run_bootstrap_v2_if_needed(target.scope(), &memory_dir).await?;

    if matches!(target, MemoryStoreTarget::GlobalAgenticOs) {
        ensure_global_memory_overview_files(&memory_dir).await?;
    }

    debug!(
        "Ensured memory store files: scope={} path={} created_memory_index={}",
        target.scope().as_label(),
        memory_dir.display(),
        created_memory_index
    );

    Ok(())
}

/// Create the subdirectory skeleton and core placeholder files for the given
/// scope.  Safe to call multiple times (all operations are idempotent).
pub(crate) async fn ensure_memory_skeleton(
    scope: MemoryScope,
    memory_dir: &std::path::Path,
) -> BitFunResult<()> {
    // Subdirectories present in both scopes.
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

    // Core singleton placeholder files with meaningful templates.
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

    for &(file_name, template) in core_files {
        ensure_markdown_placeholder(&memory_dir.join(file_name), template).await?;
    }

    Ok(())
}

/// One-time bootstrap that rewrites empty core files with meaningful templates.
///
/// Guarded by `.initialized_v2` so it only runs once per memory directory.
/// Previously-empty files that resulted from the old skeleton logic are
/// upgraded; files that already have content are left untouched.
async fn run_bootstrap_v2_if_needed(
    scope: MemoryScope,
    memory_dir: &std::path::Path,
) -> BitFunResult<()> {
    let marker = memory_dir.join(BOOTSTRAP_V2_MARKER);
    if marker.exists() {
        return Ok(());
    }

    info!(
        "Running memory bootstrap v2 (non-empty templates): scope={} memory_dir={}",
        scope.as_label(),
        memory_dir.display()
    );

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

    for &(file_name, template) in core_files {
        let path = memory_dir.join(file_name);
        let is_empty = match fs::read_to_string(&path).await {
            Ok(content) => content.trim().is_empty(),
            Err(_) => true,
        };

        if is_empty {
            debug!(
                "Bootstrap v2: writing template to empty core file: file={} scope={}",
                file_name,
                scope.as_label()
            );
            if let Err(e) = fs::write(&path, template).await {
                debug!(
                    "Bootstrap v2: failed to write template: file={} error={}",
                    file_name, e
                );
            }
        }
    }

    // Upgrade MEMORY.md if it is also empty.
    let index_path = memory_dir.join(MEMORY_INDEX_FILE);
    let index_is_empty = match fs::read_to_string(&index_path).await {
        Ok(content) => content.trim().is_empty(),
        Err(_) => true,
    };
    if index_is_empty {
        let template = match scope {
            MemoryScope::GlobalAgenticOs => GLOBAL_MEMORY_INDEX_TEMPLATE,
            MemoryScope::WorkspaceProject => PROJECT_MEMORY_INDEX_TEMPLATE,
        };
        let _ = fs::write(&index_path, template).await;
    }

    // Write marker.
    if let Err(e) = fs::write(&marker, "").await {
        debug!(
            "Bootstrap v2: failed to write marker: memory_dir={} error={}",
            memory_dir.display(),
            e
        );
    }

    info!(
        "Memory bootstrap v2 complete: scope={} memory_dir={}",
        scope.as_label(),
        memory_dir.display()
    );

    Ok(())
}
