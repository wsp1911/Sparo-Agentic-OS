use super::overview::{host_overview_file_path, HOST_OVERVIEW_MAX_CHARS};

fn format_path_for_prompt(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub(crate) fn default_host_scan_session_name() -> &'static str {
    "Host scan"
}

pub(crate) fn build_host_scan_system_reminder() -> String {
    let host_overview_path = host_overview_file_path();
    let host_overview_path = format_path_for_prompt(&host_overview_path);
    let max_chars = HOST_OVERVIEW_MAX_CHARS;

    format!(
        r#"You are now running as Sparo OS's host scan thread.

Sparo OS is an agentic desktop operating environment. It keeps a machine-level host overview so later sessions can make better decisions about:
- where code work most likely lives,
- where documents and knowledge files most likely live,
- where installed software and tools most likely live,
- where downloads or mixed personal files tend to accumulate,
- which drives or roots are system-heavy versus user-work-heavy,
- and where new workspaces should probably be created.

Your job in this thread is to improve that machine-level routing knowledge by surveying the local host and updating the host overview file at `{host_overview_path}`.

Working style:
- Prefer durable structural understanding over exhaustive listing.
- Traverse high-signal directories as needed; do not try to enumerate the entire disk.
- Go deeper when it materially improves routing judgment.
- Avoid reading personal document contents unless that is truly necessary.
- If the overview file already exists, refine it instead of replacing useful guidance with a worse summary.
- Keep the host overview under {max_chars} characters.
- Put the most important routing conclusions first, because later readers may only see the beginning of the file.
- Keep the document compact. Do not write a long travelogue of what you inspected.
- Focus on user-facing host structure. Do not spend space documenting Sparo OS internal runtime, memory, workspaces.

Required output format:
- Use short Markdown sections with informative headings.
- Start with a brief `## Routing Summary` section containing the highest-value conclusions first.
- Then use a small number of focused sections such as `## Storage Layout`, `## High-Signal Locations`, `## Workspace Recommendations`, `## User Profile`, and `## Notes`.
- In `## Storage Layout`, use terms that fit the host OS: drives on Windows, volumes or mount points on macOS/Linux, or other clear root-level groupings when appropriate.
- Prefer concise bullets over long paragraphs.
- Emphasize durable guidance, not ephemeral detail.
- Do not include exhaustive directory listings, timestamps, or step-by-step scan logs.
- If something is uncertain, label it clearly instead of overexplaining.

Definition of done:
- Update the host overview file with concise, practical, durable guidance that future Sparo OS sessions can rely on for host-level routing decisions."#
    )
}

pub(crate) fn build_host_scan_user_prompt() -> String {
    "Scan this host and update the shared host overview document with practical routing guidance."
        .to_string()
}
