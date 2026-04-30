mod manager;
mod orchestrator;
mod planning;
mod prompt;
mod restrictions;
pub(crate) mod session_summary;
mod types;

pub use manager::AutoMemoryManager;
pub use orchestrator::{handle_auto_memory_after_completed_turn, AutoMemoryCompletedTurnFollowup};
pub use planning::{
    auto_memory_runtime_config, auto_memory_scope_config, auto_memory_scope_runtime_config,
    auto_memory_throttle_policy, decide_auto_memory_post_turn_action,
    queue_action_from_schedule_decision, resolve_auto_memory_runtime_context,
    resolve_auto_memory_scope, resolve_local_auto_memory_context,
    resolve_session_auto_memory_scope, session_can_consider_auto_memory, AutoMemoryPostTurnAction,
    ResolvedAutoMemoryContext, ResolvedAutoMemoryRuntimeContext,
};
pub use prompt::{
    build_extract_prompt, build_extract_prompt_with_global, count_recent_model_visible_messages,
};
pub use restrictions::{
    build_auto_memory_runtime_restrictions, build_auto_memory_runtime_restrictions_with_extra_roots,
};
pub use session_summary::{
    build_session_summary_prompt, SESSION_IDLE_THRESHOLD_SECS, SESSION_SUMMARY_TURN_BUDGET,
};
pub use types::{
    AutoMemoryExtractionCursor, AutoMemoryQueueAction, AutoMemoryReadyReason,
    AutoMemoryScheduleDecision, AutoMemoryState, AutoMemoryThrottlePolicy,
};
