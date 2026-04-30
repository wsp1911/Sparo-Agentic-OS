//! Tauri commands for the memory store control panel.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

use bitfun_core::agentic::coordination::{
    DialogScheduler, DialogSubmissionPolicy, DialogTriggerSource,
};
use bitfun_core::service::memory_store::{
    api::{
        archive_entry, delete_entry, forget_by_tag, list_entries, read_entry, rebuild_index,
        record_memory_hit, run_repair, update_entry, ConsolidationKind, ListEntriesFilter,
        MemoryEntryDetail, MemoryEntrySummary,
    },
    MemoryStoreTarget,
};

use crate::api::AppState;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn project_target(workspace_path: &str) -> MemoryStoreTarget<'_> {
    MemoryStoreTarget::WorkspaceProject(Path::new(workspace_path))
}

fn target_for_scope<'a>(
    scope: &MemoryScopeParam,
    workspace_path: Option<&'a str>,
) -> Result<MemoryStoreTarget<'a>, String> {
    match scope {
        MemoryScopeParam::Global => Ok(MemoryStoreTarget::GlobalAgenticOs),
        MemoryScopeParam::Workspace => {
            let path = workspace_path
                .map(str::trim)
                .filter(|path| !path.is_empty())
                .ok_or_else(|| {
                    "workspacePath is required for workspace memory operations".to_string()
                })?;
            Ok(project_target(path))
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MemoryScopeParam {
    Global,
    Workspace,
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryListRequest {
    pub scope: MemoryScopeParam,
    pub workspace_path: Option<String>,
    pub filter: Option<ListEntriesFilter>,
}

#[tauri::command]
pub async fn memory_list_entries(
    _state: State<'_, AppState>,
    request: MemoryListRequest,
) -> Result<Vec<MemoryEntrySummary>, String> {
    let target = target_for_scope(&request.scope, request.workspace_path.as_deref())?;
    let filter = request.filter.unwrap_or_default();
    list_entries(target, filter)
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryReadRequest {
    pub scope: MemoryScopeParam,
    pub workspace_path: Option<String>,
    pub relative_path: String,
}

#[tauri::command]
pub async fn memory_read_entry(
    _state: State<'_, AppState>,
    request: MemoryReadRequest,
) -> Result<MemoryEntryDetail, String> {
    let target = target_for_scope(&request.scope, request.workspace_path.as_deref())?;
    read_entry(target, &request.relative_path)
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryUpdateRequest {
    pub scope: MemoryScopeParam,
    pub workspace_path: Option<String>,
    pub relative_path: String,
    pub content: String,
}

#[tauri::command]
pub async fn memory_update_entry(
    _state: State<'_, AppState>,
    request: MemoryUpdateRequest,
) -> Result<(), String> {
    let target = target_for_scope(&request.scope, request.workspace_path.as_deref())?;
    update_entry(target, &request.relative_path, &request.content)
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryArchiveRequest {
    pub scope: MemoryScopeParam,
    pub workspace_path: Option<String>,
    pub relative_path: String,
}

#[tauri::command]
pub async fn memory_archive_entry(
    _state: State<'_, AppState>,
    request: MemoryArchiveRequest,
) -> Result<(), String> {
    let target = target_for_scope(&request.scope, request.workspace_path.as_deref())?;
    archive_entry(target, &request.relative_path)
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryDeleteRequest {
    pub scope: MemoryScopeParam,
    pub workspace_path: Option<String>,
    pub relative_path: String,
}

#[tauri::command]
pub async fn memory_delete_entry(
    _state: State<'_, AppState>,
    request: MemoryDeleteRequest,
) -> Result<(), String> {
    let target = target_for_scope(&request.scope, request.workspace_path.as_deref())?;
    delete_entry(target, &request.relative_path)
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryForgetByTagRequest {
    pub scope: MemoryScopeParam,
    pub workspace_path: Option<String>,
    pub tag: String,
}

#[tauri::command]
pub async fn memory_forget_by_tag(
    _state: State<'_, AppState>,
    request: MemoryForgetByTagRequest,
) -> Result<usize, String> {
    let target = target_for_scope(&request.scope, request.workspace_path.as_deref())?;
    forget_by_tag(target, &request.tag)
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryTriggerConsolidationRequest {
    /// The session that will receive the consolidation command.
    pub session_id: String,
    pub workspace_path: Option<String>,
    pub kind: ConsolidationKind,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryTriggerConsolidationResponse {
    pub queued: bool,
}

#[tauri::command]
pub async fn memory_trigger_consolidation(
    _state: State<'_, AppState>,
    scheduler: State<'_, Arc<DialogScheduler>>,
    request: MemoryTriggerConsolidationRequest,
) -> Result<MemoryTriggerConsolidationResponse, String> {
    use bitfun_core::agentic::memory_consolidation::{
        MID_CONSOLIDATION_COMMAND, SLOW_CONSOLIDATION_COMMAND_GLOBAL,
        SLOW_CONSOLIDATION_COMMAND_PROJECT,
    };

    let command = match request.kind {
        ConsolidationKind::Mid => MID_CONSOLIDATION_COMMAND,
        ConsolidationKind::SlowGlobal => SLOW_CONSOLIDATION_COMMAND_GLOBAL,
        ConsolidationKind::SlowProject => SLOW_CONSOLIDATION_COMMAND_PROJECT,
    };

    let policy = DialogSubmissionPolicy::for_source(DialogTriggerSource::DesktopUi)
        .with_persist_agent_type(false);

    scheduler
        .submit(
            request.session_id,
            command.to_owned(),
            None,
            None,
            "auto".to_string(),
            None,
            request.workspace_path,
            policy,
            None,
            None,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(MemoryTriggerConsolidationResponse { queued: true })
}

// ---------------------------------------------------------------------------
// Index rebuild
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryRebuildIndexRequest {
    pub scope: MemoryScopeParam,
    pub workspace_path: Option<String>,
}

#[tauri::command]
pub async fn memory_rebuild_index(
    _state: State<'_, AppState>,
    request: MemoryRebuildIndexRequest,
) -> Result<(), String> {
    let target = target_for_scope(&request.scope, request.workspace_path.as_deref())?;
    rebuild_index(target).await.map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Repair
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryRunRepairRequest {
    pub scope: MemoryScopeParam,
    pub workspace_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryRunRepairResponse {
    pub actions: Vec<String>,
    pub errors: Vec<String>,
}

#[tauri::command]
pub async fn memory_run_repair(
    _state: State<'_, AppState>,
    request: MemoryRunRepairRequest,
) -> Result<MemoryRunRepairResponse, String> {
    let target = target_for_scope(&request.scope, request.workspace_path.as_deref())?;
    let report = run_repair(target).await;
    Ok(MemoryRunRepairResponse {
        actions: report.actions,
        errors: report.errors,
    })
}

// ---------------------------------------------------------------------------
// Hit tracking
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryRecordHitRequest {
    pub scope: MemoryScopeParam,
    pub workspace_path: Option<String>,
    pub relative_path: String,
}

#[tauri::command]
pub async fn memory_record_hit(
    _state: State<'_, AppState>,
    request: MemoryRecordHitRequest,
) -> Result<(), String> {
    let target = target_for_scope(&request.scope, request.workspace_path.as_deref())?;
    record_memory_hit(target, &request.relative_path)
        .await
        .map_err(|e| e.to_string())
}
