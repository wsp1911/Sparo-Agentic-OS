use super::{
    ensure_memory_store_for_target, format_manifest_path, layout::classify_relative_path,
    layout::MemoryPathCategory, list_memory_files_recursive, memory_store_dir_path_for_target,
    MemoryScope, MemoryStoreTarget, MEMORY_INDEX_FILE, MEMORY_MANIFEST_MAX_FILES,
};
use crate::util::errors::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(crate) async fn build_memory_manifest_for_target(
    target: MemoryStoreTarget<'_>,
) -> BitFunResult<Option<String>> {
    ensure_memory_store_for_target(target).await?;
    let memory_dir = memory_store_dir_path_for_target(target);
    let mut memory_files = vec![memory_dir.join(MEMORY_INDEX_FILE)];
    memory_files.extend(list_memory_files_recursive(&memory_dir).await?);

    Ok(render_memory_manifest(
        target.scope(),
        &memory_dir,
        memory_files,
    ))
}

fn render_memory_manifest(
    scope: MemoryScope,
    memory_dir: &Path,
    memory_files: Vec<PathBuf>,
) -> Option<String> {
    let mut seen = HashSet::new();
    let mut index_files: Vec<String> = Vec::new();
    let mut core_files: Vec<String> = Vec::new();
    let mut pinned_files: Vec<String> = Vec::new();
    let mut episode_files: Vec<String> = Vec::new();
    let mut session_files: Vec<String> = Vec::new();
    let mut workspace_overview_files: Vec<String> = Vec::new();
    let mut other_files: Vec<String> = Vec::new();

    for path in memory_files {
        let relative = format_manifest_path(&path, memory_dir);
        if relative.is_empty() || !seen.insert(relative.clone()) {
            continue;
        }

        // Never include archive files in the active manifest.
        let category = classify_relative_path(&relative);
        if category == MemoryPathCategory::Archive {
            continue;
        }

        match category {
            MemoryPathCategory::Index => index_files.push(relative),
            MemoryPathCategory::Core => core_files.push(relative),
            MemoryPathCategory::Pinned => pinned_files.push(relative),
            MemoryPathCategory::Episode => episode_files.push(relative),
            MemoryPathCategory::Session => session_files.push(relative),
            MemoryPathCategory::WorkspaceOverview => workspace_overview_files.push(relative),
            _ => other_files.push(relative),
        }
    }

    // Sort each group.
    core_files.sort_by(|a, b| core_sort_key(a).cmp(&core_sort_key(b)));
    pinned_files.sort();
    // Episodes: show only the most recent 30 files to keep the manifest lean.
    episode_files.sort();
    episode_files.reverse();
    episode_files.truncate(30);
    episode_files.reverse();
    // Sessions: show only the most recent 10 files.
    session_files.sort();
    session_files.reverse();
    session_files.truncate(10);
    session_files.reverse();
    workspace_overview_files.sort();
    other_files.sort();

    // Build ordered list within the global file budget.
    let mut ordered: Vec<String> = Vec::new();
    ordered.extend(index_files);
    ordered.extend(core_files);
    ordered.extend(pinned_files);
    ordered.extend(episode_files);
    ordered.extend(session_files);
    ordered.extend(other_files);

    let limit = MEMORY_MANIFEST_MAX_FILES.min(ordered.len());
    let ordinary: Vec<String> = ordered.into_iter().take(limit).collect();

    let remaining = MEMORY_MANIFEST_MAX_FILES.saturating_sub(ordinary.len());
    let workspace_overviews: Vec<String> = workspace_overview_files
        .into_iter()
        .take(remaining)
        .collect();

    match scope {
        MemoryScope::WorkspaceProject => {
            if ordinary.is_empty() {
                None
            } else {
                Some(render_file_list(&ordinary))
            }
        }
        MemoryScope::GlobalAgenticOs => {
            let mut sections = Vec::new();
            if !ordinary.is_empty() {
                sections.push(format!("### Memories\n\n{}", render_file_list(&ordinary)));
            }
            if !workspace_overviews.is_empty() {
                sections.push(format!(
                    "### Workspace overview files\n\n{}",
                    render_file_list(&workspace_overviews)
                ));
            }
            if sections.is_empty() {
                None
            } else {
                Some(sections.join("\n\n"))
            }
        }
    }
}

/// Sort key for core singleton files so they appear in the canonical order:
/// identity → narrative → persona → project → habits.
fn core_sort_key(rel: &str) -> u8 {
    match rel {
        "identity.md" => 0,
        "narrative.md" => 1,
        "persona.md" => 2,
        "project.md" => 3,
        "habits.md" => 4,
        _ => 99,
    }
}

fn render_file_list(paths: &[String]) -> String {
    paths
        .iter()
        .map(|path| format!("- {}", path))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::render_memory_manifest;
    use crate::service::memory_store::MemoryScope;
    use std::path::PathBuf;

    #[test]
    fn workspace_manifest_lists_core_files_first() {
        let memory_dir = PathBuf::from("/memory");
        let manifest = render_memory_manifest(
            MemoryScope::WorkspaceProject,
            &memory_dir,
            vec![
                memory_dir.join("habits.md"),
                memory_dir.join("MEMORY.md"),
                memory_dir.join("identity.md"),
                memory_dir.join("project.md"),
                memory_dir.join("episodes/2026-04/2026-04-29-foo.md"),
            ],
        )
        .expect("workspace manifest should exist");

        let lines: Vec<&str> = manifest.lines().collect();
        assert_eq!(lines[0], "- MEMORY.md");
        assert_eq!(lines[1], "- identity.md");
        assert_eq!(lines[2], "- project.md");
        assert_eq!(lines[3], "- habits.md");
        // Episode comes after core files.
        assert!(manifest.contains("episodes/2026-04/2026-04-29-foo.md"));
    }

    #[test]
    fn global_manifest_groups_workspace_overviews() {
        let memory_dir = PathBuf::from("/memory");
        let manifest = render_memory_manifest(
            MemoryScope::GlobalAgenticOs,
            &memory_dir,
            vec![
                memory_dir.join("narrative.md"),
                memory_dir.join("workspaces_overview/bitfun--1234abcd.md"),
                memory_dir.join("MEMORY.md"),
                memory_dir.join("identity.md"),
            ],
        )
        .expect("global manifest should exist");

        assert!(manifest.contains("### Memories"));
        assert!(manifest.contains("### Workspace overview files"));
        assert!(manifest.contains("- identity.md"));
        assert!(manifest.contains("- narrative.md"));
    }

    #[test]
    fn archive_files_are_excluded_from_manifest() {
        let memory_dir = PathBuf::from("/memory");
        let manifest = render_memory_manifest(
            MemoryScope::WorkspaceProject,
            &memory_dir,
            vec![
                memory_dir.join("MEMORY.md"),
                memory_dir.join("archive/legacy-2026-04-29/old.md"),
                memory_dir.join("habits.md"),
            ],
        )
        .expect("manifest should exist");

        assert!(!manifest.contains("archive/"));
        assert!(manifest.contains("habits.md"));
    }
}
