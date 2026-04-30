//! Shared path validation for the dedicated auto-memory tools.
//!
//! All `Memory*` tools share a single rule: the input path must resolve to a
//! location inside one of this fork's allowed memory roots. The roots are
//! advertised via `ToolUseContext::runtime_tool_restrictions.path_policy`,
//! which is built by `auto_memory::restrictions::build_auto_memory_runtime_restrictions(_with_extra_roots)`.
//!
//! For read-style tools (`MemoryRead` / `MemoryGlob` / `MemoryGrep`) we reuse
//! the same `write_roots` list as the boundary, because the auto-memory fork
//! never has any reason to read outside its own memory directory.

use crate::agentic::tools::framework::{ToolPathResolution, ToolUseContext};
use crate::agentic::tools::restrictions::is_local_path_within_root;
use std::path::Path;

/// A read-only view of the path field name to extract from a tool input.
#[derive(Debug, Clone, Copy)]
pub struct PathField(pub &'static str);

impl PathField {
    pub const FILE_PATH: Self = Self("file_path");
    pub const PATH: Self = Self("path");
}

/// Resolve the path field on `input` and ensure it lies within one of the
/// fork's memory roots. Remote workspace backends are rejected outright —
/// auto-memory storage is local-only today.
///
/// Returns the resolved path on success so callers can avoid resolving twice.
pub fn ensure_memory_path(
    input: &serde_json::Value,
    ctx: &ToolUseContext,
    field: PathField,
    field_required: bool,
) -> Result<Option<ToolPathResolution>, String> {
    let raw = match input.get(field.0).and_then(|value| value.as_str()) {
        Some(value) if !value.is_empty() => value,
        Some(_) => {
            return Err(format!("`{}` cannot be empty for memory tools", field.0));
        }
        None => {
            if field_required {
                return Err(format!(
                    "`{}` is required for memory tools (memory tools refuse implicit cwd)",
                    field.0
                ));
            }
            return Err(format!(
                "`{}` is required for memory tools — memory tools refuse implicit cwd defaults",
                field.0
            ));
        }
    };

    let resolved = ctx
        .resolve_tool_path(raw)
        .map_err(|err| format!("Failed to resolve `{}`: {}", field.0, err))?;

    if resolved.uses_remote_workspace_backend() {
        return Err(format!(
            "Memory tools do not support remote workspace paths: `{}`",
            resolved.logical_path
        ));
    }

    let roots = &ctx.runtime_tool_restrictions.path_policy.write_roots;
    if roots.is_empty() {
        return Err(
            "Memory tools require runtime restrictions with at least one memory root".to_string(),
        );
    }

    let resolved_path = Path::new(&resolved.resolved_path);
    let mut matched = false;
    for root in roots {
        let root_path = Path::new(root);
        match is_local_path_within_root(resolved_path, root_path) {
            Ok(true) => {
                matched = true;
                break;
            }
            Ok(false) => {}
            Err(_) => {}
        }
    }

    if !matched {
        return Err(format!(
            "Path is outside the allowed memory roots: `{}` (allowed roots: {})",
            resolved.logical_path,
            roots.join(", ")
        ));
    }

    // Forbidden subpaths inside any memory root. These are owned by other
    // pipelines (session-summary / slow consolidation) and the extractor
    // must never touch them, regardless of scope.
    let logical = resolved.logical_path.replace('\\', "/");
    const FORBIDDEN_FRAGMENTS: &[&str] = &[
        "/sessions/",
        "/narrative.md",
    ];
    for fragment in FORBIDDEN_FRAGMENTS {
        if logical.ends_with(fragment.trim_start_matches('/'))
            || logical.contains(fragment)
        {
            return Err(format!(
                "Memory tools cannot touch `{}` — that path is owned by another pipeline",
                resolved.logical_path
            ));
        }
    }

    Ok(Some(resolved))
}

/// Convenience wrapper that ignores the resolved path and returns `()`.
pub fn ensure_memory_path_required(
    input: &serde_json::Value,
    ctx: &ToolUseContext,
    field: PathField,
) -> Result<(), String> {
    ensure_memory_path(input, ctx, field, true).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forbidden_fragments_match_sessions_and_narrative() {
        // Smoke test that the literal-fragment scan catches the canonical
        // forbidden paths regardless of leading directories.
        let logical_session = "C:/users/test/.sparo_os/memory/sessions/turn-1.md";
        assert!(logical_session.contains("/sessions/"));

        let logical_narrative = "C:/users/test/.sparo_os/memory/narrative.md";
        assert!(logical_narrative.ends_with("narrative.md"));
    }
}
