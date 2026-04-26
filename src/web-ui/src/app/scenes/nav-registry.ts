/**
 * nav-registry — maps OverlaySceneId → lazy-loaded scene-specific NavPanel component.
 *
 * Extension pattern:
 *   1. Create `src/app/scenes/<scene>/XxxNav.tsx`
 *   2. Add one entry to SCENE_NAV_REGISTRY below
 *
 * Scenes without a registered nav component have no dedicated sidebar nav here.
 *
 * Note: `settings` and `shell` are not registered here — their nav is embedded
 * inside SettingsScene / ShellScene (self-contained left-right layouts).
 */

import { lazy } from 'react';
import type { ComponentType } from 'react';
import type { OverlaySceneId } from '../overlay/types';

type LazyNavComponent = ReturnType<typeof lazy<ComponentType>>;

const SCENE_NAV_REGISTRY: Partial<Record<OverlaySceneId, LazyNavComponent>> = {
  'file-viewer': lazy(() => import('./file-viewer/FileViewerNav')),
  // terminal: lazy(() => import('./terminal/TerminalNav')),
};

/**
 * Returns the lazy nav component registered for the given scene,
 * or `null` if there is no scene-specific nav for that overlay.
 */
export function getSceneNav(sceneId: OverlaySceneId): LazyNavComponent | null {
  return SCENE_NAV_REGISTRY[sceneId] ?? null;
}
