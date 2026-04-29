import { describe, expect, it } from 'vitest';

import { resolveWorkspaceChatInputMode } from './chatInputMode';

describe('resolveWorkspaceChatInputMode', () => {
  it('keeps unchanged modes as-is', () => {
    expect(
      resolveWorkspaceChatInputMode({
        currentMode: 'Plan',
        isAssistantWorkspace: false,
        sessionMode: 'Plan',
      })
    ).toBeNull();
  });

  it('syncs when switching between project sessions with different modes', () => {
    expect(
      resolveWorkspaceChatInputMode({
        currentMode: 'Plan',
        isAssistantWorkspace: false,
        sessionMode: 'agentic',
      })
    ).toBe('agentic');
  });

  it('restores agentic when the current mode is stale', () => {
    expect(
      resolveWorkspaceChatInputMode({
        currentMode: 'Plan',
        isAssistantWorkspace: false,
        sessionMode: 'agentic',
      })
    ).toBe('agentic');
  });

  it('restores Cowork when the current mode is stale', () => {
    expect(
      resolveWorkspaceChatInputMode({
        currentMode: 'agentic',
        isAssistantWorkspace: false,
        sessionMode: 'Cowork',
      })
    ).toBe('Cowork');
  });

  it('falls back to agentic if a project session has no mode yet', () => {
    expect(
      resolveWorkspaceChatInputMode({
        currentMode: 'Plan',
        isAssistantWorkspace: false,
        sessionMode: undefined,
      })
    ).toBeNull();
  });
});
