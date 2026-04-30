//! Slow-pass consolidation prompts.
//!
//! Two separate forks:
//!
//! **Global slow pass** (monthly):
//! - Reads all workspace sessions/*.md summaries
//! - Reads cross-workspace high-salience episode titles
//! - Rewrites global narrative.md, persona.md, habits.md, MEMORY.md
//! - Old versions archived before overwrite
//! - Write roots: global memory directory only
//!
//! **Project slow pass** (monthly or at milestone):
//! - Reads project episodes/ and sessions/
//! - Rewrites project.md, project habits.md, project identity.md, project MEMORY.md
//! - MUST NOT write to narrative.md
//! - Write roots: project memory directory only

use crate::agentic::auto_memory::build_auto_memory_runtime_restrictions;

const GLOBAL_SLOW_TEMPLATE: &str =
    include_str!("../../service/memory_store/prompts/agent_memory_consolidation_global_slow_pass.md");
const PROJECT_SLOW_TEMPLATE: &str =
    include_str!("../../service/memory_store/prompts/agent_memory_consolidation_project_slow_pass.md");

/// Turn budget for slow-pass forks.
pub(crate) const SLOW_PASS_TURN_BUDGET: usize = 6;

// ---------------------------------------------------------------------------
// Global slow pass
// ---------------------------------------------------------------------------

/// Build the global slow-pass prompt.
///
/// `global_memory_dir`         — absolute path to the global memory directory  
/// `all_workspace_memory_dirs` — display paths of each workspace memory directory  
///                               (the fork will read sessions/ from each of these)  
pub(crate) fn build_global_slow_pass_prompt(
    global_memory_dir: &str,
    all_workspace_memory_dirs: &[String],
) -> String {
    let workspace_dirs_list = if all_workspace_memory_dirs.is_empty() {
        "(none registered)".to_string()
    } else {
        all_workspace_memory_dirs
            .iter()
            .map(|d| format!("  - {}", d))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let turn_budget = SLOW_PASS_TURN_BUDGET.to_string();

    GLOBAL_SLOW_TEMPLATE
        .replace("{global_memory_dir}", global_memory_dir)
        .replace("{workspace_dirs_list}", &workspace_dirs_list)
        .replace("{turn_budget}", &turn_budget)
}

/// Build the runtime tool restrictions for a global slow-pass fork.
///
/// Read roots are unrestricted (the fork needs to read workspace sessions).
/// Write roots are restricted to the global memory directory.
pub(crate) fn build_global_slow_pass_restrictions(
    global_memory_dir: &str,
) -> crate::agentic::tools::ToolRuntimeRestrictions {
    use crate::agentic::tools::{ToolPathPolicy, ToolRuntimeRestrictions};
    use std::collections::BTreeSet;
    ToolRuntimeRestrictions {
        allowed_tool_names: ["Read", "Glob", "Grep", "Write", "Edit", "Delete"]
            .into_iter()
            .map(str::to_string)
            .collect::<BTreeSet<_>>(),
        denied_tool_names: BTreeSet::new(),
        path_policy: ToolPathPolicy {
            write_roots: vec![global_memory_dir.to_string()],
            edit_roots: vec![global_memory_dir.to_string()],
            delete_roots: vec![global_memory_dir.to_string()],
        },
        disable_snapshot_tracking: true,
    }
}

// ---------------------------------------------------------------------------
// Project slow pass
// ---------------------------------------------------------------------------

/// Build the project slow-pass prompt.
///
/// `project_memory_dir` — absolute path to the workspace memory directory  
pub(crate) fn build_project_slow_pass_prompt(project_memory_dir: &str) -> String {
    let turn_budget = SLOW_PASS_TURN_BUDGET.to_string();
    PROJECT_SLOW_TEMPLATE
        .replace("{project_memory_dir}", project_memory_dir)
        .replace("{turn_budget}", &turn_budget)
}

/// Build the runtime tool restrictions for a project slow-pass fork.
pub(crate) fn build_project_slow_pass_restrictions(
    project_memory_dir: &str,
) -> crate::agentic::tools::ToolRuntimeRestrictions {
    build_auto_memory_runtime_restrictions(project_memory_dir)
}
