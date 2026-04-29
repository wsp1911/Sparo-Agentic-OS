 

export { darkTheme } from './dark-theme';
export { lightTheme } from './light-theme';
export { bitfunChinaStyleTheme } from './china-style-theme';
export { bitfunChinaNightTheme } from './china-night-theme';
export { bitfunCyberTheme } from './cyber-theme';
export { slateTheme } from './slate-theme';

import { darkTheme } from './dark-theme';
import { lightTheme } from './light-theme';
import { bitfunChinaStyleTheme } from './china-style-theme';
import { bitfunChinaNightTheme } from './china-night-theme';
import { bitfunCyberTheme } from './cyber-theme';
import { slateTheme } from './slate-theme';
import { ThemeConfig, ThemeId, SYSTEM_THEME_ID, ThemeSelectionId } from '../types';

/** Legacy ids from older builds → current builtin id. */
export const LEGACY_THEME_ID_ALIASES: Partial<Record<ThemeId, ThemeId>> = {
  'bitfun-light': 'light',
  'sparo-light': 'light',
  'bitfun-dark': 'dark',
  'bitfun-slate': 'slate',
  'bitfun-midnight': 'slate',
};

export function resolveThemeId(id: ThemeId): ThemeId {
  return LEGACY_THEME_ID_ALIASES[id] ?? id;
}

export function resolveThemeSelectionId(id: ThemeSelectionId | null): ThemeSelectionId | null {
  if (id == null) {
    return null;
  }
  if (id === SYSTEM_THEME_ID) {
    return SYSTEM_THEME_ID;
  }
  return resolveThemeId(id) as ThemeSelectionId;
}

/** Default light / dark builtin themes used when following system appearance. */
export const DEFAULT_LIGHT_THEME_ID: ThemeId = 'light';
export const DEFAULT_DARK_THEME_ID: ThemeId = 'dark';

/**
 * Picks dark vs light from `prefers-color-scheme`.
 * Used when the user has no saved theme preference.
 */
export function getSystemPreferredDefaultThemeId(): ThemeId {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return DEFAULT_LIGHT_THEME_ID;
  }
  return window.matchMedia('(prefers-color-scheme: dark)').matches
    ? DEFAULT_DARK_THEME_ID
    : DEFAULT_LIGHT_THEME_ID;
}

/** Static fallback when system preference is unavailable (e.g. SSR). */
export const DEFAULT_THEME_ID: ThemeId = DEFAULT_LIGHT_THEME_ID;

 
export const builtinThemes: ThemeConfig[] = [
  lightTheme,
  slateTheme,
  darkTheme,
  bitfunChinaStyleTheme,
  bitfunChinaNightTheme,
  bitfunCyberTheme,
];

 


