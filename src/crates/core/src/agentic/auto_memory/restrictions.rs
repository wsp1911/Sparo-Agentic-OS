use crate::agentic::tools::{ToolPathPolicy, ToolRuntimeRestrictions};
use std::collections::BTreeSet;

pub fn build_auto_memory_runtime_restrictions(memory_dir: &str) -> ToolRuntimeRestrictions {
    build_auto_memory_runtime_restrictions_with_extra_roots(memory_dir, &[])
}

/// Like `build_auto_memory_runtime_restrictions` but also allows writing to
/// additional roots (e.g., the global memory directory during workspace
/// scope extraction for cross-cutting user-level memories).
pub fn build_auto_memory_runtime_restrictions_with_extra_roots(
    memory_dir: &str,
    extra_write_roots: &[&str],
) -> ToolRuntimeRestrictions {
    let mut write_roots = vec![memory_dir.to_string()];
    write_roots.extend(extra_write_roots.iter().map(|s| s.to_string()));
    let edit_roots = write_roots.clone();
    let delete_roots = write_roots.clone();

    ToolRuntimeRestrictions {
        // Expose ONLY the dedicated memory tools to the auto-memory fork.
        // Generic file tools (Read/Glob/Grep/Write/Edit/Delete) are not in
        // this allow-list, so the model can never call them. The memory
        // tools internally enforce that every path resolves inside one of
        // the `path_policy` roots — defense in depth alongside the runtime
        // path enforcement that wraps the inner tools.
        allowed_tool_names: [
            "MemoryRead",
            "MemoryGlob",
            "MemoryGrep",
            "MemoryWrite",
            "MemoryEdit",
            "MemoryDelete",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>(),
        denied_tool_names: BTreeSet::new(),
        path_policy: ToolPathPolicy {
            write_roots,
            edit_roots,
            delete_roots,
        },
        disable_snapshot_tracking: true,
    }
}
