//! Mid-pass consolidation prompt.
//!
//! The mid-pass fork agent:
//! - Clusters similar episodic entries (merging near-duplicates, linking the originals)
//! - Abstracts recurring patterns into habits.md / persona.md / project.md update proposals
//!   (written as draft files to archive/proposals/ rather than directly overwriting core files)
//! - Detects contradictions between entries (marks older entry archived, links from newer)
//!
//! Mechanical decay and archive-threshold moves are handled by the deterministic
//! Rust pre-pass before this prompt runs.

use super::decay::DecayConfig;
use crate::agentic::auto_memory::build_auto_memory_runtime_restrictions;
use crate::service::memory_store::layout::ARCHIVE_DIR;

const MID_PASS_TEMPLATE: &str =
    include_str!("../../service/memory_store/prompts/agent_memory_consolidation_mid_pass.md");

/// Turn budget for the mid-pass fork.
pub(crate) const MID_PASS_TURN_BUDGET: usize = 4;

/// Build the mid-pass consolidation prompt.
///
/// `memory_dir`      — absolute path to the memory directory (display form)  
/// `_decay_config`   — current decay configuration, handled by the pre-pass  
/// `scope_label`     — "workspace" or "global"  
pub(crate) fn build_mid_pass_prompt(
    memory_dir: &str,
    _decay_config: &DecayConfig,
    scope_label: &str,
) -> String {
    let proposals_dir = format!("{}/{}/proposals", memory_dir, ARCHIVE_DIR);
    let turn_budget = MID_PASS_TURN_BUDGET.to_string();

    MID_PASS_TEMPLATE
        .replace("{memory_dir}", memory_dir)
        .replace("{proposals_dir}", &proposals_dir)
        .replace("{scope_label}", scope_label)
        .replace("{turn_budget}", &turn_budget)
}

/// Build the runtime tool restrictions for a mid-pass fork.
pub(crate) fn build_mid_pass_restrictions(
    memory_dir: &str,
) -> crate::agentic::tools::ToolRuntimeRestrictions {
    build_auto_memory_runtime_restrictions(memory_dir)
}
