import React, { useCallback, useEffect, useMemo, useState } from 'react';
import {
  AlertTriangle,
  AppWindow,
  ChevronDown,
  ChevronUp,
  ExternalLink,
  Loader2,
  RefreshCw,
  Send,
  Trash2,
} from 'lucide-react';
import { liveAppAPI } from '@/infrastructure/api/service-api/LiveAppAPI';
import { api } from '@/infrastructure/api/service-api/ApiClient';
import type { LiveApp } from '@/infrastructure/api/service-api/LiveAppAPI';
import { useCurrentWorkspace } from '@/infrastructure/contexts/WorkspaceContext';
import { useTheme } from '@/infrastructure/theme/hooks/useTheme';
import { useI18n } from '@/infrastructure/i18n';
import { useOverlayManager } from '@/app/hooks/useOverlayManager';
import type { OverlaySceneId } from '@/app/overlay/types';
import { Button, IconButton } from '@/component-library';
import { flowChatManager } from '@/flow_chat/services/FlowChatManager';
import { notificationService } from '@/shared/notification-system';
import LiveAppRunner from './LiveAppRunner';
import './LiveAppStudioPanel.scss';

interface RuntimeIssue {
  appId: string;
  severity: 'fatal' | 'warning' | 'noise';
  message: string;
  source?: string;
  stack?: string;
  category?: string;
  timestampMs: number;
}

interface LiveAppStudioPanelProps {
  sessionId: string | null;
  appId?: string;
}

const MAX_VISIBLE_ISSUES = 20;

const LiveAppStudioPanel: React.FC<LiveAppStudioPanelProps> = ({ sessionId, appId }) => {
  const { workspacePath } = useCurrentWorkspace();
  const { themeType } = useTheme();
  const { currentLanguage, t } = useI18n('common');
  const { openOverlay } = useOverlayManager();
  const [app, setApp] = useState<LiveApp | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [runnerKey, setRunnerKey] = useState(0);
  const [issues, setIssues] = useState<RuntimeIssue[]>([]);
  const [sendingIssues, setSendingIssues] = useState(false);
  const [clearingIssues, setClearingIssues] = useState(false);
  const [diagnosticsExpanded, setDiagnosticsExpanded] = useState(true);

  const load = useCallback(async () => {
    if (!appId) return;
    setLoading(true);
    setError(null);
    try {
      const loaded = await liveAppAPI.getLiveApp(appId, themeType ?? 'dark', workspacePath || undefined);
      setApp(loaded);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, [appId, themeType, workspacePath]);

  useEffect(() => {
    setApp(null);
    setIssues([]);
    if (appId) {
      void load();
    }
  }, [appId, load]);

  useEffect(() => {
    if (!appId) return;
    const shouldHandle = (payload?: { id?: string }) => payload?.id === appId;
    const reload = (payload?: { id?: string }) => {
      if (!shouldHandle(payload)) return;
      setRunnerKey((value) => value + 1);
      void load();
    };
    const unlistenUpdated = api.listen<{ id?: string }>('liveapp-updated', reload);
    const unlistenRecompiled = api.listen<{ id?: string }>('liveapp-recompiled', reload);
    const unlistenIssue = api.listen<RuntimeIssue>('liveapp-runtime-error', (payload) => {
      if (payload?.appId !== appId) return;
      if (payload.severity === 'noise') return;
      setIssues((current) => [payload, ...current].slice(0, MAX_VISIBLE_ISSUES));
    });
    const unlistenCleared = api.listen<{ appId?: string }>('liveapp-runtime-errors-cleared', (payload) => {
      if (payload?.appId !== appId) return;
      setIssues([]);
    });

    return () => {
      unlistenUpdated();
      unlistenRecompiled();
      unlistenIssue();
      unlistenCleared();
    };
  }, [appId, load]);

  const issueCounts = useMemo(() => {
    return issues.reduce(
      (acc, issue) => {
        if (issue.severity === 'fatal') acc.fatal += 1;
        if (issue.severity === 'warning') acc.warning += 1;
        acc.total += 1;
        return acc;
      },
      { fatal: 0, warning: 0, total: 0 },
    );
  }, [issues]);

  const handleOpenInApps = () => {
    if (appId) {
      openOverlay(`live-app:${appId}` as OverlaySceneId);
    }
  };

  const buildIssuePrompt = useCallback(() => {
    const appLabel = app ? `${app.name} (${app.id})` : appId ?? 'current Live App';
    const lines = issues.slice(0, MAX_VISIBLE_ISSUES).map((issue, index) => {
      const stack = issue.stack ? `\nStack:\n${issue.stack}` : '';
      const source = issue.source ? `\nSource: ${issue.source}` : '';
      return [
        `#${index + 1} [${issue.severity}] ${issue.category ?? 'runtime'}`,
        `Message: ${issue.message}`,
        source.trim(),
        stack.trim(),
      ].filter(Boolean).join('\n');
    });

    return [
      `请修复 Live App 的运行时错误。App: ${appLabel}`,
      '',
      '要求：',
      '- 先定位错误原因，再修改当前 Live App 的 source/meta/package 文件。',
      '- 修改后运行 LiveAppRecompile 和 LiveAppRuntimeProbe。',
      '- Diagnostics 里不能留下 fatal 错误。',
      '',
      '最近错误：',
      lines.join('\n\n---\n\n'),
    ].join('\n');
  }, [app, appId, issues]);

  const handleSendIssuesToAi = useCallback(async () => {
    if (!sessionId || issues.length === 0 || sendingIssues) return;
    setSendingIssues(true);
    try {
      await flowChatManager.sendMessage(buildIssuePrompt(), sessionId, t('liveAppStudio.diagnostics.sendDisplay'));
      notificationService.success(t('liveAppStudio.diagnostics.sent'), { duration: 2000 });
    } catch (err) {
      notificationService.error(err instanceof Error ? err.message : String(err), { duration: 4000 });
    } finally {
      setSendingIssues(false);
    }
  }, [buildIssuePrompt, issues.length, sendingIssues, sessionId, t]);

  const handleClearIssues = useCallback(async () => {
    if (!appId || clearingIssues) return;
    setClearingIssues(true);
    try {
      await liveAppAPI.clearRuntimeIssues(appId);
      setIssues([]);
    } catch (err) {
      notificationService.error(err instanceof Error ? err.message : String(err), { duration: 4000 });
    } finally {
      setClearingIssues(false);
    }
  }, [appId, clearingIssues]);

  return (
    <div
      className={`live-app-studio-panel${diagnosticsExpanded ? '' : ' is-diagnostics-collapsed'}`}
      data-session-id={sessionId ?? ''}
    >
      <div className="live-app-studio-panel__toolbar">
        <div className="live-app-studio-panel__title">
          <AppWindow size={15} />
          <span>{app?.name || t('liveAppStudio.panel.title')}</span>
        </div>
        <div className="live-app-studio-panel__meta">
          <span>{themeType ?? 'dark'}</span>
          <span>{currentLanguage}</span>
        </div>
        <IconButton
          variant="ghost"
          size="small"
          onClick={() => {
            setRunnerKey((value) => value + 1);
            void load();
          }}
          disabled={!appId || loading}
          tooltip={t('liveAppStudio.panel.reload')}
        >
          {loading ? <Loader2 size={14} className="live-app-studio-panel__spin" /> : <RefreshCw size={14} />}
        </IconButton>
        <IconButton
          variant="ghost"
          size="small"
          onClick={handleOpenInApps}
          disabled={!appId}
          tooltip={t('liveAppStudio.panel.openInApps')}
        >
          <ExternalLink size={14} />
        </IconButton>
      </div>

      <div className="live-app-studio-panel__preview">
        {!appId && (
          <div className="live-app-studio-panel__empty">
            <AppWindow size={34} strokeWidth={1.5} />
            <div>{t('liveAppStudio.panel.emptyTitle')}</div>
            <p>{t('liveAppStudio.panel.emptyDescription')}</p>
          </div>
        )}
        {appId && loading && !app && (
          <div className="live-app-studio-panel__empty">
            <Loader2 size={30} className="live-app-studio-panel__spin" />
            <div>{t('liveAppStudio.panel.loading')}</div>
          </div>
        )}
        {error && (
          <div className="live-app-studio-panel__empty is-error">
            <AlertTriangle size={30} strokeWidth={1.5} />
            <div>{t('liveAppStudio.panel.loadFailed')}</div>
            <p>{error}</p>
            <Button variant="secondary" size="small" onClick={() => void load()}>
              {t('liveAppStudio.panel.retry')}
            </Button>
          </div>
        )}
        {app && !error && (
          <React.Suspense fallback={null}>
            <LiveAppRunner key={`${app.id}-${runnerKey}`} app={app} />
          </React.Suspense>
        )}
      </div>

      <div className="live-app-studio-panel__diagnostics">
        <div className="live-app-studio-panel__diagnostics-header">
          <span>{t('liveAppStudio.diagnostics.title')}</span>
          <span className={issueCounts.fatal > 0 ? 'is-fatal' : 'is-ok'}>
            {issueCounts.fatal > 0
              ? t('liveAppStudio.diagnostics.fatalCount', { count: issueCounts.fatal })
              : t('liveAppStudio.diagnostics.ok')}
          </span>
          {issueCounts.warning > 0 && (
            <span className="is-warning">
              {t('liveAppStudio.diagnostics.warningCount', { count: issueCounts.warning })}
            </span>
          )}
          <div className="live-app-studio-panel__diagnostics-actions">
            <span className="live-app-studio-panel__diagnostics-toggle">
              <IconButton
                variant="ghost"
                size="xs"
                onClick={() => setDiagnosticsExpanded((value) => !value)}
                tooltip={
                  diagnosticsExpanded
                    ? t('liveAppStudio.diagnostics.collapse')
                    : t('liveAppStudio.diagnostics.expand')
                }
                aria-label={
                  diagnosticsExpanded
                    ? t('liveAppStudio.diagnostics.collapse')
                    : t('liveAppStudio.diagnostics.expand')
                }
              >
                {diagnosticsExpanded ? <ChevronDown size={12} /> : <ChevronUp size={12} />}
              </IconButton>
              {issueCounts.total > 0 && (
                <span className={issueCounts.fatal > 0 ? 'is-fatal-badge' : 'is-warning-badge'}>
                  {issueCounts.total > 99 ? '99+' : issueCounts.total}
                </span>
              )}
            </span>
            <IconButton
              variant="ghost"
              size="xs"
              onClick={() => void handleSendIssuesToAi()}
              disabled={!sessionId || issues.length === 0 || sendingIssues}
              tooltip={t('liveAppStudio.diagnostics.sendToAi')}
            >
              {sendingIssues ? <Loader2 size={12} className="live-app-studio-panel__spin" /> : <Send size={12} />}
            </IconButton>
            <IconButton
              variant="ghost"
              size="xs"
              onClick={() => void handleClearIssues()}
              disabled={!appId || issues.length === 0 || clearingIssues}
              tooltip={t('liveAppStudio.diagnostics.clear')}
            >
              {clearingIssues ? <Loader2 size={12} className="live-app-studio-panel__spin" /> : <Trash2 size={12} />}
            </IconButton>
          </div>
        </div>
        {diagnosticsExpanded && (
          <div className="live-app-studio-panel__diagnostics-list">
            {issues.length === 0 ? (
              <div className="live-app-studio-panel__diagnostics-empty">
                {t('liveAppStudio.diagnostics.empty')}
              </div>
            ) : (
              issues.map((issue, index) => (
                <div key={`${issue.timestampMs}-${index}`} className={`live-app-studio-panel__issue is-${issue.severity}`}>
                  <span>{issue.category ?? issue.severity}</span>
                  <div className="live-app-studio-panel__issue-body">
                    <p title={issue.stack || issue.message}>{issue.message}</p>
                    {(issue.source || issue.stack) && (
                      <pre>
                        {[issue.source, issue.stack].filter(Boolean).join('\n')}
                      </pre>
                    )}
                  </div>
                </div>
              ))
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export default LiveAppStudioPanel;
