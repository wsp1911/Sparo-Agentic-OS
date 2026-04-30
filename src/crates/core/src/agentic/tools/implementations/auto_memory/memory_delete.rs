//! Memory-only wrapper around `DeleteFileTool`.

use super::path_guard::{ensure_memory_path_required, PathField};
use crate::agentic::tools::framework::{
    Tool, ToolRenderOptions, ToolResult, ToolUseContext, ValidationResult,
};
use crate::agentic::tools::implementations::DeleteFileTool;
use crate::util::errors::{BitFunError, BitFunResult};
use async_trait::async_trait;
use serde_json::Value;

pub struct MemoryDeleteTool {
    inner: DeleteFileTool,
}

impl Default for MemoryDeleteTool {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryDeleteTool {
    pub fn new() -> Self {
        Self {
            inner: DeleteFileTool::new(),
        }
    }
}

#[async_trait]
impl Tool for MemoryDeleteTool {
    fn name(&self) -> &str {
        "MemoryDelete"
    }

    async fn description(&self) -> BitFunResult<String> {
        Ok(
            "Delete a memory file. The `path` must be absolute and resolve inside one of the \
             auto-memory fork's allowed memory roots; deletions outside those roots are refused \
             at the tool layer. Same usage as `Delete`."
                .to_string(),
        )
    }

    fn input_schema(&self) -> Value {
        self.inner.input_schema()
    }

    fn is_readonly(&self) -> bool {
        false
    }

    fn is_concurrency_safe(&self, input: Option<&Value>) -> bool {
        self.inner.is_concurrency_safe(input)
    }

    fn needs_permissions(&self, input: Option<&Value>) -> bool {
        self.inner.needs_permissions(input)
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
