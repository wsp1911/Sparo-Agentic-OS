import React, { useEffect, useMemo, useState } from 'react';
import { Archive, ChevronDown, FolderOpen, Lock, Pencil, Save, Trash2, X } from 'lucide-react';
import { Markdown, Tooltip } from '@/component-library';
import type { MemoryRecord } from '../MemoryLibraryAPI';
import { getRelatedRecords, getTypeColor } from '../utils/memoryLayout';

interface MemoryDetailDrawerProps {
  record: MemoryRecord | null;
  allRecords: MemoryRecord[];
  workspaceLabels: Record<string, string>;
  isOpen: boolean;
  isSaving: boolean;
  onClose: () => void;
  onSave: (record: MemoryRecord, content: string) => Promise<void>;
  onReveal: (record: MemoryRecord) => void;
  onDelete: (record: MemoryRecord) => void;
  onArchive: (record: MemoryRecord) => void;
  onSelectRelated: (record: MemoryRecord) => void;
  formatDate: (timestamp?: number) => string;
  typeLabel: (type: MemoryRecord['type']) => string;
  scopeLabel: (scope: MemoryRecord['scope']) => string;
  reasonLabel: (reason: 'index' | 'same-folder' | 'cross-scope') => string;
  usageHint: (type: MemoryRecord['type']) => string;
  t: (key: string, vars?: Record<string, unknown>) => string;
}

const MemoryDetailDrawer: React.FC<MemoryDetailDrawerProps> = ({
  record,
  allRecords,
  workspaceLabels,
  isOpen,
  isSaving,
  onClose,
  onSave,
  onReveal,
  onDelete,
  onArchive,
  onSelectRelated,
  formatDate,
  typeLabel,
  scopeLabel,
  reasonLabel,
  usageHint,
  t,
}) => {
  const [isEditing, setIsEditing] = useState(false);
  const [draft, setDraft] = useState('');
  const [relationsOpen, setRelationsOpen] = useState(false);

  const recordId = record?.id;
  const recordContent = record?.content;
  useEffect(() => {
    if (!recordId) return;
    setIsEditing(false);
    setDraft(recordContent ?? '');
    setRelationsOpen(false);
  }, [recordId, recordContent]);

  useEffect(() => {
    if (!isOpen) return undefined;
    const handler = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [isOpen, onClose]);

  const related = useMemo(() => {
    if (!record) return [];
    const workspaceLabel = record.scope === 'workspace'
      ? workspaceLabels[record.memoryDir]
      : Object.values(workspaceLabels)[0];
    return getRelatedRecords(record, allRecords, workspaceLabel);
  }, [record, allRecords, workspaceLabels]);

  if (!record) {
    return (
      <aside className={`memory-drawer${isOpen ? ' is-open' : ''}`} aria-hidden={!isOpen}>
        <div className="memory-drawer__empty">{t('memoryLibrary.empty.noSelection')}</div>
      </aside>
    );
  }

  const canDelete = !record.isIndex;
  const color = getTypeColor(record.type);

  const handleSaveClick = async () => {
    await onSave(record, draft);
    setIsEditing(false);
  };

  return (
    <aside className={`memory-drawer${isOpen ? ' is-open' : ''}`} aria-hidden={!isOpen}>
      <header className="memory-drawer__header">
        <div className="memory-drawer__header-row">
          <div className="memory-drawer__header-tags">
            <Tooltip content={usageHint(record.type)} placement="bottom">
              <span className="memory-drawer__type-chip" style={{ borderColor: color, color }}>
                <span className="memory-drawer__type-dot" style={{ background: color }} />
                {typeLabel(record.type)}
              </span>
            </Tooltip>
            <span className="memory-drawer__scope">{scopeLabel(record.scope)}</span>
            {record.updatedAt ? (
              <span className="memory-drawer__updated">{formatDate(record.updatedAt)}</span>
            ) : null}
          </div>
          <button
            type="button"
            className="memory-drawer__close"
            onClick={onClose}
            aria-label={t('memoryLibrary.actions.cancel')}
          >
            <X size={15} />
          </button>
        </div>

        <div className="memory-drawer__title-row">
          <h3 className="memory-drawer__title">{record.title}</h3>
          <div className="memory-drawer__title-actions">
            {isEditing ? (
              <>
                <Tooltip content={t('memoryLibrary.actions.save')} placement="bottom">
                  <button
                    type="button"
                    className="memory-drawer__icon-btn memory-drawer__icon-btn--primary"
                    onClick={() => void handleSaveClick()}
                    disabled={isSaving}
                    aria-label={t('memoryLibrary.actions.save')}
                  >
                    <Save size={15} />
                  </button>
                </Tooltip>
                <Tooltip content={t('memoryLibrary.actions.cancel')} placement="bottom">
                  <button
                    type="button"
                    className="memory-drawer__icon-btn"
                    onClick={() => {
                      setIsEditing(false);
                      setDraft(record.content ?? '');
                    }}
                    disabled={isSaving}
                    aria-label={t('memoryLibrary.actions.cancel')}
                  >
                    <X size={15} />
                  </button>
                </Tooltip>
              </>
            ) : (
              <>
                <Tooltip content={t('memoryLibrary.actions.edit')} placement="bottom">
                  <button
                    type="button"
                    className="memory-drawer__icon-btn"
                    onClick={() => setIsEditing(true)}
                    aria-label={t('memoryLibrary.actions.edit')}
                  >
                    <Pencil size={15} />
                  </button>
                </Tooltip>
                <Tooltip content={t('memoryLibrary.actions.reveal')} placement="bottom">
                  <button
                    type="button"
                    className="memory-drawer__icon-btn"
                    onClick={() => onReveal(record)}
                    aria-label={t('memoryLibrary.actions.reveal')}
                  >
                    <FolderOpen size={15} />
                  </button>
                </Tooltip>
                {record.status !== 'archived' ? (
                  <Tooltip content={t('memoryLibrary.actions.archive')} placement="bottom">
                    <button
                      type="button"
                      className="memory-drawer__icon-btn"
                      disabled={!canDelete}
                      onClick={() => canDelete && onArchive(record)}
                      aria-label={t('memoryLibrary.actions.archive')}
                    >
                      <Archive size={15} />
                    </button>
                  </Tooltip>
                ) : null}
                <Tooltip content={t('memoryLibrary.actions.forget')} placement="bottom">
                  <button
                    type="button"
                    className="memory-drawer__icon-btn memory-drawer__icon-btn--danger"
                    disabled={!canDelete}
                    onClick={() => canDelete && onDelete(record)}
                    aria-label={t('memoryLibrary.actions.forget')}
                  >
                    <Trash2 size={15} />
                  </button>
                </Tooltip>
              </>
            )}
          </div>
        </div>

        <div className="memory-drawer__path" title={record.path}>{record.relativePath}</div>

        {/* Metadata row: layer / status / strength / sensitivity / tags */}
        <div className="memory-drawer__meta-row">
          {record.layer ? (
            <span className="memory-drawer__meta-chip memory-drawer__meta-chip--layer">
              {record.layer}
            </span>
          ) : null}
          {record.status ? (
            <span className={`memory-drawer__meta-chip memory-drawer__meta-chip--status memory-drawer__meta-chip--status-${record.status}`}>
              {t(`memoryLibrary.statuses.${record.status}`)}
            </span>
          ) : null}
          {record.sensitivity && record.sensitivity !== 'normal' ? (
            <span className="memory-drawer__meta-chip memory-drawer__meta-chip--sensitivity">
              <Lock size={10} />
              {t(`memoryLibrary.sensitivity.${record.sensitivity}`)}
            </span>
          ) : null}
          {typeof record.strength === 'number' ? (
            <span className="memory-drawer__meta-chip memory-drawer__meta-chip--strength">
              <span
                className="memory-drawer__strength-bar"
                style={{ '--strength': record.strength } as React.CSSProperties}
                title={`${t('memoryLibrary.drawer.strength')}: ${Math.round(record.strength * 100)}%`}
              />
            </span>
          ) : null}
          {record.tags?.map((tag) => (
            <span key={tag} className="memory-drawer__meta-chip memory-drawer__meta-chip--tag">
              #{tag}
            </span>
          ))}
        </div>
        {record.sourceSession ? (
          <div className="memory-drawer__source-session">
            {t('memoryLibrary.drawer.sourceSession')}: <code>{record.sourceSession}</code>
          </div>
        ) : null}
      </header>

      <div className="memory-drawer__body">
        {isEditing ? (
          <textarea
            className="memory-drawer__editor"
            value={draft}
            onChange={(event) => setDraft(event.target.value)}
            spellCheck={false}
            autoFocus
          />
        ) : (
          <div className="memory-drawer__markdown">
            <Markdown content={record.content || t('memoryLibrary.empty.emptyFile')} />
          </div>
        )}

        {!isEditing && related.length > 0 ? (
          <section
            className={`memory-drawer__relations-section${relationsOpen ? ' is-open' : ''}`}
          >
            <button
              type="button"
              className="memory-drawer__relations-toggle"
              onClick={() => setRelationsOpen((current) => !current)}
              aria-expanded={relationsOpen}
            >
              <span>{t('memoryLibrary.drawer.tabs.relations')}</span>
              <span className="memory-drawer__relations-count">{related.length}</span>
              <span className="memory-drawer__relations-chevron" aria-hidden>
                <ChevronDown size={14} />
              </span>
            </button>
            {relationsOpen ? (
              <ul className="memory-drawer__relations">
                {related.map(({ record: rel, reason }) => (
                  <li key={rel.id}>
                    <button
                      type="button"
                      className="memory-drawer__relation-item"
                      onClick={() => onSelectRelated(rel)}
                    >
                      <span
                        className="memory-drawer__relation-dot"
                        style={{ background: getTypeColor(rel.type) }}
                      />
                      <span className="memory-drawer__relation-body">
                        <span className="memory-drawer__relation-title">{rel.title}</span>
                        <span className="memory-drawer__relation-meta">
                          {reasonLabel(reason)} · {rel.relativePath}
                        </span>
                      </span>
                    </button>
                  </li>
                ))}
              </ul>
            ) : null}
          </section>
        ) : null}
      </div>
    </aside>
  );
};

export default MemoryDetailDrawer;
