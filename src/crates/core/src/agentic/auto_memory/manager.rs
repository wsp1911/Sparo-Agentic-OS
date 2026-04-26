use crate::agentic::coordination::ConversationCoordinator;
use dashmap::DashMap;
use log::{debug, warn};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
struct WorkspaceRunState {
    worker_running: bool,
    pending_sessions: HashSet<String>,
    known_sessions: HashSet<String>,
    active_session_id: Option<String>,
    active_session_cancellation_token: Option<CancellationToken>,
}

pub struct AutoMemoryManager {
    workspace_runs: Arc<DashMap<String, WorkspaceRunState>>,
    session_workspace_keys: Arc<DashMap<String, String>>,
    global_semaphore: Arc<Semaphore>,
}

impl AutoMemoryManager {
    pub fn new() -> Self {
        Self {
            workspace_runs: Arc::new(DashMap::new()),
            session_workspace_keys: Arc::new(DashMap::new()),
            global_semaphore: Arc::new(Semaphore::new(1)),
        }
    }

    pub fn schedule_after_turn(
        self: &Arc<Self>,
        coordinator: Arc<ConversationCoordinator>,
        session_id: String,
        workspace_key: String,
    ) {
        self.session_workspace_keys
            .insert(session_id.clone(), workspace_key.clone());

        let (should_spawn, pending_session_count, active_session_id) = {
            let mut should_spawn = false;
            if let Some(mut existing) = self.workspace_runs.get_mut(&workspace_key) {
                existing.pending_sessions.insert(session_id.clone());
                existing.known_sessions.insert(session_id.clone());
                let pending_session_count = existing.pending_sessions.len();
                let active_session_id = existing.active_session_id.clone();
                if !existing.worker_running {
                    existing.worker_running = true;
                    should_spawn = true;
                }
                (should_spawn, pending_session_count, active_session_id)
            } else {
                let mut pending_sessions = HashSet::new();
                pending_sessions.insert(session_id.clone());
                let mut known_sessions = HashSet::new();
                known_sessions.insert(session_id.clone());
                self.workspace_runs.insert(
                    workspace_key.clone(),
                    WorkspaceRunState {
                        worker_running: true,
                        pending_sessions,
                        known_sessions,
                        active_session_id: None,
                        active_session_cancellation_token: None,
                    },
                );
                should_spawn = true;
                (should_spawn, 1, None)
            }
        };

        debug!(
            "Scheduled auto memory session: session_id={}, workspace_key={}, worker_spawned={}, pending_sessions={}, active_session_id={}",
            session_id,
            workspace_key,
            should_spawn,
            pending_session_count,
            active_session_id.as_deref().unwrap_or("<none>")
        );

        if !should_spawn {
            return;
        }

        let manager = self.clone();
        tokio::spawn(async move {
            manager.run_workspace_loop(coordinator, workspace_key).await;
        });
    }

    pub fn cancel_session(&self, session_id: &str) {
        let Some((_, workspace_key)) = self.session_workspace_keys.remove(session_id) else {
            return;
        };

        if let Some(mut state) = self.workspace_runs.get_mut(&workspace_key) {
            state.pending_sessions.remove(session_id);
            if state.active_session_id.as_deref() == Some(session_id) {
                if let Some(token) = state.active_session_cancellation_token.as_ref() {
                    token.cancel();
                }
                debug!(
                    "Cancelled active auto memory session run: session_id={}, workspace_key={}",
                    session_id, workspace_key
                );
            } else {
                debug!(
                    "Removed pending auto memory session before execution: session_id={}, workspace_key={}, pending_sessions_remaining={}",
                    session_id,
                    workspace_key,
                    state.pending_sessions.len()
                );
            }
        }
    }

    async fn run_workspace_loop(
        self: Arc<Self>,
        coordinator: Arc<ConversationCoordinator>,
        workspace_key: String,
    ) {
        debug!(
            "Started auto memory workspace worker: workspace_key={}",
            workspace_key
        );

        loop {
            let Some((session_id, cancellation_token)) =
                self.start_next_session_run(&workspace_key)
            else {
                break;
            };

            let permit = self.global_semaphore.clone().acquire_owned().await;
            let run_result = match permit {
                Ok(_permit) => {
                    coordinator
                        .run_auto_memory_cycle(&session_id, &cancellation_token)
                        .await
                }
                Err(err) => {
                    warn!(
                        "Auto memory global semaphore closed unexpectedly: workspace_key={}, session_id={}, error={}",
                        workspace_key, session_id, err
                    );
                    self.finish_session_run(&workspace_key, &session_id, false);
                    continue;
                }
            };

            let mut executed = false;
            match run_result {
                Ok(did_run) => {
                    executed = did_run;
                }
                Err(crate::util::errors::BitFunError::Cancelled(_)) => {
                    debug!(
                        "Auto memory cycle cancelled: workspace_key={}, session_id={}",
                        workspace_key, session_id
                    );
                }
                Err(err) => {
                    warn!(
                        "Auto memory cycle failed: workspace_key={}, session_id={}, error={}",
                        workspace_key, session_id, err
                    );
                }
            }

            let should_requeue = if cancellation_token.is_cancelled() {
                false
            } else {
                coordinator.has_pending_auto_memory(&session_id).await
            };
            self.finish_session_run(&workspace_key, &session_id, should_requeue);

            debug!(
                "Finished auto memory session run: workspace_key={}, session_id={}, executed={}, requeued={}",
                workspace_key, session_id, executed, should_requeue
            );
        }

        debug!(
            "Stopped auto memory workspace worker: workspace_key={}",
            workspace_key
        );
    }

    fn start_next_session_run(&self, workspace_key: &str) -> Option<(String, CancellationToken)> {
        let mut state = self.workspace_runs.get_mut(workspace_key)?;
        let Some(session_id) = state.pending_sessions.iter().next().cloned() else {
            state.worker_running = false;
            state.active_session_id = None;
            state.active_session_cancellation_token = None;
            state.known_sessions.clear();
            return None;
        };
        state.pending_sessions.remove(&session_id);
        let cancellation_token = CancellationToken::new();
        state.active_session_id = Some(session_id.clone());
        state.active_session_cancellation_token = Some(cancellation_token.clone());
        debug!(
            "Starting auto memory session run: workspace_key={}, session_id={}, pending_sessions_remaining={}",
            workspace_key,
            session_id,
            state.pending_sessions.len()
        );
        Some((session_id, cancellation_token))
    }

    fn finish_session_run(&self, workspace_key: &str, session_id: &str, should_requeue: bool) {
        if let Some(mut state) = self.workspace_runs.get_mut(workspace_key) {
            if state.active_session_id.as_deref() == Some(session_id) {
                state.active_session_id = None;
                state.active_session_cancellation_token = None;
            }
            if should_requeue {
                state.pending_sessions.insert(session_id.to_string());
                state.known_sessions.insert(session_id.to_string());
            }
            debug!(
                "Updated auto memory workspace run state: workspace_key={}, session_id={}, should_requeue={}, pending_sessions={}",
                workspace_key,
                session_id,
                should_requeue,
                state.pending_sessions.len()
            );
        }
    }
}
