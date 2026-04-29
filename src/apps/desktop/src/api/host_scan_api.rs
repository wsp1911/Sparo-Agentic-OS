//! Host scan API
//!
//! Desktop adapter for `/scan_host`.
//!
//! `/scan_host` runs as a hidden transient child session shown in the side
//! panel. The frontend creates a temporary session shell, while the backend
//! owns the actual prompt/reminder construction and starts the hidden turn.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

use bitfun_core::agentic::coordination::ConversationCoordinator;
use bitfun_core::service::{get_global_host_auto_scan_service, HostScanTrigger};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartHostScanRequest {
    pub request_id: String,
    pub parent_session_id: String,
    pub child_session_id: String,
    pub child_session_name: Option<String>,
    /// Optional model id override. Supports "fast"/"primary" aliases.
    pub model_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartHostScanResponse {
    pub ok: bool,
}

#[tauri::command]
pub async fn start_host_scan_stream(
    coordinator: State<'_, Arc<ConversationCoordinator>>,
    request: StartHostScanRequest,
) -> Result<StartHostScanResponse, String> {
    if request.request_id.trim().is_empty() {
        return Err("requestId is required".to_string());
    }
    if request.parent_session_id.trim().is_empty() {
        return Err("parentSessionId is required".to_string());
    }
    if request.child_session_id.trim().is_empty() {
        return Err("childSessionId is required".to_string());
    }

    let turn_id = coordinator
        .start_hidden_host_scan_turn(
            &request.request_id,
            &request.parent_session_id,
            &request.child_session_id,
            request.child_session_name.as_deref(),
            request.model_id.as_deref(),
        )
        .await
        .map_err(|error| error.to_string())?;

    if let Some(service) = get_global_host_auto_scan_service() {
        service
            .register_scan_turn(&turn_id, HostScanTrigger::Manual)
            .await
            .map_err(|error| error.to_string())?;
    }

    Ok(StartHostScanResponse { ok: true })
}
