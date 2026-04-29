import React, { useCallback, useEffect, useMemo, useState } from 'react';
import {
  LayoutList,
  RefreshCcw,
  Settings,
  X,
} from 'lucide-react';
import { ConfirmDialog, Search } from '@/component-library';
import { useI18n } from '@/infrastructure/i18n/hooks/useI18n';
import { useCurrentWorkspace } from '@/infrastructure/contexts/WorkspaceContext';
import { WorkspaceKind } from '@/shared/types';
import { notificationService } from '@/shared/notification-system';
import { useOverlayManager } from '../../hooks/useOverlayManager';
import { useSettingsStore } from '../settings/settingsStore';
import {
  memoryLibraryAPI,
  type MemoryRecord,
  type MemoryRecordType,
  type MemoryScopeKey,
  type MemorySpace,
} from './MemoryLibraryAPI';
import MemoryGraph from './components/MemoryGraph';
import MemoryList from './components/MemoryList';
import MemoryDetailDrawer from './components/MemoryDetailDrawer';
import { TYPE_COLORS } from './utils/memoryLayout';
import './MemoryScene.scss';

type TypeFilter = 'all' | MemoryRecordType;

const MEMORY_TYPES: TypeFilter[] = [
  'all',
  'index',
  'user',
  'feedback',
  'project',
  'reference',
  'workspace_overview',
  'unknown',
];

function formatDate(timestamp?: number): string {
  if (!timestamp) return '';
  return new Intl.DateTimeFormat(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(timestamp));
}

const deriveWorkspaceLabel = (
  workspaceName: string | undefined,
  workspacePath: string | undefined,
  memoryDir: string | undefined,
  fallback: string,
): string => {
  if (workspaceName && workspaceName.trim()) return workspaceName;
  const candidate = (workspacePath || memoryDir || '').replace(/\\/g, '/').replace(/\/+$/, '');
  if (candidate) {
    const tail = candidate.split('/').pop();
    if (tail) return tail;
  }
  return fallback;
};

const MemoryScene: React.FC = () => {
  const { t } = useI18n('common');
  const { workspace, workspacePath, workspaceName, hasWorkspace } = useCurrentWorkspace();
  const { openOverlay } = useOverlayManager();
  const setSettingsTab = useSettingsStore((state) => state.setActiveTab);

  const [records, setRecords] = useState<MemoryRecord[]>([]);
  const [spaces, setSpaces] = useState<MemorySpace[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [query, setQuery] = useState('');
  const [typeFilter, setTypeFilter] = useState<TypeFilter>('all');
  const [listOpen, setListOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<MemoryRecord | null>(null);

  const loadRecords = useCallback(async () => {
    setIsLoading(true);
    try {
      const storagePaths = await memoryLibraryAPI.getStoragePaths();

      const nextSpaces: MemorySpace[] = [
        {
          scope: 'global',
          label: t('memoryLibrary.scopes.global'),
          memoryDir: storagePaths.agenticOsMemoryDir,
          available: true,
        },
      ];

      if (hasWorkspace && workspacePath && workspace?.workspaceKind !== WorkspaceKind.Remote) {
        try {
          const projectPaths = await memoryLibraryAPI.getProjectStoragePaths(workspacePath);
          nextSpaces.push({
            scope: 'workspace',
            label: t('memoryLibrary.scopes.workspace'),
            memoryDir: projectPaths.memoryDir,
            available: true,
          });
        } catch {
          nextSpaces.push({
            scope: 'workspace',
            label: t('memoryLibrary.scopes.workspace'),
            memoryDir: '',
            available: false,
          });
        }
      }

      const nextRecords = (await Promise.all(
        nextSpaces.map((space) => memoryLibraryAPI.listMemoryRecords(space)),
      )).flat();

      setSpaces(nextSpaces);
      setRecords(nextRecords);
    } catch (_error) {
      notificationService.error(t('memoryLibrary.messages.loadFailed'));
    } finally {
      setIsLoading(false);
    }
  }, [hasWorkspace, t, workspace, workspacePath]);

  useEffect(() => {
    void loadRecords();
  }, [loadRecords]);

  const workspaceLabels = useMemo(() => {
    const map: Record<string, string> = {};
    for (const space of spaces) {
      if (space.scope !== 'workspace' || !space.memoryDir) continue;
      map[space.memoryDir] = deriveWorkspaceLabel(
        workspaceName,
        workspacePath,
        space.memoryDir,
        space.label,
      );
    }
    return map;
  }, [spaces, workspaceName, workspacePath]);

  const globalLabel = t('memoryLibrary.scopes.global');

  const filteredRecords = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();
    return records.filter((record) => {
      if (typeFilter !== 'all' && record.type !== typeFilter) return false;
      if (!normalizedQuery) return true;
      const haystack = [
        record.title,
        record.description,
        record.relativePath,
        record.type,
        record.scope,
        record.content,
      ].join(' ').toLowerCase();
      return haystack.includes(normalizedQuery);
    });
  }, [query, records, typeFilter]);

  const highlightedIds = useMemo<Set<string> | undefined>(() => {
    if (typeFilter === 'all' && !query.trim()) return undefined;
    return new Set(filteredRecords.map((record) => record.id));
  }, [filteredRecords, query, typeFilter]);

  const selectedRecord = useMemo(
    () => records.find((record) => record.id === selectedId) ?? null,
    [records, selectedId],
  );

  const counts = useMemo(() => ({
    total: records.length,
    global: records.filter((r) => r.scope === 'global').length,
    workspace: records.filter((r) => r.scope === 'workspace').length,
    workspaceCount: new Set(
      records.filter((r) => r.scope === 'workspace').map((r) => r.memoryDir),
    ).size,
  }), [records]);

  const handleSelect = useCallback((record: MemoryRecord) => {
    setSelectedId(record.id);
    setDrawerOpen(true);
  }, []);

  const handleClearSelection = useCallback(() => {
    setSelectedId(null);
    setDrawerOpen(false);
  }, []);

  const handleCloseDrawer = useCallback(() => {
    setDrawerOpen(false);
  }, []);

  const handleSave = useCallback(async (record: MemoryRecord, content: string) => {
    setIsSaving(true);
    try {
      const refreshed = await memoryLibraryAPI.saveMemoryRecord(record, content);
      setRecords((current) => current.map((item) => (
        item.id === record.id ? refreshed : item
      )));
      setSelectedId(refreshed.id);
      notificationService.success(t('memoryLibrary.messages.saveSuccess'));
    } catch {
      notificationService.error(t('memoryLibrary.messages.saveFailed'));
    } finally {
      setIsSaving(false);
    }
  }, [t]);

  const handleReveal = useCallback(async (record: MemoryRecord) => {
    try {
      await memoryLibraryAPI.revealMemoryRecord(record);
    } catch {
      notificationService.error(t('memoryLibrary.messages.revealFailed'));
    }
  }, [t]);

  const handleDeleteConfirmed = useCallback(async (record: MemoryRecord) => {
    try {
      await memoryLibraryAPI.deleteMemoryRecord(record);
      setRecords((current) => current.filter((item) => item.id !== record.id));
      setSelectedId((current) => (current === record.id ? null : current));
      notificationService.success(t('memoryLibrary.messages.deleteSuccess'));
    } catch {
      notificationService.error(t('memoryLibrary.messages.deleteFailed'));
    }
  }, [t]);

  const handleOpenSettings = () => {
    setSettingsTab('memory');
    openOverlay('settings');
  };

  const typeLabel = useCallback(
    (type: MemoryRecordType) => t(`memoryLibrary.types.${type}`),
    [t],
  );
  const scopeLabel = useCallback(
    (scope: MemoryScopeKey) => t(`memoryLibrary.scopes.${scope}`),
    [t],
  );
  const reasonLabel = useCallback(
    (reason: 'index' | 'same-folder' | 'cross-scope') => t(`memoryLibrary.drawer.relations.reasons.${reason}`),
    [t],
  );
  const usageHint = useCallback(
    (type: MemoryRecordType) => t(`memoryLibrary.usageHints.${type}`),
    [t],
  );

  const headerSubtitle = isLoading
    ? t('memoryLibrary.loading')
    : t('memoryLibrary.overview.summary', {
        total: counts.total,
        global: counts.global,
        workspaces: counts.workspaceCount,
      });

  return (
    <div className="memory-scene">
      <div className="memory-scene__stage">
        <header className="memory-scene__header">
          <div className="memory-scene__header-text">
            <h2>{t('memoryLibrary.title')}</h2>
            <div className="memory-scene__header-subline">
              <p>{headerSubtitle}</p>
            </div>
          </div>
        </header>

        <div className="memory-scene__toolbar">
          <div className="memory-scene__toolbar-main">
            <button
              type="button"
              className={`memory-scene__icon-btn${listOpen ? ' is-active' : ''}`}
              onClick={() => setListOpen((current) => !current)}
              aria-label={t('memoryLibrary.overview.modes.list')}
              title={t('memoryLibrary.overview.modes.list')}
            >
              <LayoutList size={15} />
            </button>

            <div className="memory-scene__search">
              <Search
                value={query}
                onChange={setQuery}
                onClear={() => setQuery('')}
                placeholder={t('memoryLibrary.searchPlaceholder')}
                size="medium"
              />
            </div>

            <div className="memory-scene__type-pills">
              {MEMORY_TYPES.map((type) => (
                <button
                  key={type}
                  type="button"
                  className={`memory-scene__type-pill${typeFilter === type ? ' is-active' : ''}`}
                  onClick={() => setTypeFilter(type)}
                  style={
                    type !== 'all'
                      ? ({ '--pill-color': TYPE_COLORS[type as MemoryRecordType] } as React.CSSProperties)
                      : undefined
                  }
                >
                  {type !== 'all' ? (
                    <span
                      className="memory-scene__type-pill-dot"
                      style={{ background: TYPE_COLORS[type as MemoryRecordType] }}
                    />
                  ) : null}
                  {t(`memoryLibrary.types.${type}`)}
                </button>
              ))}
            </div>
          </div>

          <div className="memory-scene__header-actions">
            <button
              type="button"
              className="memory-scene__icon-btn"
              onClick={() => void loadRecords()}
              aria-label={t('memoryLibrary.actions.refresh')}
              title={t('memoryLibrary.actions.refresh')}
            >
              <RefreshCcw size={15} />
            </button>
            <button
              type="button"
              className="memory-scene__icon-btn"
              onClick={handleOpenSettings}
              aria-label={t('memoryLibrary.actions.openSettings')}
              title={t('memoryLibrary.actions.openSettings')}
            >
              <Settings size={15} />
            </button>
          </div>
        </div>

        <div className="memory-scene__canvas">
          <MemoryGraph
            records={records}
            workspaceLabels={workspaceLabels}
            globalLabel={globalLabel}
            selectedId={selectedId}
            highlightedIds={highlightedIds}
            onSelect={handleSelect}
            onClearSelection={handleClearSelection}
            emptyMessage={
              isLoading ? t('memoryLibrary.loading') : t('memoryLibrary.empty.noResults')
            }
          />

          <aside
            className={`memory-scene__list-panel${listOpen ? ' is-open' : ''}`}
            aria-hidden={!listOpen}
          >
            <header className="memory-scene__list-panel-header">
              <span className="memory-scene__list-panel-title">
                {t('memoryLibrary.overview.modes.list')}
              </span>
              <span className="memory-scene__list-panel-count">
                {filteredRecords.length}
              </span>
              <button
                type="button"
                className="memory-scene__list-panel-close"
                onClick={() => setListOpen(false)}
                aria-label={t('memoryLibrary.actions.cancel')}
              >
                <X size={14} />
              </button>
            </header>
            <div className="memory-scene__list-panel-body">
              <MemoryList
                records={filteredRecords}
                workspaceLabels={workspaceLabels}
                globalLabel={globalLabel}
                selectedId={selectedId}
                onSelect={handleSelect}
                onDelete={(record) => setDeleteTarget(record)}
                emptyMessage={
                  isLoading ? t('memoryLibrary.loading') : t('memoryLibrary.empty.noResults')
                }
                formatDate={formatDate}
              />
            </div>
          </aside>

          <MemoryDetailDrawer
            record={selectedRecord}
            allRecords={records}
            workspaceLabels={workspaceLabels}
            isOpen={drawerOpen}
            isSaving={isSaving}
            onClose={handleCloseDrawer}
            onSave={handleSave}
            onReveal={(record) => void handleReveal(record)}
            onDelete={(record) => setDeleteTarget(record)}
            onSelectRelated={handleSelect}
            formatDate={formatDate}
            typeLabel={typeLabel}
            scopeLabel={scopeLabel}
            reasonLabel={reasonLabel}
            usageHint={usageHint}
            t={t}
          />
        </div>
      </div>

      <ConfirmDialog
        isOpen={Boolean(deleteTarget)}
        onClose={() => setDeleteTarget(null)}
        onConfirm={() => deleteTarget && void handleDeleteConfirmed(deleteTarget)}
        title={t('memoryLibrary.deleteDialog.title')}
        message={t('memoryLibrary.deleteDialog.message', { name: deleteTarget?.title ?? '' })}
        confirmText={t('memoryLibrary.actions.forget')}
        confirmDanger
        preview={deleteTarget?.relativePath}
      />
    </div>
  );
};

export default MemoryScene;
