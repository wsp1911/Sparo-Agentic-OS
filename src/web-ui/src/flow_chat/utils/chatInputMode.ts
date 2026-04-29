export function resolveWorkspaceChatInputMode(params: {
  currentMode: string;
  isAssistantWorkspace: boolean;
  sessionMode?: string | null;
}): string | null {
  const normalizedSessionMode = params.sessionMode?.trim();

  if (normalizedSessionMode && normalizedSessionMode !== params.currentMode) {
    return normalizedSessionMode;
  }

  return null;
}
