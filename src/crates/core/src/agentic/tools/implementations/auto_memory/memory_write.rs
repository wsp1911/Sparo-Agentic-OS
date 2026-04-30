//! Memory-only wrapper around `FileWriteTool`.

use super::path_guard::{ensure_memory_path_required, PathField};
use crate::agentic::tools::framework::{
    Tool, ToolRenderOptions, ToolResult, ToolUseContext, ValidationResult,
};
use crate::agentic::tools::implementations::FileWriteTool;
use crate::util::errors::{BitFunError, BitFunResult};
use async_trait::async_trait;
use serde_json::Value;

pub struct MemoryWriteTool {
    inner: FileWriteTool,
}

impl Default for MemoryWriteTool {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryWriteTool {
    pub fn new() -> Self {
        Self {
            inner: FileWriteTool::new(),
        }
    }
}

#[async_trait]
impl Tool for MemoryWriteTool {
    fn name(&self) -> &str {
        "MemoryWrite"
    }

    async fn description(&self) -> BitFunResult<String> {
        Ok(
            "Write or overwrite a memory file. The `file_path` must be absolute and resolve \
             inside one of the auto-memory fork's allowed memory roots; writes outside those \
             roots are refused at the tool layer. Same usage as `Write`."
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
