import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { ConfigPageLoading, ConfigPageMessage, NumberInput, Select, Switch } from '@/component-library';
import { createLogger } from '@/shared/utils/logger';
import { configManager } from '../services/ConfigManager';
import type { AppHostScanConfig } from '../types';
import {
  ConfigPageContent,
  ConfigPageHeader,
  ConfigPageLayout,
  ConfigPageRow,
  ConfigPageSection,
} from './common';
import './AIFeaturesConfig.scss';

const log = createLogger('MemoryConfig');

type AutoMemoryScopeKey = 'global' | 'workspace';

type AutoMemoryScopeState = {
  enabled: boolean;
  extractEveryEligibleTurns: number;
};

type AutoMemoryState = Record<AutoMemoryScopeKey, AutoMemoryScopeState>;

const DEFAULT_AUTO_MEMORY_STATE: AutoMemoryState = {
  global: {
    enabled: true,
    extractEveryEligibleTurns: 6,
  },
  workspace: {
    enabled: true,
    extractEveryEligibleTurns: 1,
  },
};

const AUTO_MEMORY_CONFIG_PATHS = {
  global: {
    enabled: 'ai.auto_memory.global.enabled',
    extractEveryEligibleTurns: 'ai.auto_memory.global.extract_every_eligible_turns',
  },
  workspace: {
    enabled: 'ai.auto_memory.workspace.enabled',
    extractEveryEligibleTurns: 'ai.auto_memory.workspace.extract_every_eligible_turns',
  },
} as const;

const AUTO_MEMORY_SCOPES: AutoMemoryScopeKey[] = ['global', 'workspace'];

const DEFAULT_EXTRACT_EVERY_ELIGIBLE_TURNS =
  DEFAULT_AUTO_MEMORY_STATE.workspace.extractEveryEligibleTurns;
const normalizeExtractEveryEligibleTurns = (value: number) =>
  Math.max(DEFAULT_EXTRACT_EVERY_ELIGIBLE_TURNS, value);

const DEFAULT_HOST_SCAN_CONFIG: AppHostScanConfig = {
  auto_scan_enabled: false,
  auto_scan_interval_days: 7,
};

const MemoryConfig: React.FC = () => {
  const { t } = useTranslation('settings/memory');
  const [isLoading, setIsLoading] = useState(true);
  const [autoMemoryState, setAutoMemoryState] = useState<AutoMemoryState>(
    DEFAULT_AUTO_MEMORY_STATE
  );
  const [hostScanConfig, setHostScanConfig] = useState<AppHostScanConfig>(DEFAULT_HOST_SCAN_CONFIG);
  const [isSaving, setIsSaving] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);
  const messageTimeoutRef = useRef<number | null>(null);

  const hostScanIntervalOptions = useMemo(
    () => [
      { value: '1', label: t('hostScan.intervals.daily') },
      { value: '3', label: t('hostScan.intervals.every3Days') },
      { value: '7', label: t('hostScan.intervals.weekly') },
      { value: '14', label: t('hostScan.intervals.every2Weeks') },
      { value: '30', label: t('hostScan.intervals.monthly') },
    ],
    [t]
  );

  const showMessage = useCallback((type: 'success' | 'error', text: string) => {
    setMessage({ type, text });
    if (messageTimeoutRef.current !== null) {
      window.clearTimeout(messageTimeoutRef.current);
    }
    messageTimeoutRef.current = window.setTimeout(() => {
      setMessage(null);
      messageTimeoutRef.current = null;
    }, 3000);
  }, []);

  useEffect(() => {
    let cancelled = false;

    const loadConfig = async () => {
      setIsLoading(true);
      try {
        const [
          loadedGlobalEnabled,
          loadedGlobalThreshold,
          loadedWorkspaceEnabled,
          loadedWorkspaceThreshold,
          loadedHostScanConfig,
        ] = await Promise.all([
          configManager.getConfig<boolean>(AUTO_MEMORY_CONFIG_PATHS.global.enabled),
          configManager.getConfig<number>(AUTO_MEMORY_CONFIG_PATHS.global.extractEveryEligibleTurns),
          configManager.getConfig<boolean>(AUTO_MEMORY_CONFIG_PATHS.workspace.enabled),
          configManager.getConfig<number>(
            AUTO_MEMORY_CONFIG_PATHS.workspace.extractEveryEligibleTurns
          ),
          configManager.getConfig<AppHostScanConfig>('app.host_scan'),
        ]);

        if (cancelled) {
          return;
        }

        setAutoMemoryState({
          global: {
            enabled: loadedGlobalEnabled ?? DEFAULT_AUTO_MEMORY_STATE.global.enabled,
            extractEveryEligibleTurns: normalizeExtractEveryEligibleTurns(
              loadedGlobalThreshold ?? DEFAULT_AUTO_MEMORY_STATE.global.extractEveryEligibleTurns
            ),
          },
          workspace: {
            enabled: loadedWorkspaceEnabled ?? DEFAULT_AUTO_MEMORY_STATE.workspace.enabled,
            extractEveryEligibleTurns: normalizeExtractEveryEligibleTurns(
              loadedWorkspaceThreshold ??
                DEFAULT_AUTO_MEMORY_STATE.workspace.extractEveryEligibleTurns
            ),
          },
        });
        setHostScanConfig({
          auto_scan_enabled:
            loadedHostScanConfig?.auto_scan_enabled ?? DEFAULT_HOST_SCAN_CONFIG.auto_scan_enabled,
          auto_scan_interval_days:
            loadedHostScanConfig?.auto_scan_interval_days ??
            DEFAULT_HOST_SCAN_CONFIG.auto_scan_interval_days,
        });
      } catch (error) {
        log.error('Failed to load auto memory settings', error);
        if (!cancelled) {
          showMessage('error', t('messages.loadFailed'));
        }
      } finally {
        if (!cancelled) {
          setIsLoading(false);
        }
      }
    };

    void loadConfig();

    return () => {
      cancelled = true;
    };
  }, [showMessage, t]);

  useEffect(() => () => {
    if (messageTimeoutRef.current !== null) {
      window.clearTimeout(messageTimeoutRef.current);
    }
  }, []);

  const updateScopeState = (
    scope: AutoMemoryScopeKey,
    nextState: Partial<AutoMemoryScopeState>
  ) => {
    setAutoMemoryState((previousState) => ({
      ...previousState,
      [scope]: {
        ...previousState[scope],
        ...nextState,
      },
    }));
  };

  const saveEnabled = async (scope: AutoMemoryScopeKey, nextValue: boolean) => {
    const previousValue = autoMemoryState[scope].enabled;
    updateScopeState(scope, { enabled: nextValue });
    setIsSaving(true);
    try {
      await configManager.setConfig(AUTO_MEMORY_CONFIG_PATHS[scope].enabled, nextValue);
      showMessage('success', t('messages.saveSuccess'));
    } catch (error) {
      log.error('Failed to save auto memory enabled setting', {
        scope,
        error,
      });
      updateScopeState(scope, { enabled: previousValue });
      showMessage('error', t('messages.saveFailed'));
    } finally {
      setIsSaving(false);
    }
  };

  const saveThreshold = async (scope: AutoMemoryScopeKey, nextValue: number) => {
    const normalizedValue = normalizeExtractEveryEligibleTurns(nextValue);
    const previousValue = autoMemoryState[scope].extractEveryEligibleTurns;

    if (normalizedValue === previousValue) {
      return;
    }

    updateScopeState(scope, { extractEveryEligibleTurns: normalizedValue });
    setIsSaving(true);
    try {
      await configManager.setConfig(
        AUTO_MEMORY_CONFIG_PATHS[scope].extractEveryEligibleTurns,
        normalizedValue
      );
      showMessage('success', t('messages.saveSuccess'));
    } catch (error) {
      log.error('Failed to save auto memory threshold setting', {
        scope,
        error,
      });
      updateScopeState(scope, { extractEveryEligibleTurns: previousValue });
      showMessage('error', t('messages.saveFailed'));
    } finally {
      setIsSaving(false);
    }
  };

  const persistHostScanConfig = useCallback(
    async (nextConfig: AppHostScanConfig, successMessage: string) => {
      setIsSaving(true);
      try {
        await configManager.setConfig('app.host_scan', nextConfig);
        configManager.clearCache();
        setHostScanConfig(nextConfig);
        showMessage('success', successMessage);
      } catch (error) {
        log.error('Failed to save host scan config', { nextConfig, error });
        showMessage('error', t('hostScan.messages.saveFailed'));
      } finally {
        setIsSaving(false);
      }
    },
    [showMessage, t]
  );

  const handleHostScanEnabledChange = useCallback(
    async (checked: boolean) => {
      await persistHostScanConfig(
        { ...hostScanConfig, auto_scan_enabled: checked },
        t('hostScan.messages.enabledUpdated')
      );
    },
    [hostScanConfig, persistHostScanConfig, t]
  );

  const handleHostScanIntervalChange = useCallback(
    async (value: string) => {
      const nextInterval = Number.parseInt(value, 10);
      if (!Number.isFinite(nextInterval) || nextInterval <= 0) {
        return;
      }

      await persistHostScanConfig(
        { ...hostScanConfig, auto_scan_interval_days: nextInterval },
        t('hostScan.messages.intervalUpdated')
      );
    },
    [hostScanConfig, persistHostScanConfig, t]
  );

  if (isLoading) {
    return (
      <ConfigPageLayout className="bitfun-func-agent-config">
        <ConfigPageHeader title={t('title')} subtitle={t('subtitle')} />
        <ConfigPageContent className="bitfun-func-agent-config__content">
          <ConfigPageLoading text={t('loading.text')} />
        </ConfigPageContent>
      </ConfigPageLayout>
    );
  }

  const renderScopeSection = (scope: AutoMemoryScopeKey) => (
    <ConfigPageSection
      key={scope}
      title={t(`autoMemory.${scope}.sectionTitle`)}
      description={t(`autoMemory.${scope}.sectionDescription`)}
    >
      <ConfigPageRow
        label={t(`autoMemory.${scope}.enabled`)}
        description={t(`autoMemory.${scope}.enabledDesc`)}
        align="center"
      >
        <div className="bitfun-func-agent-config__row-control">
          <Switch
            checked={autoMemoryState[scope].enabled}
            onChange={(event) => void saveEnabled(scope, event.target.checked)}
            disabled={isSaving}
            size="small"
          />
        </div>
      </ConfigPageRow>
      <ConfigPageRow
        label={t(`autoMemory.${scope}.extractEveryEligibleTurns`)}
        description={t(`autoMemory.${scope}.extractEveryEligibleTurnsDesc`)}
        align="center"
      >
        <div className="bitfun-func-agent-config__row-control">
          <NumberInput
            value={autoMemoryState[scope].extractEveryEligibleTurns}
            onChange={(value) => void saveThreshold(scope, value)}
            min={1}
            max={100}
            step={1}
            disabled={isSaving}
            size="small"
            variant="compact"
          />
        </div>
      </ConfigPageRow>
    </ConfigPageSection>
  );

  return (
    <ConfigPageLayout className="bitfun-func-agent-config">
      <ConfigPageHeader title={t('title')} subtitle={t('subtitle')} />
      <ConfigPageContent className="bitfun-func-agent-config__content">
        <ConfigPageMessage message={message} />
        {AUTO_MEMORY_SCOPES.map(renderScopeSection)}
        <ConfigPageSection
          title={t('hostScan.title')}
          description={t('hostScan.hint')}
        >
          <ConfigPageRow
            label={t('hostScan.enable.label')}
            align="center"
          >
            <div className="bitfun-func-agent-config__row-control">
              <Switch
                checked={hostScanConfig.auto_scan_enabled}
                onChange={(event) => void handleHostScanEnabledChange(event.target.checked)}
                disabled={isSaving}
                size="small"
              />
            </div>
          </ConfigPageRow>
          <ConfigPageRow
            label={t('hostScan.interval.label')}
            description={t('hostScan.interval.description')}
            align="center"
          >
            <div className="bitfun-func-agent-config__row-control">
              <Select
                value={String(hostScanConfig.auto_scan_interval_days)}
                onChange={(value) => void handleHostScanIntervalChange(value as string)}
                options={hostScanIntervalOptions}
                disabled={isSaving || !hostScanConfig.auto_scan_enabled}
              />
            </div>
          </ConfigPageRow>
        </ConfigPageSection>
      </ConfigPageContent>
    </ConfigPageLayout>
  );
};

export default MemoryConfig;
