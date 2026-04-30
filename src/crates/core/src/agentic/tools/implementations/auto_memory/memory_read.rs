//! Memory-only wrapper around `FileReadTool`.

use super::path_guard::{ensure_memory_path_required, PathField};
use crate::agentic::tools::framework::{
    Tool, ToolRenderOptions, ToolResult, ToolUseContext, ValidationResult,
};
use crate::agentic::tools::implementations::FileReadTool;
use crate::util::errors::{BitFunError, BitFunResult};
use async_trait::async_trait;
use serde_json::Value;

pub struct MemoryReadTool {
    inner: FileReadTool,
}

impl Default for MemoryReadTool {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryReadTool {
    pub fn new() -> Self {
        Self {
            inner: FileReadTool::new(),
        }
    }
}

#[async_trait]
impl Tool for MemoryReadTool {
    fn name(&self) -> &str {
        "MemoryRead"
    }

    async fn description(&self) -> BitFunResult<String> {
        Ok(
            "Read a memory file. The `file_path` must be absolute and resolve inside one of the \
             auto-memory fork's allowed memory roots; reads outside those roots are refused at \
             the tool layer. Same usage and limits as `Read`."
                .to_string(),
        )
    }

    fn input_schema(&self) -> Value {
        self.inner.input_schema()
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
            if let Err(message) =
                ensure_memory_path_required(input, ctx, PathField::FILE_PATH)
            {
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
        ensure_memory_path_required(input, context, PathField::FILE_PATH)
            .map_err(BitFunError::tool)?;
        self.inner.call_impl(input, context).await
    }
}
