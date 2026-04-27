pub use crate::agentic::tools::workspace_paths::{
    normalize_path, resolve_path, resolve_path_with_workspace,
};

use crate::agentic::tools::framework::ToolUseContext;
use crate::live_app::try_get_global_live_app_manager;
use crate::util::errors::{BitFunError, BitFunResult};
use std::path::Path;

pub async fn enforce_live_app_studio_source_write(
    context: &ToolUseContext,
    resolved_path: &str,
) -> BitFunResult<()> {
    if context.agent_type.as_deref() != Some("LiveAppStudio") {
        return Ok(());
    }

    let manager = try_get_global_live_app_manager()
        .ok_or_else(|| BitFunError::tool("LiveAppManager not initialized".to_string()))?;
    let target = Path::new(resolved_path);
    let apps = manager.list().await?;
    for app in apps {
        let app_dir = manager.path_manager().live_app_dir(&app.id);
        let source_dir = app_dir.join("source");
        let allowed_manifest_files = [app_dir.join("meta.json"), app_dir.join("package.json")];
        if target.starts_with(&source_dir)
            || allowed_manifest_files
                .iter()
                .any(|allowed| target == allowed.as_path())
        {
            return Ok(());
        }
    }

    Err(BitFunError::validation(
        "LiveAppStudio can only write Live App source files or manifest files".to_string(),
    ))
}
