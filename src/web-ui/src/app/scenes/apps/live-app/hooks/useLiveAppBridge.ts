/**
 * useLiveAppBridge — handles postMessage JSON-RPC from the Live App iframe:
 * worker.call → JS Worker, dialog.open/save/message → Tauri dialog,
 * ai.* → Host AI client, clipboard.* → Host navigator.clipboard.
 * Also handles bitfun/request-theme and pushes theme changes to the iframe.
 */
import { useLayoutEffect, useRef, useEffect, RefObject } from 'react';
import { liveAppAPI } from '@/infrastructure/api/service-api/LiveAppAPI';
import { open as dialogOpen, save as dialogSave, message as dialogMessage } from '@tauri-apps/plugin-dialog';
import type { LiveApp } from '@/infrastructure/api/service-api/LiveAppAPI';
import { useCurrentWorkspace } from '@/infrastructure/contexts/WorkspaceContext';
import { useTheme } from '@/infrastructure/theme/hooks/useTheme';
import { useI18n } from '@/infrastructure/i18n';
import { buildLiveAppThemeVars } from '../buildLiveAppThemeVars';
import { api } from '@/infrastructure/api/service-api/ApiClient';

interface JSONRPC {
  jsonrpc?: string;
  id: number | string;
  method: string;
  params?: Record<string, unknown>;
}

interface AiStreamPayload {
  appId: string;
  streamId: string;
  type: 'chunk' | 'done' | 'error';
  data: Record<string, unknown>;
}

interface RuntimeIssuePayload {
  appId?: string;
  severity?: 'fatal' | 'warning' | 'noise';
  message?: string;
  source?: string;
  stack?: string;
  category?: string;
  timestampMs?: number;
}

const NOOP_BRIDGE_METHODS = new Set([
  // Emitted by the injected scroll-boundary script when iframe scrolling reaches an edge.
  'bitfun/sandbox-wheel',
]);

function safeStringify(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === 'string') return error;
  return String(error);
}

function errorStack(error: unknown): string | undefined {
  return error instanceof Error ? error.stack : undefined;
}

export function useLiveAppBridge(
  iframeRef: RefObject<HTMLIFrameElement>,
  app: LiveApp,
) {
  const { workspacePath } = useCurrentWorkspace();
  const { theme: currentTheme } = useTheme();
  const { currentLanguage } = useI18n();
  const themeRef = useRef(currentTheme);
  themeRef.current = currentTheme;
  const localeRef = useRef(currentLanguage);
  localeRef.current = currentLanguage;
  const workspacePathRef = useRef(workspacePath);
  workspacePathRef.current = workspacePath;

  const appIdRef = useRef(app.id);
  useLayoutEffect(() => {
    appIdRef.current = app.id;
  }, [app.id]);

  useLayoutEffect(() => {
    const handler = async (event: MessageEvent) => {
      if (!iframeRef.current || event.source !== iframeRef.current.contentWindow) return;
      const msg = event.data as JSONRPC & { method?: string };
      if (!msg?.method) return;

      const { id, method, params = {} } = msg;
      const appId = appIdRef.current;
      const bridgeContext = [
        `appId: ${appId}`,
        `method: ${method}`,
        `params: ${safeStringify(params)}`,
        `message: ${safeStringify(msg)}`,
      ].join('\n');
      const reply = (result: unknown) =>
        iframeRef.current?.contentWindow?.postMessage({ jsonrpc: '2.0', id, result }, '*');
      const replyError = (message: string) =>
        iframeRef.current?.contentWindow?.postMessage(
          { jsonrpc: '2.0', id, error: { code: -32000, message } },
          '*',
        );

      if (method === 'bitfun/request-theme') {
        const payload = buildLiveAppThemeVars(themeRef.current);
        if (payload && iframeRef.current?.contentWindow) {
          iframeRef.current.contentWindow.postMessage(
            { type: 'bitfun:event', event: 'themeChange', payload },
            '*',
          );
        }
        return;
      }

      if (method === 'bitfun/request-locale') {
        if (iframeRef.current?.contentWindow) {
          iframeRef.current.contentWindow.postMessage(
            { type: 'bitfun:event', event: 'localeChange', payload: { locale: localeRef.current } },
            '*',
          );
        }
        return;
      }

      if (method === 'bitfun/runtime-error') {
        const issue = params as RuntimeIssuePayload;
        void liveAppAPI.reportRuntimeIssue({
          appId,
          severity: issue.severity ?? 'fatal',
          message: issue.message ?? 'Unknown runtime error',
          source: issue.source,
          stack: issue.stack,
          category: issue.category ?? 'runtime',
          timestampMs: issue.timestampMs ?? Date.now(),
        }).catch(() => undefined);
        return;
      }

      if (NOOP_BRIDGE_METHODS.has(method)) {
        return;
      }

      try {
        if (method === 'worker.call') {
          const result = await liveAppAPI.workerCall(
            appId,
            (params.method as string) ?? '',
            (params.params as Record<string, unknown>) ?? {},
            workspacePathRef.current || undefined,
          );
          reply(result);
          return;
        }
        if (method === 'dialog.open') {
          reply(await dialogOpen(params as unknown as Parameters<typeof dialogOpen>[0]));
          return;
        }
        if (method === 'dialog.save') {
          reply(await dialogSave(params as unknown as Parameters<typeof dialogSave>[0]));
          return;
        }
        if (method === 'dialog.message') {
          reply(await dialogMessage(params as unknown as Parameters<typeof dialogMessage>[0]));
          return;
        }

        if (method === 'ai.complete') {
          const result = await liveAppAPI.aiComplete(appId, (params.prompt as string) ?? '', {
            systemPrompt: params.systemPrompt as string | undefined,
            model: params.model as string | undefined,
            maxTokens: params.maxTokens as number | undefined,
            temperature: params.temperature as number | undefined,
          });
          reply(result);
          return;
        }
        if (method === 'ai.chat') {
          const result = await liveAppAPI.aiChat(
            appId,
            (params.messages as { role: 'user' | 'assistant'; content: string }[]) ?? [],
            (params.streamId as string) ?? '',
            {
              systemPrompt: params.systemPrompt as string | undefined,
              model: params.model as string | undefined,
              maxTokens: params.maxTokens as number | undefined,
              temperature: params.temperature as number | undefined,
            },
          );
          reply(result);
          return;
        }
        if (method === 'ai.cancel') {
          await liveAppAPI.aiCancel(appId, (params.streamId as string) ?? '');
          reply(null);
          return;
        }
        if (method === 'ai.getModels') {
          const models = await liveAppAPI.aiListModels(appId);
          reply(models);
          return;
        }

        if (method === 'clipboard.writeText') {
          await navigator.clipboard.writeText((params.text as string) ?? '');
          reply(null);
          return;
        }
        if (method === 'clipboard.readText') {
          const text = await navigator.clipboard.readText();
          reply(text);
          return;
        }

        const message = `Unknown method: ${method}`;
        void liveAppAPI.reportRuntimeIssue({
          appId,
          severity: 'warning',
          message,
          source: `bridge:${method}`,
          stack: `Unsupported Live App bridge call.\n${bridgeContext}`,
          category: 'bridge:unknown-method',
          timestampMs: Date.now(),
        }).catch(() => undefined);
        replyError(message);
      } catch (error) {
        const message = `Bridge call failed: ${method}: ${errorMessage(error)}`;
        void liveAppAPI.reportRuntimeIssue({
          appId,
          severity: 'fatal',
          message,
          source: `bridge:${method}`,
          stack: [
            errorStack(error),
            bridgeContext,
          ].filter(Boolean).join('\n\n'),
          category: `bridge:${method}`,
          timestampMs: Date.now(),
        }).catch(() => undefined);
        replyError(message);
      }
    };
    window.addEventListener('message', handler);
    return () => {
      window.removeEventListener('message', handler);
    };
  }, [iframeRef]);

  useEffect(() => {
    const payload = buildLiveAppThemeVars(currentTheme);
    if (!payload || !iframeRef.current?.contentWindow) return;
    iframeRef.current.contentWindow.postMessage(
      { type: 'bitfun:event', event: 'themeChange', payload },
      '*',
    );
  }, [currentTheme, iframeRef]);

  useEffect(() => {
    if (!currentLanguage || !iframeRef.current?.contentWindow) return;
    iframeRef.current.contentWindow.postMessage(
      { type: 'bitfun:event', event: 'localeChange', payload: { locale: currentLanguage } },
      '*',
    );
  }, [currentLanguage, iframeRef]);

  useEffect(() => {
    const currentAppId = app.id;
    const unlisten = api.listen<AiStreamPayload>('liveapp://ai-stream', (payload) => {
      if (!iframeRef.current?.contentWindow) return;
      if (payload.appId !== currentAppId) return;
      iframeRef.current.contentWindow.postMessage(
        {
          type: 'bitfun:event',
          event: 'ai:stream',
          payload: {
            streamId: payload.streamId,
            type: payload.type,
            data: payload.data,
          },
        },
        '*',
      );
    });

    return () => {
      unlisten();
    };
  }, [app.id, iframeRef]);

  useEffect(() => {
    const currentAppId = app.id;
    const eventName = `liveapp://worker-event:${currentAppId}`;
    const unlisten = api.listen<{ appId: string; event: string; data: unknown }>(
      eventName,
      (payload) => {
        if (!iframeRef.current?.contentWindow) return;
        iframeRef.current.contentWindow.postMessage(
          {
            type: 'bitfun:event',
            event: 'worker:event',
            payload: {
              event: payload.event,
              data: payload.data,
            },
          },
          '*',
        );
      },
    );

    return () => {
      unlisten();
    };
  }, [app.id, iframeRef]);
}
