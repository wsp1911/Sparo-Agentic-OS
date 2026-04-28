use crate::infrastructure::get_path_manager_arc;
use crate::util::errors::*;
use tokio::fs;

pub(crate) const HOST_OVERVIEW_MAX_CHARS: usize = 4_000;

pub(crate) fn host_overview_file_path() -> std::path::PathBuf {
    get_path_manager_arc().agentic_os_host_overview_path()
}

pub(crate) async fn ensure_host_overview_runtime_dir() -> BitFunResult<()> {
    let host_dir = get_path_manager_arc().agentic_os_host_dir();
    fs::create_dir_all(&host_dir).await.map_err(|error| {
        BitFunError::service(format!(
            "Failed to create Agentic OS host runtime directory {}: {}",
            host_dir.display(),
            error
        ))
    })?;
    Ok(())
}

pub(crate) async fn build_host_overview_context() -> BitFunResult<Option<String>> {
    ensure_host_overview_runtime_dir().await?;

    let path = host_overview_file_path();
    let mut section = format!(
        "# Host Environment Context\nThe runtime-managed host overview file lives at `{}`. This file is separate from memory and is reserved for machine-level routing context about where work, tools, documents, and new workspaces likely belong on this host.\n",
        format_path_for_prompt(&path)
    );

    if !path.exists() {
        section.push_str(
            "\nNo host overview has been written yet. If the current task is to scan the host, create this file.\n",
        );
        return Ok(Some(section));
    }

    let content = fs::read_to_string(&path).await.map_err(|error| {
        BitFunError::service(format!(
            "Failed to read host overview file {}: {}",
            path.display(),
            error
        ))
    })?;

    let trimmed = content.trim();
    if trimmed.is_empty() {
        section.push_str(
            "\nThe host overview file already exists but is currently empty. If the current task is to scan the host, populate it.\n",
        );
        return Ok(Some(section));
    }

    let truncated = truncate_to_char_boundary(trimmed, HOST_OVERVIEW_MAX_CHARS);
    section.push_str(
        "\nUse the existing overview below as prior machine-level context. Treat it as guidance, not ground truth, and verify before taking sensitive actions.\n\n",
    );
    section.push_str(&format!("<host_overview>{}</host_overview>", truncated));

    Ok(Some(section))
}

fn format_path_for_prompt(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn truncate_to_char_boundary(value: &str, max_chars: usize) -> &str {
    if value.chars().count() <= max_chars {
        return value;
    }

    let mut end = value.len();
    for (count, (index, _)) in value.char_indices().enumerate() {
        if count == max_chars {
            end = index;
            break;
        }
    }

    &value[..end]
}

#[cfg(test)]
mod tests {
    use super::truncate_to_char_boundary;

    #[test]
    fn truncate_preserves_utf8_boundaries() {
        let value = "盘符 overview";
        assert_eq!(truncate_to_char_boundary(value, 2), "盘符");
    }
}
