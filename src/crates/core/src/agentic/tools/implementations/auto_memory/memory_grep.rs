//! Memory-only wrapper around `GrepTool`. Forces the `path` field to be
//! present (the underlying tool defaults to ".", which would escape memory
//! scope inside the auto-memory fork).

use super::path_guard::{ensure_memory_path_required, PathField};
use crate::agentic::tools::framework::{
    Tool, ToolRenderOptions, ToolResult, ToolUseContext, ValidationResult,
};
use crate::agentic::tools::implementations::GrepTool;
use crate::util::errors::{BitFunError, BitFunResult};
use async_trait::async_trait;
use serde_json::Value;

pub struct MemoryGrepTool {
    inner: GrepTool,
}

impl Default for MemoryGrepTool {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryGrepTool {
    pub fn new() -> Self {
        Self {
            inner: GrepTool::new(),
        }
    }
}

#[async_trait]
impl Tool for MemoryGrepTool {
    fn name(&self) -> &str {
        "MemoryGrep"
    }

    async fn description(&self) -> BitFunResult<String> {
        Ok(
            "Search memory files with ripgrep. The `path` field is REQUIRED (no implicit cwd) \
             and must resolve inside one of the auto-memory fork's allowed memory roots; \
             searches outside those roots are refused at the tool layer. Same options as `Grep`."
                .to_string(),
        )
    }

    fn input_schema(&self) -> Value {
        // Reuse the inner schema but mark `path` as required for memory tools.
        let mut schema = self.inner.input_schema();
        if let Some(required) = schema.get_mut("required").and_then(|v| v.as_array_mut()) {
            if !required.iter().any(|item| item.as_str() == Some("path")) {
                required.push(Value::String("path".to_string()));
            }
        }
        schema
    }

    fn is_readonly(&self) -> bool {
        true
    }

    fn is_concurrency_safe(&self, _input: Option<&Value>) -> bool {
        true
    }

    fn needs_permissions(&self, _input: Option<&Value>) -> bool {
        false
    }

    async fn validate_input(
        &self,
        input: &Value,
        context: Option<&ToolUseContext>,
    ) -> ValidationResult {
        if let Some(ctx) = context {
            if let Err(message) = ensure_memory_path_required(input, ctx, PathField::PATH) {
                return ValidationResult {
                    result: false,
                    message: Some(message),
                    error_code: Some(403),
                    meta: None,
                };
            }
        }
        self.inner.validate_input(input, context).await
    }

    fn render_tool_use_message(&self, input: &Value, options: &ToolRenderOptions) -> String {
        format!("Memory: {}", self.inner.render_tool_use_message(input, options))
    }

    async fn call_impl(
        &self,
        input: &Value,
        context: &ToolUseContext,
    ) -> BitFunResult<Vec<ToolResult>> {
        ensure_memory_path_required(input, context, PathField::PATH)
            .map_err(BitFunError::tool)?;
        self.inner.call_impl(input, context).await
    }
}
