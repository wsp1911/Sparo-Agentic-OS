/**
 * Thin client-side facade for the DesignArtifact tool.
 *
 * All calls route through the standard `execute_tool` Tauri command so the
 * backend applies the same validation, snapshotting, and path resolution rules
 * the Design agent would apply. After every call the returned manifest is
 * written back into `designArtifactStore`, keeping the UI canonical.
 */

import { toolAPI } from '@/infrastructure/api';
import { useDesignArtifactStore, type DesignArtifactManifest, type ArtifactEventKind } from './store/designArtifactStore';
import { createLogger } from '@/shared/utils/logger';

const log = createLogger('designArtifactAPI');

interface InvokeResult {
  success?: boolean;
  artifact_event?: ArtifactEventKind | string;
  manifest?: DesignArtifactManifest;
  manifests?: DesignArtifactManifest[];
  export_path?: string;
  error?: string;
  [key: string]: unknown;
}

function parseResult(raw: unknown): InvokeResult | null {
  if (!raw) return null;
  if (typeof raw === 'string') {
    try {
      return JSON.parse(raw) as InvokeResult;
    } catch {
      return null;
    }
  }
  if (typeof raw !== 'object') return null;
  const envelope = raw as any;
  // Desktop `execute_tool` returns { tool_name, success, result, error, validation_error }
  // where `result` is the tool-level JSON (our { success, artifact_event, manifest }).
  if (envelope.success === false || envelope.error || envelope.validationError || envelope.validation_error) {
    const inner = (envelope.result && typeof envelope.result === 'object') ? envelope.result : {};
    return {
      ...(inner as InvokeResult),
      success: false,
      error:
        envelope.error ||
        envelope.validationError ||
        envelope.validation_error ||
        'Tool execution failed',
    };
  }
  if (envelope.result && typeof envelope.result === 'object') {
    return envelope.result as InvokeResult;
  }
  if (envelope.results && Array.isArray(envelope.results)) {
    const first = envelope.results[0];
    if (first && typeof first === 'object') return first as InvokeResult;
  }
  return envelope as InvokeResult;
}

async function invokeAction(
  input: Record<string, unknown>,
  workspacePath?: string
): Promise<InvokeResult> {
  const raw = await toolAPI.executeTool({
    toolName: 'DesignArtifact',
    parameters: input,
    workspacePath,
  } as any);
  const result = parseResult(raw) || {};
  if (result.manifest) {
    useDesignArtifactStore
      .getState()
      .upsertManifest(result.manifest, (result.artifact_event as ArtifactEventKind) || 'ok');
  }
  if (result.manifests) {
    useDesignArtifactStore.getState().upsertManifests(result.manifests);
  }
  if (result.success === false) {
    const message = String(result.error || 'DesignArtifact action failed');
    if (
      /invalid json/i.test(message) ||
      /payload is too large/i.test(message) ||
      /truncat/i.test(message)
    ) {
      throw new Error(
        `${message} The design payload was too large for a single tool call. Create a tiny scaffold first, then update files in smaller chunks.`
      );
    }
    throw new Error(message);
  }
  return result;
}

export const designArtifactAPI = {
  async updateFile(
    artifactId: string,
    path: string,
    content: string,
    opts?: { expectedVersion?: string; as?: string; force?: boolean; workspacePath?: string }
  ): Promise<InvokeResult> {
    return invokeAction(
      {
        action: 'update_file',
        artifact_id: artifactId,
        path,
        content,
        expected_version: opts?.expectedVersion,
        as: opts?.as ?? 'human',
        force: opts?.force,
      },
      opts?.workspacePath
    );
  },

  async snapshot(
    artifactId: string,
    opts?: { summary?: string; author?: string; workspacePath?: string }
  ): Promise<InvokeResult> {
    return invokeAction(
      {
        action: 'snapshot',
        artifact_id: artifactId,
        summary: opts?.summary ?? 'manual snapshot',
        author: opts?.author ?? 'human',
      },
      opts?.workspacePath
    );
  },

  async acquireLock(
    artifactId: string,
    opts?: { holder?: string; note?: string; force?: boolean; workspacePath?: string }
  ): Promise<InvokeResult> {
    return invokeAction(
      {
        action: 'acquire_lock',
        artifact_id: artifactId,
        holder: opts?.holder ?? 'human',
        note: opts?.note,
        force: opts?.force,
      },
      opts?.workspacePath
    );
  },

  async releaseLock(artifactId: string, workspacePath?: string): Promise<InvokeResult> {
    return invokeAction(
      { action: 'release_lock', artifact_id: artifactId },
      workspacePath
    );
  },

  async setThumbnail(
    artifactId: string,
    dataUrl: string,
    workspacePath?: string
  ): Promise<InvokeResult> {
    return invokeAction(
      { action: 'set_thumbnail', artifact_id: artifactId, data_url: dataUrl },
      workspacePath
    );
  },

  async zipExport(artifactId: string, workspacePath?: string): Promise<InvokeResult> {
    return invokeAction(
      { action: 'zip_export', artifact_id: artifactId },
      workspacePath
    );
  },

  async archive(artifactId: string, unarchive = false, workspacePath?: string): Promise<InvokeResult> {
    return invokeAction(
      { action: 'archive', artifact_id: artifactId, unarchive },
      workspacePath
    );
  },

  async list(workspacePath?: string): Promise<DesignArtifactManifest[]> {
    try {
      const result = await invokeAction({ action: 'list' }, workspacePath);
      return result.manifests ?? [];
    } catch (err) {
      log.warn('Failed to list design artifacts', err);
      return [];
    }
  },
};

export default designArtifactAPI;
