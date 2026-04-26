import React, { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { ConfigPageLoading, NumberInput, Switch } from '@/component-library';
import { notificationService } from '@/shared/notification-system';
import { createLogger } from '@/shared/utils/logger';
import { configManager } from '../services/ConfigManager';
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

const MemoryConfig: React.FC = () => {
  const { t } = useTranslation('settings/memory');
  const [isLoading, setIsLoading] = useState(true);
  const [autoMemoryState, setAutoMemoryState] = useState<AutoMemoryState>(
    DEFAULT_AUTO_MEMORY_STATE
  );
  const [isSaving, setIsSaving] = useState(false);

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
        ] = await Promise.all([
          configManager.getConfig<boolean>(AUTO_MEMORY_CONFIG_PATHS.global.enabled),
          configManager.getConfig<number>(AUTO_MEMORY_CONFIG_PATHS.global.extractEveryEligibleTurns),
          configManager.getConfig<boolean>(AUTO_MEMORY_CONFIG_PATHS.workspace.enabled),
          configManager.getConfig<number>(
            AUTO_MEMORY_CONFIG_PATHS.workspace.extractEveryEligibleTurns
          ),
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
      } catch (error) {
        log.error('Failed to load auto memory settings', error);
        if (!cancelled) {
          notificationService.error(t('messages.saveFailed'));
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
  }, [t]);

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
      notificationService.success(t('messages.saveSuccess'), { duration: 2000 });
    } catch (error) {
      log.error('Failed to save auto memory enabled setting', {
        scope,
        error,
      });
      updateScopeState(scope, { enabled: previousValue });
      notificationService.error(t('messages.saveFailed'));
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
      notificationService.success(t('messages.saveSuccess'), { duration: 2000 });
    } catch (error) {
      log.error('Failed to save auto memory threshold setting', {
        scope,
        error,
      });
      updateScopeState(scope, { extractEveryEligibleTurns: previousValue });
      notificationService.error(t('messages.saveFailed'));
    } finally {
      setIsSaving(false);
    }
  };

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
        {AUTO_MEMORY_SCOPES.map(renderScopeSection)}
      </ConfigPageContent>
    </ConfigPageLayout>
  );
};

export default MemoryConfig;
