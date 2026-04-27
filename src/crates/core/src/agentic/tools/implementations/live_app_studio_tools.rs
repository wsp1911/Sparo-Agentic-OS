//! Live App Studio tools — recompile, runtime probe, and review matrix.

use crate::agentic::tools::framework::{Tool, ToolResult, ToolUseContext};
use crate::infrastructure::events::{emit_global_event, BackendEvent};
use crate::live_app::try_get_global_live_app_manager;
use crate::live_app::types::{LiveAppRuntimeIssue, LiveAppRuntimeIssueSeverity};
use crate::util::errors::{BitFunError, BitFunResult};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

pub struct LiveAppRecompileTool;

impl LiveAppRecompileTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LiveAppRecompileTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for LiveAppRecompileTool {
    fn name(&self) -> &str {
        "LiveAppRecompile"
    }

    async fn description(&self) -> BitFunResult<String> {
        Ok("Sync a Live App from its source files, recompile compiled_html, and emit update events for the right-side preview.".to_string())
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["app_id"],
            "properties": {
                "app_id": { "type": "string", "description": "Live App id" },
                "theme": { "type": "string", "description": "Theme type, default dark" }
            }
        })
    }

    fn is_readonly(&self) -> bool {
        false
    }

    fn needs_permissions(&self, _input: Option<&Value>) -> bool {
        false
    }

    async fn call_impl(
        &self,
        input: &Value,
        context: &ToolUseContext,
    ) -> BitFunResult<Vec<ToolResult>> {
        let app_id = required_string(input, "app_id")?;
        let theme = input.get("theme").and_then(Value::as_str).unwrap_or("dark");
        let manager = try_get_global_live_app_manager()
            .ok_or_else(|| BitFunError::tool("LiveAppManager not initialized".to_string()))?;

        let app = manager
            .sync_from_fs(app_id, theme, context.workspace_root())
            .await
            .map_err(|e| {
                BitFunError::tool(format!("Failed to sync and recompile Live App: {e}"))
            })?;

        let payload = json!({
            "id": app.id,
            "name": app.name,
            "reason": "studio-recompile",
        });
        let _ = emit_global_event(BackendEvent::Custom {
            event_name: "liveapp-recompiled".to_string(),
            payload: payload.clone(),
        })
        .await;
        let _ = emit_global_event(BackendEvent::Custom {
            event_name: "liveapp-updated".to_string(),
            payload,
        })
        .await;

        let data = json!({
            "ok": true,
            "app_id": app.id,
            "version": app.version,
            "compiled_html_size": app.compiled_html.len(),
            "warnings": [],
        });
        Ok(vec![ToolResult::ok(
            data,
            Some(format!(
                "Live App '{}' synced and recompiled. compiled_html_size={}",
                app.name,
                app.compiled_html.len()
            )),
        )])
    }
}

pub struct LiveAppRuntimeProbeTool;

impl LiveAppRuntimeProbeTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LiveAppRuntimeProbeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for LiveAppRuntimeProbeTool {
    fn name(&self) -> &str {
        "LiveAppRuntimeProbe"
    }

    async fn description(&self) -> BitFunResult<String> {
        Ok("Read recent runtime issues reported by a Live App iframe and bridge calls, grouped by severity.".to_string())
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["app_id"],
            "properties": {
                "app_id": { "type": "string", "description": "Live App id" },
                "since_ms": { "type": "integer", "description": "Only include issues with timestamp >= this Unix milliseconds value" },
                "include_noise": { "type": "boolean", "description": "Return noise issues instead of just a count" }
            }
        })
    }

    fn is_readonly(&self) -> bool {
        true
    }

    async fn call_impl(
        &self,
        input: &Value,
        _context: &ToolUseContext,
    ) -> BitFunResult<Vec<ToolResult>> {
        let app_id = required_string(input, "app_id")?;
        let since_ms = input.get("since_ms").and_then(Value::as_i64);
        let include_noise = input
            .get("include_noise")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let manager = try_get_global_live_app_manager()
            .ok_or_else(|| BitFunError::tool("LiveAppManager not initialized".to_string()))?;

        let issues = manager.runtime_issues(app_id, since_ms).await;
        let fatal: Vec<&LiveAppRuntimeIssue> = issues
            .iter()
            .filter(|issue| issue.severity == LiveAppRuntimeIssueSeverity::Fatal)
            .collect();
        let warning: Vec<&LiveAppRuntimeIssue> = issues
            .iter()
            .filter(|issue| issue.severity == LiveAppRuntimeIssueSeverity::Warning)
            .collect();
        let noise: Vec<&LiveAppRuntimeIssue> = issues
            .iter()
            .filter(|issue| issue.severity == LiveAppRuntimeIssueSeverity::Noise)
            .collect();

        let data = json!({
            "app_id": app_id,
            "fatal": fatal,
            "warning": warning,
            "noise_count": noise.len(),
            "noise": if include_noise { json!(noise) } else { Value::Null },
            "ok": fatal.is_empty(),
        });
        Ok(vec![ToolResult::ok(
            data,
            Some(format!(
                "Runtime probe: {} fatal, {} warning, {} noise",
                fatal.len(),
                warning.len(),
                noise.len()
            )),
        )])
    }
}

pub struct LiveAppScreenshotMatrixTool;

impl LiveAppScreenshotMatrixTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LiveAppScreenshotMatrixTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for LiveAppScreenshotMatrixTool {
    fn name(&self) -> &str {
        "LiveAppScreenshotMatrix"
    }

    async fn description(&self) -> BitFunResult<String> {
        Ok("Prepare a 4-state Live App visual review matrix for light/dark and zh-CN/en-US, and notify the UI to capture screenshots when available.".to_string())
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["app_id"],
            "properties": {
                "app_id": { "type": "string", "description": "Live App id" }
            }
        })
    }

    fn is_readonly(&self) -> bool {
        false
    }

    fn needs_permissions(&self, _input: Option<&Value>) -> bool {
        false
    }

    async fn call_impl(
        &self,
        input: &Value,
        _context: &ToolUseContext,
    ) -> BitFunResult<Vec<ToolResult>> {
        let app_id = required_string(input, "app_id")?;
        let manager = try_get_global_live_app_manager()
            .ok_or_else(|| BitFunError::tool("LiveAppManager not initialized".to_string()))?;
        let app = manager
            .get(app_id)
            .await
            .map_err(|e| BitFunError::tool(format!("Failed to load Live App: {e}")))?;
        let timestamp = Utc::now().timestamp_millis();
        let review_dir = manager
            .path_manager()
            .live_app_dir(app_id)
            .join("_review")
            .join(timestamp.to_string());
        tokio::fs::create_dir_all(&review_dir).await?;

        let states = [
            ("light", "zh-CN"),
            ("light", "en-US"),
            ("dark", "zh-CN"),
            ("dark", "en-US"),
        ];
        let screenshots: Vec<Value> = states
            .iter()
            .map(|(theme, locale)| {
                json!({
                    "theme": theme,
                    "locale": locale,
                    "path": Value::Null,
                    "status": "capture_requested",
                })
            })
            .collect();
        let manifest = json!({
            "app_id": app.id,
            "app_name": app.name,
            "created_at": timestamp,
            "status": "capture_requested",
            "screenshots": screenshots,
        });
        let manifest_path = review_dir.join("manifest.json");
        tokio::fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?).await?;

        let payload = json!({
            "appId": app_id,
            "manifestPath": manifest_path.to_string_lossy(),
            "reviewDir": review_dir.to_string_lossy(),
            "states": states.iter().map(|(theme, locale)| json!({ "theme": theme, "locale": locale })).collect::<Vec<_>>(),
        });
        let _ = emit_global_event(BackendEvent::Custom {
            event_name: "liveapp-screenshot-matrix-requested".to_string(),
            payload,
        })
        .await;

        let data = json!({
            "manifest_path": manifest_path.to_string_lossy(),
            "screenshots": screenshots,
            "status": "capture_requested",
        });
        Ok(vec![ToolResult::ok(
            data,
            Some(format!(
                "Screenshot matrix requested for Live App '{}'. Manifest: {}",
                app.name,
                manifest_path.to_string_lossy()
            )),
        )])
    }
}

fn required_string<'a>(input: &'a Value, field: &str) -> BitFunResult<&'a str> {
    input
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| BitFunError::validation(format!("Missing required field: {field}")))
}
