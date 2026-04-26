/**
 * Design tokens — canonical schema + resolver helpers.
 *
 * Mirrors the Rust `CanonicalTokens` struct in `design_artifact_tool.rs` and
 * the documented schema in `prompts/design_mode.md`. Any new key has to be
 * added in BOTH places; this file is the single entry point every UI surface
 * (Proposal card, Studio, Canvas preview) reads from so nobody hand-rolls
 * their own "pick('primary', 'accent', 'brand', ...)" cascade.
 *
 * If the `DesignTokens` tool is ever extended with a new canonical key, add it
 * here, to `CanonicalTokens` in Rust, and to the prompt's schema bullet list.
 */

import type { DesignTokenProposal } from './store/designTokensStore';

// -------------------------------- Resolve ---------------------------------

/** Read the first non-empty string value across a list of candidate keys. */
export function pickString(
  source: Record<string, unknown> | undefined | null,
  ...keys: string[]
): string | undefined {
  if (!source) return undefined;
  for (const key of keys) {
    const v = source[key];
    if (typeof v === 'string' && v.trim().length > 0) return v;
    if (typeof v === 'number') return String(v);
  }
  return undefined;
}

/**
 * Resolve a proposal into the canonical (flat) CSS-variable map. All tokens are
 * optional and come with a neutral fallback so a partial proposal still renders
 * a coherent system rather than a mess of `undefined`s.
 *
 * The variable names match `render_tokens_css` in the Rust tool; the preview
 * iframe's scaffold uses the same names.
 */
export interface ResolvedTokens {
  // Typography
  '--dt-font-family': string;
  '--dt-font-family-mono': string;
  '--dt-font-display': string;
  '--dt-font-headline': string;
  '--dt-font-title': string;
  '--dt-font-body': string;
  '--dt-font-caption': string;
  // Colors (surface-agnostic by default — callers that need a light/dark
  // preview should override `--dt-background` / `--dt-surface` / text vars).
  '--dt-background': string;
  '--dt-surface': string;
  '--dt-surface-elevated': string;
  '--dt-border': string;
  '--dt-text': string;
  '--dt-text-secondary': string;
  '--dt-text-muted': string;
  '--dt-primary': string;
  '--dt-primary-hover': string;
  '--dt-accent': string;
  '--dt-success': string;
  '--dt-warning': string;
  '--dt-danger': string;
  // Radius
  '--dt-radius-sm': string;
  '--dt-radius-md': string;
  '--dt-radius-lg': string;
  '--dt-radius-full': string;
  // Shadow
  '--dt-shadow-sm': string;
  '--dt-shadow-md': string;
  '--dt-shadow-lg': string;
  // Spacing
  '--dt-space-xs': string;
  '--dt-space-sm': string;
  '--dt-space-md': string;
  '--dt-space-lg': string;
  '--dt-space-xl': string;
  // Motion
  '--dt-duration': string;
  '--dt-ease': string;
}

/** Light-/dark-surface override helper for the Studio & Proposal card preview. */
export interface SurfaceOverride {
  background: string;
  surface: string;
  surfaceElevated: string;
  border: string;
  text: string;
  textSecondary: string;
  textMuted: string;
}

export const LIGHT_SURFACE: SurfaceOverride = {
  background: '#f7f7f5',
  surface: '#ffffff',
  surfaceElevated: '#ffffff',
  border: 'rgba(12, 13, 16, 0.09)',
  text: '#0c0d10',
  textSecondary: 'rgba(12, 13, 16, 0.72)',
  textMuted: 'rgba(12, 13, 16, 0.55)',
};

export const DARK_SURFACE: SurfaceOverride = {
  background: '#0b0b0d',
  surface: '#141418',
  surfaceElevated: '#1a1c21',
  border: 'rgba(255, 255, 255, 0.09)',
  text: '#f5f7fb',
  textSecondary: 'rgba(245, 247, 251, 0.72)',
  textMuted: 'rgba(245, 247, 251, 0.55)',
};

export function resolveTokens(
  proposal: Partial<DesignTokenProposal> | undefined | null
): ResolvedTokens {
  const colors = (proposal?.colors ?? {}) as Record<string, string>;
  const typography = (proposal?.typography ?? {}) as Record<string, any>;
  const scale = (typography?.scale ?? {}) as Record<string, any>;
  const radius = (proposal?.radius ?? {}) as Record<string, any>;
  const shadow = (proposal?.shadow ?? {}) as Record<string, any>;
  const spacing = (proposal?.spacing ?? {}) as Record<string, any>;
  const motion = (proposal?.motion ?? {}) as Record<string, any>;
  const motionDuration = (motion?.duration ?? {}) as Record<string, any>;

  const primary =
    pickString(colors, 'primary', 'accent', 'brand') ?? '#0b0b0c';

  return {
    '--dt-font-family':
      pickString(typography, 'fontFamily', 'family') ??
      'Inter, system-ui, -apple-system, sans-serif',
    '--dt-font-family-mono':
      pickString(typography, 'fontFamilyMono', 'familyMono') ??
      'ui-monospace, SFMono-Regular, Menlo, monospace',
    '--dt-font-display': pickString(scale, 'display', 'headline', 'title') ?? '48px',
    '--dt-font-headline': pickString(scale, 'headline', 'display', 'title') ?? '32px',
    '--dt-font-title': pickString(scale, 'title', 'heading') ?? '20px',
    '--dt-font-body': pickString(scale, 'body', 'base') ?? '15px',
    '--dt-font-caption': pickString(scale, 'caption', 'small') ?? '12px',

    '--dt-background': pickString(colors, 'background', 'bg') ?? '#ffffff',
    '--dt-surface': pickString(colors, 'surface') ?? '#fafafa',
    '--dt-surface-elevated':
      pickString(colors, 'surfaceElevated', 'surface_elevated') ?? '#ffffff',
    '--dt-border': pickString(colors, 'border') ?? 'rgba(17, 17, 17, 0.08)',
    '--dt-text': pickString(colors, 'text', 'textPrimary', 'text_primary') ?? '#0b0b0c',
    '--dt-text-secondary':
      pickString(colors, 'textSecondary', 'text_secondary') ?? 'rgba(11, 11, 12, 0.72)',
    '--dt-text-muted':
      pickString(colors, 'textMuted', 'text_muted') ?? 'rgba(11, 11, 12, 0.52)',
    '--dt-primary': primary,
    '--dt-primary-hover':
      pickString(colors, 'primaryHover', 'primary_hover') ?? primary,
    '--dt-accent': pickString(colors, 'accent', 'primary', 'brand') ?? primary,
    '--dt-success': pickString(colors, 'success') ?? '#16a34a',
    '--dt-warning': pickString(colors, 'warning') ?? '#d97706',
    '--dt-danger': pickString(colors, 'danger', 'error') ?? '#dc2626',

    '--dt-radius-sm': pickString(radius, 'sm', 'xs') ?? '4px',
    '--dt-radius-md': pickString(radius, 'md', 'base') ?? '8px',
    '--dt-radius-lg': pickString(radius, 'lg') ?? '16px',
    '--dt-radius-full': pickString(radius, 'full', 'pill') ?? '999px',

    '--dt-shadow-sm': pickString(shadow, 'sm') ?? '0 1px 2px rgba(0,0,0,0.06)',
    '--dt-shadow-md': pickString(shadow, 'md', 'base') ?? '0 4px 14px rgba(0,0,0,0.10)',
    '--dt-shadow-lg': pickString(shadow, 'lg') ?? '0 18px 40px rgba(0,0,0,0.18)',

    '--dt-space-xs': pickString(spacing, 'xs') ?? '4px',
    '--dt-space-sm': pickString(spacing, 'sm', 'xs') ?? '8px',
    '--dt-space-md': pickString(spacing, 'md', 'base') ?? '16px',
    '--dt-space-lg': pickString(spacing, 'lg') ?? '24px',
    '--dt-space-xl': pickString(spacing, 'xl', 'lg') ?? '40px',

    '--dt-duration': pickString(motionDuration, 'normal', 'base', 'md') ?? '200ms',
    '--dt-ease': pickString(motion, 'ease') ?? 'cubic-bezier(0.4, 0, 0.2, 1)',
  };
}

/** Apply a light/dark surface override to an existing resolved token map. */
export function applySurface(
  base: ResolvedTokens,
  surface: SurfaceOverride
): ResolvedTokens {
  return {
    ...base,
    '--dt-background': surface.background,
    '--dt-surface': surface.surface,
    '--dt-surface-elevated': surface.surfaceElevated,
    '--dt-border': surface.border,
    '--dt-text': surface.text,
    '--dt-text-secondary': surface.textSecondary,
    '--dt-text-muted': surface.textMuted,
  };
}

// -------------------------- Store key canonicalization --------------------

/**
 * Canonical key used by `useDesignTokensStore.byScope`. Must match whatever
 * `DesignTokens.propose` returns in `data.path`, so:
 *
 *   - If the backend echoes an absolute path we keep it as-is (after separator
 *     normalization).
 *   - Otherwise we derive a deterministic key from workspace + artifact id.
 *
 * This removes the subtle bug where `DesignTokensStudio` fell back to
 * `Object.values(docs)[0]` because its computed scopePath didn't match the
 * path the backend emitted.
 */
export function canonicalScopeKey(input: {
  explicitPath?: string | null;
  workspacePath?: string | null;
  artifactId?: string | null;
}): string {
  const { explicitPath, workspacePath, artifactId } = input;
  if (explicitPath && typeof explicitPath === 'string') {
    return explicitPath.replace(/\\/g, '/');
  }
  const ws = (workspacePath || '').replace(/\\/g, '/').replace(/\/+$/, '');
  if (artifactId) {
    return ws ? `${ws}/.design/${artifactId}/tokens.json` : `artifact:${artifactId}`;
  }
  return ws ? `${ws}/.design/tokens.json` : 'workspace';
}
