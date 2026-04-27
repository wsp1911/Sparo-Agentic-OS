---
name: liveapp-dev
description: Build, debug, and evolve Sparo OS Live Apps in packaged releases. Use for creating new Live Apps, working with window.app APIs, LiveAppStudio, InitLiveApp, app.fs, app.shell, app.storage, app.net, app.ai, or Live App runtime debugging.
---

# Sparo OS Live App Development

This is the packaged-release baseline for Live App work. It is intentionally compact and does not assume that the source repository, demo apps, or development docs are present on disk.

## Runtime Model

Live Apps use standard Web APIs plus the `window.app` bridge.

- UI runs as browser ESM in `ui.js`.
- Optional backend logic runs in `worker.js` and is called from UI with `app.call(method, params)`.
- Persistent app data should use `app.storage`.
- Host filesystem, shell, network, AI, dialog, and clipboard access must go through `window.app.*` APIs and declared permissions.

Default to no worker and minimal permissions. Enable broader capabilities only when the user's requested app requires them.

## Generation Loop

1. Clarify only user-facing intent: purpose, audience, data source, privacy or external access, and visual reference. Do not ask about runtime, permissions implementation, i18n, framework choices, or file layout.
2. Pick a visual direction before writing UI. If demo or bundled source anchors are unavailable, use the baseline utility style below.
3. Create the app with `InitLiveApp`.
4. Implement a placeholder-first skeleton in `index.html`, `style.css`, `ui.js`, and only use `worker.js` when needed.
5. After each coherent source-edit batch, run `LiveAppRecompile` then `LiveAppRuntimeProbe`.
6. Fix all fatal runtime errors before returning control to the user.
7. Use `LiveAppScreenshotMatrix` for light/dark and zh-CN/en-US review when visual quality matters. It requests captures; if images are not available yet, still inspect CSS for invalid or invented theme variables so the preview matches the intended host theme.

## Design Baseline

- Use host theme variables first: `--bitfun-bg`, `--bitfun-text`, `--bitfun-border`, `--bitfun-accent`, with fallbacks.
- Valid host theme variables are: `--bitfun-bg`, `--bitfun-bg-secondary`, `--bitfun-bg-tertiary`, `--bitfun-bg-elevated`, `--bitfun-text`, `--bitfun-text-secondary`, `--bitfun-text-muted`, `--bitfun-accent`, `--bitfun-accent-hover`, `--bitfun-success`, `--bitfun-warning`, `--bitfun-error`, `--bitfun-info`, `--bitfun-border`, `--bitfun-border-subtle`, `--bitfun-element-bg`, `--bitfun-element-hover`, `--bitfun-radius`, `--bitfun-radius-lg`, `--bitfun-font-sans`, `--bitfun-font-mono`, `--bitfun-scrollbar-thumb`, and `--bitfun-scrollbar-thumb-hover`.
- Do not treat invented names like `--bitfun-surface`, `--bitfun-card`, `--theme-bg`, or `--color-primary` as host variables. They are allowed only as app-local aliases defined in `:root`.
- Keep one dominant neutral surface, one secondary surface, and one restrained accent.
- Use `var(--bitfun-font-sans, system-ui, sans-serif)`.
- Title: 18-22px. Section labels: 13-15px. Body: 13-14px. Caption: 11-12px.
- Use one primary radius and one small radius consistently.
- Hit targets should be at least 32px tall.
- Empty states should explain what to do next. Do not add fake metrics or decorative charts just to fill space.

## Always Avoid

- Blue-purple Aurora gradients.
- Emoji as the primary app icon or main visual motif.
- Left color-bar plus rounded-card layouts.
- Decorative 1-2px lines under headings.
- Mixed corner radii without a system.
- Fake stats, sparklines, or icons used only to fill empty space.
- Hard-coded personal data, secrets, local absolute paths, or API keys.

## Permissions

Start with:

- `permissions.node.enabled = false`
- `permissions.fs.read = ["{appdata}"]`
- `permissions.fs.write = ["{appdata}"]`
- `permissions.shell.allow = []`
- `permissions.net.allow = []`

If workspace access is required, include a clear `permission_rationale` in metadata. If shell access is required, allow only the specific executable family needed, such as `git`.

## i18n

Ship zh-CN and en-US strings. Keep persisted data language-neutral. Use locale-aware display strings at render time.
