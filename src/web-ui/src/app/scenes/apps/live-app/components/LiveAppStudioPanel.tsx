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

interface RuntimeLog {
  appId: string;
  level: 'debug' | 'info' | 'warn' | 'error';
  category: string;
  message: string;
  source?: string;
  stack?: string;
  details?: unknown;
  timestampMs: number;
}

interface LiveAppStudioPanelProps {
  sessionId: string | null;
  appId?: string;
}

const MAX_VISIBLE_ISSUES = 20;
const MAX_VISIBLE_LOGS = 100;

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
  const [logs, setLogs] = useState<RuntimeLog[]>([]);
  const [runtimeView, setRuntimeView] = useState<'issues' | 'logs'>('issues');
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
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      void liveAppAPI.reportRuntimeLog({
        appId,
        level: 'error',
        category: 'studio:preview',
        message: `Failed to load Live App preview: ${message}`,
      }).catch(() => undefined);
    } finally {
      setLoading(false);
    }
  }, [appId, themeType, workspacePath]);

  useEffect(() => {
    setApp(null);
    setIssues([]);
    setLogs([]);
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
    const reloadAfterRecompile = (payload?: { id?: string }) => {
      if (!shouldHandle(payload)) return;
      setIssues([]);
      reload(payload);
    };
    const unlistenUpdated = api.listen<{ id?: string }>('liveapp-updated', reload);
    const unlistenRecompiled = api.listen<{ id?: string }>('liveapp-recompiled', reloadAfterRecompile);
    const unlistenIssue = api.listen<RuntimeIssue>('liveapp-runtime-error', (payload) => {
      if (payload?.appId !== appId) return;
      if (payload.severity === 'noise') return;
      setIssues((current) => [payload, ...current].slice(0, MAX_VISIBLE_ISSUES));
    });
    const unlistenLog = api.listen<RuntimeLog>('liveapp-runtime-log', (payload) => {
      if (payload?.appId !== appId) return;
      setLogs((current) => [payload, ...current].slice(0, MAX_VISIBLE_LOGS));
    });
    const unlistenCleared = api.listen<{ appId?: string }>('liveapp-runtime-errors-cleared', (payload) => {
      if (payload?.appId !== appId) return;
      setIssues([]);
      setLogs([]);
    });

    return () => {
      unlistenUpdated();
      unlistenRecompiled();
      unlistenIssue();
      unlistenLog();
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

  const visibleLogs = useMemo(() => {
    return logs.filter((entry) => entry.level !== 'debug');
  }, [logs]);

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

    const logLines = visibleLogs.slice(0, 40).map((entry, index) => {
      const details = entry.details ? `\nDetails: ${JSON.stringify(entry.details)}` : '';
      const stack = entry.stack ? `\nStack:\n${entry.stack}` : '';
      const source = entry.source ? `\nSource: ${entry.source}` : '';
      return [
        `#${index + 1} [${entry.level}] ${entry.category}`,
        `Message: ${entry.message}`,
        source.trim(),
        details.trim(),
        stack.trim(),
      ].filter(Boolean).join('\n');
    });

    return [
      `请根据 Live App Studio 的运行诊断修复当前 Live App。App: ${appLabel}`,
      '',
      '要求：',
      '- 先结合 fatal/warning 和最近 runtime logs 判断根因，再修改当前 Live App 的 source/meta/package 文件。',
      '- 修改后运行 LiveAppRecompile 和 LiveAppRuntimeProbe。',
      '- 如果没有 fatal 但行为仍异常，请用 LiveAppRuntimeProbe({ include_logs: true, tail: 80 }) 读取最近日志。',
      '- Runtime 里不能留下 fatal 错误。',
      '',
      '最近错误：',
      lines.length > 0 ? lines.join('\n\n---\n\n') : '暂无 fatal/warning issue。',
      '',
      '最近日志：',
      logLines.length > 0 ? logLines.join('\n\n---\n\n') : '暂无 runtime log。',
    ].join('\n');
  }, [app, appId, issues, visibleLogs]);

  const handleSendIssuesToAi = useCallback(async () => {
    if (!sessionId || (issues.length === 0 && visibleLogs.length === 0) || sendingIssues) return;
    setSendingIssues(true);
    try {
      await flowChatManager.sendMessage(buildIssuePrompt(), sessionId, t('liveAppStudio.diagnostics.sendDisplay'));
      notificationService.success(t('liveAppStudio.diagnostics.sent'), { duration: 2000 });
    } catch (err) {
      notificationService.error(err instanceof Error ? err.message : String(err), { duration: 4000 });
    } finally {
      setSendingIssues(false);
    }
  }, [buildIssuePrompt, issues.length, sendingIssues, sessionId, t, visibleLogs.length]);

  const handleClearIssues = useCallback(async () => {
    if (!appId || clearingIssues) return;
    setClearingIssues(true);
    try {
      await liveAppAPI.clearRuntimeIssues(appId);
      setIssues([]);
      setLogs([]);
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
              disabled={!sessionId || (issues.length === 0 && visibleLogs.length === 0) || sendingIssues}
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
          <>
            <div className="live-app-studio-panel__runtime-tabs">
              <button
                type="button"
                className={runtimeView === 'issues' ? 'is-active' : ''}
                onClick={() => setRuntimeView('issues')}
              >
                {t('liveAppStudio.diagnostics.issuesTab')}
              </button>
              <button
                type="button"
                className={runtimeView === 'logs' ? 'is-active' : ''}
                onClick={() => setRuntimeView('logs')}
              >
                {t('liveAppStudio.diagnostics.logsTab')}
              </button>
            </div>
            <div className="live-app-studio-panel__diagnostics-list">
              {runtimeView === 'issues' && (
                issues.length === 0 ? (
                  <div className="live-app-studio-panel__diagnostics-empty">
                    {t('liveAppStudio.diagnostics.empty')}
                  </div>
                ) : (
                  issues.map((issue, index) => (
                    <div key={`${issue.timestampMs}-${index}`} className={`live-app-studio-panel__issue is-${issue.severity}`}>
                      <span className="live-app-studio-panel__issue-category">{issue.category ?? issue.severity}</span>
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
                )
              )}
              {runtimeView === 'logs' && (
                visibleLogs.length === 0 ? (
                  <div className="live-app-studio-panel__diagnostics-empty">
                    {t('liveAppStudio.diagnostics.logsEmpty')}
                  </div>
                ) : (
                  visibleLogs.map((entry, index) => (
                    <div key={`${entry.timestampMs}-${index}`} className={`live-app-studio-panel__issue is-${entry.level}`}>
                      <span className="live-app-studio-panel__issue-category">
                        {entry.level} / {entry.category}
                      </span>
                      <div className="live-app-studio-panel__issue-body">
                        <p title={entry.stack || entry.message}>{entry.message}</p>
                        {!!(entry.source || entry.stack || entry.details != null) && (
                          <pre>
                            {[entry.source, entry.details != null ? JSON.stringify(entry.details) : undefined, entry.stack]
                              .filter(Boolean)
                              .join('\n')}
                          </pre>
                        )}
                      </div>
                    </div>
                  ))
                )
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
};

export default LiveAppStudioPanel;
