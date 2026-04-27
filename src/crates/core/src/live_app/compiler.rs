//! Live App compiler — assemble source (html/css/ui_js) + Import Map + Runtime Adapter + CSP into compiled_html.

use crate::live_app::bridge_builder::{
    build_bridge_script, build_csp_content, build_import_map, build_live_app_default_theme_css,
    scroll_boundary_script,
};
use crate::live_app::runtime_ui_kit::{build_runtime_ui_kit_css, build_runtime_ui_kit_script};
use crate::live_app::types::{LiveAppPermissions, LiveAppSource};
use crate::util::errors::{BitFunError, BitFunResult};

/// Compile Live App source into full HTML with Import Map, Runtime Adapter, and CSP injected.
pub fn compile(
    source: &LiveAppSource,
    permissions: &LiveAppPermissions,
    app_id: &str,
    app_data_dir: &str,
    workspace_dir: &str,
    theme: &str,
) -> BitFunResult<String> {
    let platform = if cfg!(target_os = "windows") {
        "win32"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    };

    let bridge = build_bridge_script(app_id, app_data_dir, workspace_dir, theme, platform);
    let csp = build_csp_content(permissions);
    let csp_tag = format!(
        "<meta http-equiv=\"Content-Security-Policy\" content=\"{}\">",
        csp.replace('"', "&quot;")
    );
    let scroll = scroll_boundary_script();
    let theme_default_style = build_live_app_default_theme_css();
    let runtime_ui_kit_style = build_runtime_ui_kit_css();
    let import_map = build_import_map(&source.esm_dependencies);
    let style_tag = if source.css.is_empty() {
        String::new()
    } else {
        format!("<style>\n{}\n</style>", source.css)
    };
    let bridge_script_tag = format!("<script>\n{}\n</script>", bridge);
    let runtime_ui_kit_script = build_runtime_ui_kit_script();
    let user_script_tag = if source.ui_js.is_empty() {
        String::new()
    } else {
        format!("<script type=\"module\">\n{}\n</script>", source.ui_js)
    };

    let head_content = format!(
        "\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
        theme_default_style,
        runtime_ui_kit_style,
        csp_tag,
        scroll,
        import_map,
        bridge_script_tag,
        runtime_ui_kit_script,
        style_tag,
    );

    let html = if source.html.trim().is_empty() {
        let theme_attr = format!(" data-theme-type=\"{}\"", escape_html_attr(theme));
        format!(
            r#"<!DOCTYPE html>
<html{theme_attr}>
<head>{head}</head>
<body>
{user_script}
</body>
</html>"#,
            theme_attr = theme_attr,
            head = head_content,
            user_script = user_script_tag,
        )
    } else {
        let with_theme = inject_data_theme_type(&source.html, theme);
        let with_head = inject_into_head(&with_theme, &head_content)?;
        inject_before_body_close(&with_head, &user_script_tag)
    };

    Ok(html)
}

/// Place content just before </body>. If no </body> found, append before </html> or at end.
fn inject_before_body_close(html: &str, content: &str) -> String {
    if content.is_empty() {
        return html.to_string();
    }
    if let Some(pos) = html.rfind("</body>") {
        let (before, after) = html.split_at(pos);
        return format!("{}\n{}\n{}", before, content, after);
    }
    if let Some(pos) = html.rfind("</html>") {
        let (before, after) = html.split_at(pos);
        return format!("{}\n{}\n{}", before, content, after);
    }
    format!("{}\n{}", html, content)
}

fn escape_html_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Inject or replace data-theme-type on the first <html> tag.
fn inject_data_theme_type(html: &str, theme: &str) -> String {
    let safe = escape_html_attr(theme);
    if let Some(idx) = html.find("<html") {
        let after_html = idx + 5;
        let rest = &html[after_html..];
        if let Some(close) = rest.find('>') {
            let insert = format!(" data-theme-type=\"{}\"", safe);
            return format!(
                "{}{}>{}",
                &html[..after_html + close],
                insert,
                &html[after_html + close + 1..]
            );
        }
    }
    html.to_string()
}

fn inject_into_head(html: &str, content: &str) -> BitFunResult<String> {
    if let Some(head_start) = html.find("<head") {
        let after_head_open = if let Some(close_bracket) = html[head_start..].find('>') {
            head_start + close_bracket + 1
        } else {
            return Err(BitFunError::validation(
                "Invalid HTML: <head> not properly opened".to_string(),
            ));
        };
        let before = &html[..after_head_open];
        let after = &html[after_head_open..];
        return Ok(format!("{}{}{}", before, content, after));
    }

    if let Some(html_open) = html.find("<html") {
        let after_html_open = if let Some(close_bracket) = html[html_open..].find('>') {
            html_open + close_bracket + 1
        } else {
            return Err(BitFunError::validation(
                "Invalid HTML: <html> not properly opened".to_string(),
            ));
        };
        let before = &html[..after_html_open];
        let after = &html[after_html_open..];
        return Ok(format!("{}\n<head>{}</head>{}", before, content, after));
    }

    Ok(format!(
        r#"<!DOCTYPE html>
<html>
<head>{}</head>
<body>
{}
</body>
</html>"#,
        content, html
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_into_head() {
        let html =
            r#"<!DOCTYPE html><html><head><meta charset="utf-8"></head><body>x</body></html>"#;
        let content = "<!-- injected -->";
        let out = inject_into_head(html, content).unwrap();
        assert!(out.contains("<!-- injected -->"));
        assert!(out.contains("<meta charset"));
    }

    #[test]
    fn compile_injects_runtime_ui_kit() {
        let source = LiveAppSource {
            html: r#"<!DOCTYPE html><html><head></head><body><div id="app"></div></body></html>"#
                .to_string(),
            css: String::new(),
            ui_js: String::new(),
            esm_dependencies: Vec::new(),
            worker_js: String::new(),
            npm_dependencies: Vec::new(),
        };
        let html = compile(
            &source,
            &LiveAppPermissions::default(),
            "app-id",
            "/tmp/appdata",
            "/tmp/workspace",
            "dark",
        )
        .unwrap();

        assert!(html.contains("bitfun-runtime-ui-kit"));
        assert!(html.contains("window.app.ui = ui"));
        assert!(html.contains("btn-primary"));
    }
}
