//! Dedicated tools for the auto-memory extraction / consolidation forks.
//!
//! These tools are thin wrappers over the general-purpose file tools that add
//! a single invariant at the code layer: every input path must resolve inside
//! one of the fork's allowed memory roots (as advertised via
//! `ToolUseContext::runtime_tool_restrictions.path_policy.write_roots`).
//!
//! The auto-memory fork is configured (via `auto_memory::restrictions`) to
//! expose **only** these `Memory*` tool names to the model, which lets the
//! extraction prompt drop most of its tool-restriction prose.

pub mod path_guard;

pub mod memory_delete;
pub mod memory_edit;
pub mod memory_glob;
pub mod memory_grep;
pub mod memory_read;
pub mod memory_write;

pub use memory_delete::MemoryDeleteTool;
pub use memory_edit::MemoryEditTool;
pub use memory_glob::MemoryGlobTool;
pub use memory_grep::MemoryGrepTool;
pub use memory_read::MemoryReadTool;
pub use memory_write::MemoryWriteTool;
