//! Full memory policy texts for **one-shot model prompts** (main agent only).
//! Extraction and consolidation prompts live next to their call sites under
//! `prompts/agent_memory_*`.

use super::MemoryScope;

const AGENT_MEMORY_MAIN: &str = include_str!("prompts/agent_memory_main.md");

/// Shared memory philosophy block — included by both the main agent prompt
/// and the extraction subagent prompt so they cannot drift.
pub(crate) const MEMORY_PHILOSOPHY: &str = include_str!("prompts/_memory_philosophy.md");

/// Shared "never save / never apply" reverse-trigger block.
pub(crate) const MEMORY_NEVER_SAVE: &str = include_str!("prompts/_memory_never_save.md");

/// Build the scope block injected at the top of the unified main prompt.
fn build_main_scope_block(scope: MemoryScope, memory_dir_display: &str) -> String {
    match scope {
        MemoryScope::WorkspaceProject => format!(
            "**Active scope: WORKSPACE.** Memory directory: `{memory_dir_display}`.\n\n\
Applicable type sections in §3: `habit`, `identity` (project-rules anchor), `project`, `episodic`, `reference`. Skip `persona`, `narrative`, `vision`, `workspaces_overview` — they belong to GLOBAL.\n\n\
Files you may eventually write (most writes are owned by the extraction or consolidation passes; in normal turns the main agent does not author memory): `project.md`, `habits.md`, `identity.md`, `pinned/<slug>.md`, `episodes/YYYY-MM/YYYY-MM-DD-<slug>.md`, `MEMORY.md`. Do NOT write `persona.md`, `narrative.md`, anything under `sessions/`, or `workspaces_overview/`."
        ),
        MemoryScope::GlobalAgenticOs => format!(
            "**Active scope: GLOBAL.** Memory directory: `{memory_dir_display}`.\n\n\
Applicable type sections in §3: `persona`, `habit`, `identity` (top-level assistant identity), `narrative` (read-only here), `vision`, `reference`, `workspaces_overview`. Skip `project`, `episodic` — they belong to WORKSPACE.\n\n\
Files you may eventually write: `persona.md`, `habits.md`, `identity.md`, `pinned/<slug>.md`, `workspaces_overview/<slug>.md`, `MEMORY.md`. Do NOT write `narrative.md` (owned by the slow consolidation pass), `project.md`, anything under `episodes/`, or anything under `sessions/`."
        ),
    }
}

pub(crate) fn render_main_memory_prompt(
    scope: MemoryScope,
    memory_dir_display: &str,
    index_file_name: &str,
) -> String {
    let scope_block = build_main_scope_block(scope, memory_dir_display);
    AGENT_MEMORY_MAIN
        .replace("{memory_dir_display}", memory_dir_display)
        .replace("{scope_block}", &scope_block)
        .replace("{philosophy_block}", MEMORY_PHILOSOPHY.trim_end())
        .replace("{never_save_block}", MEMORY_NEVER_SAVE.trim_end())
        .replace("MEMORY.md", index_file_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_main_prompt_declares_scope_and_skips_global_only_types() {
        let prompt = render_main_memory_prompt(
            MemoryScope::WorkspaceProject,
            "/workspace/.sparo_os/memory",
            "MEMORY.md",
        );
        assert!(prompt.contains("Active scope: WORKSPACE"));
        assert!(prompt.contains("Skip `persona`, `narrative`"));
        assert!(prompt.contains("Index, not transcript"));
        assert!(!prompt.contains("{scope_block}"));
        assert!(!prompt.contains("{philosophy_block}"));
        assert!(!prompt.contains("{never_save_block}"));
    }

    #[test]
    fn global_main_prompt_declares_scope_and_skips_workspace_only_types() {
        let prompt = render_main_memory_prompt(
            MemoryScope::GlobalAgenticOs,
            "/home/user/.sparo_os/memory",
            "MEMORY.md",
        );
        assert!(prompt.contains("Active scope: GLOBAL"));
        assert!(prompt.contains("Skip `project`, `episodic`"));
        assert!(prompt.contains("narrative.md"));
        assert!(!prompt.contains("{scope_block}"));
    }

    #[test]
    fn philosophy_includes_index_not_transcript_rule() {
        assert!(MEMORY_PHILOSOPHY.contains("Index, not transcript"));
        assert!(MEMORY_PHILOSOPHY.contains("Default action"));
        assert!(MEMORY_PHILOSOPHY.contains("Memory ≠ facts"));
    }

    #[test]
    fn never_save_block_includes_secrets_and_re_derivable_rules() {
        assert!(MEMORY_NEVER_SAVE.contains("Secrets"));
        assert!(MEMORY_NEVER_SAVE.contains("Re-derivable facts"));
        assert!(MEMORY_NEVER_SAVE.contains("Inferred personal traits"));
    }
}
