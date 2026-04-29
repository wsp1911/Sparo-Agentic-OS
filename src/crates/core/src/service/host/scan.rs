use super::overview::{host_overview_file_path, HOST_OVERVIEW_CONTEXT_MAX_LINES};

const HOST_SCAN_ALLOWED_TOOL_NAMES: [&str; 6] = ["Read", "Grep", "Glob", "Write", "Edit", "Bash"];

fn format_path_for_prompt(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub(crate) fn default_host_scan_session_name() -> &'static str {
    "Host scan"
}

pub(crate) fn host_scan_allowed_tools() -> Vec<String> {
    HOST_SCAN_ALLOWED_TOOL_NAMES
        .into_iter()
        .map(str::to_string)
        .collect()
}

pub(crate) fn build_host_scan_system_reminder() -> String {
    let host_overview_path = host_overview_file_path();
    let host_overview_path = format_path_for_prompt(&host_overview_path);
    let context_max_lines = HOST_OVERVIEW_CONTEXT_MAX_LINES;

    format!(
        r#"You are now running as Sparo OS's host scan thread.

Sparo OS maintains a machine-level host overview so later sessions can quickly judge:
- where code work most likely lives,
- where documents and knowledge files most likely live,
- where installed software and tools most likely live,
- where downloads or mixed personal files tend to accumulate,
- which drives or roots are system-heavy versus user-work-heavy,
- and where new workspaces should probably be created.

Your job is to improve that routing knowledge by surveying the local host and updating `{host_overview_path}`.

Priorities:
- Build durable structural understanding, not an exhaustive inventory.
- Focus on high-signal directories and only go deeper when it materially improves routing judgment.
- Focus on user-facing host structure, avoid mentioning Sparo OS internal runtime, memory, or workspace details.

Working rules:
- Avoid reading personal document contents unless it is truly necessary.
- If the overview already exists, refine it instead of replacing useful guidance with a weaker summary.
- Only update the file when you can materially improve routing guidance.
- Prefer tightening, replacing, or removing weak content over blindly appending more text.
- Later sessions only load the first {context_max_lines} lines, so put the most important conclusions first.
- Keep the document compact. Do not write a travelogue of what you inspected.

Output format:
- Use short Markdown sections with informative headings.
- Start with a brief `## Routing Summary` section containing the highest-value conclusions first.
- Then use a small number of focused sections such as `## Storage Layout`, `## High-Signal Locations`, `## Workspace Recommendations`, `## User Profile`, and `## Notes`.
- In `## Storage Layout`, use host-appropriate terms: drives on Windows, volumes or mount points on macOS/Linux, or other clear root-level groupings when needed.
- Prefer concise bullets over long paragraphs.
- Emphasize durable guidance, not ephemeral detail.
- Do not include exhaustive directory listings, timestamps, or step-by-step scan logs.
- If something is uncertain, label it clearly.

Definition of done:
- The host overview file contains concise, practical, durable guidance that future Sparo OS sessions can rely on for host-level routing decisions."#
    )
}

pub(crate) fn build_host_scan_user_prompt() -> String {
    "Scan this host and update the shared host overview document with practical routing guidance."
        .to_string()
}
