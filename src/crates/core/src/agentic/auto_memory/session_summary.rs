//! Session summary fork pipeline.
//!
//! When a session becomes idle (or is closed), a lightweight fork agent
//! writes a session summary to `sessions/YYYY-MM-DD-<session-id>.md`.
//!
//! Turn budget: 2 — turn 1 reads existing file (if any), turn 2 writes.
//! Write roots restricted to `<memory_dir>/sessions/`.
//!
//! Output format:
//!
//! ```markdown
//! ---
//! layer: session
//! created: <ISO 8601>
//! source_session: <session-id>
//! status: confirmed
//! tags: [...]
//! ---
//!
//! # <Short title (≤10 words)>
//!
//! <3–5 sentence summary>
//!
//! ## Unfinished items
//! - ...
//!
//! ## Related episodes
//! - episodes/YYYY-MM/YYYY-MM-DD-<slug>.md
//! ```

use chrono::Utc;

const SESSION_SUMMARY_TEMPLATE: &str =
    include_str!("../../service/memory_store/prompts/agent_memory_session_summary.md");

/// Maximum number of turns for the session summary fork.
///
/// Turn 1: read existing target file (if any) and any recently-written
///         tentative episodes from this session in parallel.
/// Turn 2: write/update the session summary file, and promote
///         `status: tentative` → `status: confirmed` on episodes that
///         survived the session.
pub const SESSION_SUMMARY_TURN_BUDGET: usize = 3;

/// Minimum idle time in seconds before a session qualifies for summary.
pub const SESSION_IDLE_THRESHOLD_SECS: u64 = 120;

/// Build the prompt for the session summary fork.
#[allow(dead_code)]
///
/// `session_id`     — parent session identifier  
/// `memory_dir`     — absolute path to the memory directory (display form)  
/// `sessions_dir`   — absolute path to the sessions/ subdirectory  
pub fn build_session_summary_prompt(
    session_id: &str,
    memory_dir: &str,
    sessions_dir: &str,
) -> String {
    let date_str = Utc::now().format("%Y-%m-%d").to_string();
    let target_file = format!("{}/{}-{}.md", sessions_dir, date_str, session_id);

    let episodes_dir = format!("{}/episodes", memory_dir);
    let turn_budget = SESSION_SUMMARY_TURN_BUDGET.to_string();

    SESSION_SUMMARY_TEMPLATE
        .replace("{target_file}", &target_file)
        .replace("{sessions_dir}", sessions_dir)
        .replace("{episodes_dir}", &episodes_dir)
        .replace("{turn_budget}", &turn_budget)
        .replace("{date_iso}", &date_str)
        .replace("{session_id}", session_id)
        .replace("{memory_dir}", memory_dir)
}
