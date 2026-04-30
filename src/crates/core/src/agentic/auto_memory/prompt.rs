use crate::agentic::core::{Message, MessageRole, MessageSemanticKind};
use crate::service::memory_store::MemoryScope;

const EXTRACT_WORKSPACE: &str = include_str!("prompts/agent_memory_extract_workspace.md");
const EXTRACT_GLOBAL: &str = include_str!("prompts/agent_memory_extract_global.md");

pub fn count_recent_model_visible_messages(
    messages: &[Message],
    since_turn_id: Option<&str>,
) -> usize {
    let boundary_index = since_turn_id.and_then(|turn_id| {
        messages
            .iter()
            .enumerate()
            .rev()
            .find(|(_, message)| message.metadata.turn_id.as_deref() == Some(turn_id))
            .map(|(index, _)| index)
    });

    let count = messages
        .iter()
        .enumerate()
        .filter(|(index, _)| boundary_index.map_or(true, |boundary| *index > boundary))
        .filter(|(_, message)| is_model_visible_message(message))
        .count();

    count.max(1)
}

fn is_model_visible_message(message: &Message) -> bool {
    if matches!(
        message.metadata.semantic_kind,
        Some(MessageSemanticKind::ComputerUseVerificationScreenshot)
            | Some(MessageSemanticKind::ComputerUsePostActionSnapshot)
    ) {
        return false;
    }

    matches!(
        message.role,
        MessageRole::User | MessageRole::Assistant | MessageRole::Tool
    )
}

pub fn build_extract_prompt(
    recent_message_count: usize,
    memory_dir: &str,
    existing_memories: Option<&str>,
    memory_scope: MemoryScope,
) -> String {
    build_extract_prompt_with_global(
        recent_message_count,
        memory_dir,
        None,
        existing_memories,
        memory_scope,
    )
}

/// Build the extraction prompt with optional dual-scope routing.
///
/// When `global_memory_dir` is provided (non-empty and different from
/// `memory_dir`), the agent is instructed to route user-level memories to the
/// global scope while keeping project-specific content in the workspace scope.
pub fn build_extract_prompt_with_global(
    recent_message_count: usize,
    memory_dir: &str,
    global_memory_dir: Option<&str>,
    existing_memories: Option<&str>,
    memory_scope: MemoryScope,
) -> String {
    let manifest = existing_memories
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            format!(
                "\n\n## Existing memory files\n\n{}\n\nCheck this list before writing — update an existing file rather than creating a duplicate.",
                value.trim()
            )
        })
        .unwrap_or_default();

    // Build the scope routing section.
    //
    // - For workspace extraction with a global dir available, emit dual-scope routing
    //   with a clear default (workspace) and a no-double-write rule.
    // - For global extraction, emit a short single-scope reminder.
    let routing_section = match (memory_scope, global_memory_dir) {
        (MemoryScope::WorkspaceProject, Some(global_dir))
            if !global_dir.is_empty() && global_dir != memory_dir =>
        {
            format!(
                "\n\n**Scope routing — pick exactly one directory per memory:**\n\
- **Default scope: workspace** (`{memory_dir}`). When in doubt, route here.\n\
- Escalate to **global** (`{global_dir}`) ONLY when the memory is clearly about the user as a person across projects: communication style, durable preferences, cross-workspace tools, identity-level direction. Write global items to `{global_dir}/habits.md`, `{global_dir}/identity.md`, or `{global_dir}/pinned/<slug>.md`. Do NOT write `persona.md` from extraction — let the slow consolidation pass author it.\n\
- Workspace items go to `{memory_dir}/project.md`, `{memory_dir}/habits.md`, `{memory_dir}/identity.md` (project-rules anchor only), `{memory_dir}/pinned/<slug>.md`, or `{memory_dir}/episodes/YYYY-MM/YYYY-MM-DD-<slug>.md`.\n\
- **Never double-write.** Do not save the same memory to both scopes. Pick one.\n\
- Episodic entries always go in the workspace scope (`{memory_dir}/episodes/`); never write episodes to global.\n\
- Do **not** write under `{memory_dir}/sessions/` or `{global_dir}/sessions/` — those paths are owned by the session-summary pass."
            )
        }
        (MemoryScope::GlobalAgenticOs, _) => format!(
            "\n\n**Scope: global.** You are extracting cross-project memories about the user. Episodic entries belong to workspaces — do not write any episodes here. Do not write under `{memory_dir}/sessions/` (owned by the session-summary pass). Do not write `narrative.md` (owned by the slow consolidation pass)."
        ),
        _ => format!(
            "\n\n**Scope: workspace (single).** Do not write under `{memory_dir}/sessions/` (owned by the session-summary pass)."
        ),
    };

    let write_roots_description = match global_memory_dir {
        Some(global_dir) if !global_dir.is_empty() && global_dir != memory_dir => {
            format!("paths inside `{memory_dir}` or `{global_dir}`")
        }
        _ => format!("paths inside `{memory_dir}`"),
    };

    let r_count = recent_message_count.to_string();
    let tpl = match memory_scope {
        MemoryScope::WorkspaceProject => EXTRACT_WORKSPACE,
        MemoryScope::GlobalAgenticOs => EXTRACT_GLOBAL,
    };

    tpl.replace("{recent_message_count}", &r_count)
        .replace("{write_roots_description}", &write_roots_description)
        .replace("{routing_section}", &routing_section)
        .replace("{manifest}", &manifest)
}

#[cfg(test)]
mod tests {
    use super::{
        build_extract_prompt, build_extract_prompt_with_global, count_recent_model_visible_messages,
    };
    use crate::agentic::core::{Message, MessageSemanticKind, ToolCall, ToolResult};
    use serde_json::json;

    #[test]
    fn counts_runtime_message_flow_including_tool_results() {
        let messages = vec![
            Message::user("old user".to_string()).with_turn_id("turn-1".to_string()),
            Message::assistant_with_tools(
                "calling tool".to_string(),
                vec![ToolCall {
                    tool_id: "tool-1".to_string(),
                    tool_name: "Read".to_string(),
                    arguments: json!({ "file_path": "a.txt" }),
                    is_error: false,
                }],
            )
            .with_turn_id("turn-2".to_string()),
            Message::tool_result(ToolResult {
                tool_id: "tool-1".to_string(),
                tool_name: "Read".to_string(),
                result: json!({ "content": "hello" }),
                result_for_assistant: Some("hello".to_string()),
                is_error: false,
                duration_ms: None,
                image_attachments: None,
            })
            .with_turn_id("turn-2".to_string()),
            Message::assistant("final answer".to_string()).with_turn_id("turn-2".to_string()),
        ];

        assert_eq!(
            count_recent_model_visible_messages(&messages, Some("turn-1")),
            3
        );
    }

    #[test]
    fn falls_back_to_all_visible_messages_when_boundary_turn_is_missing() {
        let messages = vec![
            Message::user("user".to_string()).with_turn_id("turn-2".to_string()),
            Message::assistant("assistant".to_string()).with_turn_id("turn-2".to_string()),
        ];

        assert_eq!(
            count_recent_model_visible_messages(&messages, Some("missing-turn")),
            2
        );
    }

    #[test]
    fn uses_last_message_of_boundary_turn_before_counting() {
        let messages = vec![
            Message::user("turn one".to_string()).with_turn_id("turn-1".to_string()),
            Message::assistant("answer one".to_string()).with_turn_id("turn-1".to_string()),
            Message::user("turn two".to_string()).with_turn_id("turn-2".to_string()),
            Message::assistant("answer two".to_string()).with_turn_id("turn-2".to_string()),
        ];

        assert_eq!(
            count_recent_model_visible_messages(&messages, Some("turn-1")),
            2
        );
    }

    #[test]
    fn excludes_system_and_non_model_visible_runtime_messages() {
        let messages = vec![
            Message::system("system".to_string()),
            Message::user("user".to_string()).with_turn_id("turn-1".to_string()),
            Message::assistant("verification".to_string())
                .with_turn_id("turn-1".to_string())
                .with_semantic_kind(MessageSemanticKind::ComputerUseVerificationScreenshot),
            Message::assistant("answer".to_string()).with_turn_id("turn-1".to_string()),
        ];

        assert_eq!(count_recent_model_visible_messages(&messages, None), 2);
    }

    #[test]
    fn extract_prompt_omits_full_memory_access_guidance_sections() {
        let prompt = build_extract_prompt(
            7,
            "/workspace/memory",
            None,
            crate::service::memory_store::MemoryScope::WorkspaceProject,
        );

        assert!(
            !prompt.contains("If the user explicitly asks you to remember something"),
            "extract prompt should not include explicit request guidance"
        );
        assert!(
            !prompt.contains("## When to access memories"),
            "extract prompt should not include access-memory guidance"
        );
        assert!(
            !prompt.contains("## Before recommending from memory"),
            "extract prompt should not include post-access memory guidance"
        );
        assert!(
            !prompt.contains("## Memory and other forms of persistence"),
            "extract prompt should stop before memory persistence guidance"
        );
    }

    #[test]
    fn global_extract_prompt_uses_global_memory_policy() {
        let prompt = build_extract_prompt(
            7,
            "/global/memory",
            None,
            crate::service::memory_store::MemoryScope::GlobalAgenticOs,
        );

        assert!(prompt.contains("## Special workspace overview files"));
        assert!(prompt.contains("<name>identity</name>"));
        assert!(prompt.contains("<name>narrative</name>"));
        assert!(prompt.contains("<name>vision</name>"));
        assert!(!prompt.contains("<name>project</name>"));
        assert!(!prompt.contains("## When to access memories"));
    }

    // -------- New behavior tests for the optimized memory prompts ----------

    #[test]
    fn extract_prompt_inverts_default_to_do_nothing() {
        let prompt = build_extract_prompt(
            7,
            "/workspace/memory",
            None,
            crate::service::memory_store::MemoryScope::WorkspaceProject,
        );

        // The salience gate must default to "do nothing" so extraction is
        // biased toward signal over coverage.
        assert!(
            prompt.contains("Default action") && prompt.contains("do nothing"),
            "extract prompt should make 'do nothing' the explicit default"
        );
        // Lifecycle: extraction writes tentative, not confirmed directly.
        assert!(
            prompt.contains("status: tentative"),
            "extract prompt should require status: tentative on new entries"
        );
        assert!(
            prompt.contains("session-summary pass will promote"),
            "extract prompt should hand off promotion to session-summary"
        );
        // Front matter must require entities and links to keep entries
        // discoverable via agentic search.
        assert!(prompt.contains("entities:"));
        assert!(prompt.contains("links:"));
        // Reverse triggers must be present.
        assert!(prompt.contains("Never save (reverse triggers)"));
        // Single-line success format.
        assert!(prompt.contains("Memory updated: N entries."));
    }

    #[test]
    fn extract_prompt_workspace_scope_routing_defaults_to_workspace() {
        let prompt = build_extract_prompt_with_global(
            7,
            "/workspace/memory",
            Some("/global/memory"),
            None,
            crate::service::memory_store::MemoryScope::WorkspaceProject,
        );

        assert!(prompt.contains("Default scope: workspace"));
        assert!(prompt.contains("Never double-write"));
        assert!(prompt.contains("never write episodes to global"));
        // sessions/ is owned by the session-summary pass.
        assert!(prompt.contains("session-summary pass"));
    }

    #[test]
    fn extract_prompt_global_scope_uses_short_routing_section() {
        let prompt = build_extract_prompt_with_global(
            7,
            "/global/memory",
            Some("/global/memory"),
            None,
            crate::service::memory_store::MemoryScope::GlobalAgenticOs,
        );

        assert!(prompt.contains("Scope: global"));
        assert!(prompt.contains("Episodic entries belong to workspaces"));
        assert!(prompt.contains("narrative.md"));
    }

    #[test]
    fn extract_prompt_carries_philosophy_header() {
        let prompt = build_extract_prompt(
            7,
            "/workspace/memory",
            None,
            crate::service::memory_store::MemoryScope::WorkspaceProject,
        );

        // The philosophy header is the single source of truth for "why we
        // have memory at all". It must be present in every memory prompt.
        assert!(prompt.contains("## Memory philosophy (read first)"));
        assert!(prompt.contains("Behavior over narration"));
        assert!(prompt.contains("Memory ≠ facts"));
    }
}
