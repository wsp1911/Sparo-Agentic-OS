//! Memory consolidation module.
//!
//! Provides background "dreaming" passes that keep the memory store lean and
//! accurate:
//!
//! - **Fast pass** — handled by `agentic/auto_memory` (unchanged).
//! - **Mid pass** (daily) — clusters episodes, detects conflicts, decays weak
//!   entries, writes abstraction proposals.
//! - **Slow pass** (monthly) — rewrites core narrative/persona/project files
//!   for both global and workspace scopes.
//!
//! The mid/slow passes are triggered by the `agentic/memory_consolidation`
//! cron jobs registered by `schedule::register_consolidation_jobs`.  The
//! `ConversationCoordinator` intercepts the magic command strings and calls
//! the appropriate cycle method instead of running a regular LLM turn.

pub mod decay;
pub(crate) mod executor;
pub(crate) mod mid_pass;
pub(crate) mod schedule;
pub(crate) mod slow_pass;

pub use decay::LifecycleConfig;
pub(crate) use executor::{run_lifecycle_pass, LifecyclePassSummary};
pub(crate) use mid_pass::{
    build_mid_pass_prompt, build_mid_pass_restrictions, MID_PASS_TURN_BUDGET,
};
pub use schedule::{
    register_consolidation_jobs, MID_CONSOLIDATION_COMMAND, SLOW_CONSOLIDATION_COMMAND_GLOBAL,
    SLOW_CONSOLIDATION_COMMAND_PROJECT,
};
pub(crate) use slow_pass::{
    build_global_slow_pass_prompt, build_global_slow_pass_restrictions,
    build_project_slow_pass_prompt, build_project_slow_pass_restrictions, SLOW_PASS_TURN_BUDGET,
};
