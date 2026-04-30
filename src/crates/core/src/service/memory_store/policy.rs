//! Full memory policy texts for **one-shot model prompts** (main agent only).
//! Extraction and consolidation prompts live next to their call sites under
//! `prompts/agent_memory_*`.

const AGENT_MEMORY_MAIN_WORKSPACE: &str =
    include_str!("prompts/agent_memory_main_workspace.md");
const AGENT_MEMORY_MAIN_GLOBAL: &str = include_str!("prompts/agent_memory_main_global.md");

pub(crate) fn render_workspace_main_memory_prompt(
    memory_dir_display: &str,
    index_file_name: &str,
) -> String {
    AGENT_MEMORY_MAIN_WORKSPACE
        .replace("{memory_dir_display}", memory_dir_display)
        .replace("MEMORY.md", index_file_name)
}

pub(crate) fn render_global_main_memory_prompt(
    memory_dir_display: &str,
    index_file_name: &str,
) -> String {
    AGENT_MEMORY_MAIN_GLOBAL
        .replace("{memory_dir_display}", memory_dir_display)
        .replace("MEMORY.md", index_file_name)
}
