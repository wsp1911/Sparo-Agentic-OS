import { create } from 'zustand';

export interface DesignTokenProposal {
  id: string;
  name: string;
  mood: string;
  colors: Record<string, string>;
  typography: Record<string, unknown>;
  radius: Record<string, unknown>;
  shadow: Record<string, unknown>;
  motion: Record<string, unknown>;
  spacing?: Record<string, unknown>;
  component_samples: Record<string, unknown>;
  created_at?: string;
}

export interface DesignTokensDocument {
  version: number;
  proposals: DesignTokenProposal[];
  committed_id?: string | null;
  committed_at?: string | null;
  scope?: string | null;
}

interface DesignTokensState {
  byScope: Record<string, DesignTokensDocument>;
  lastProposedAt?: number;
  upsert: (scopeKey: string, document: DesignTokensDocument) => void;
  clear: (scopeKey: string) => void;
}

export const useDesignTokensStore = create<DesignTokensState>((set) => ({
  byScope: {},
  lastProposedAt: undefined,
  upsert: (scopeKey, document) =>
    set((state) => ({
      byScope: { ...state.byScope, [scopeKey]: document },
      lastProposedAt: Date.now(),
    })),
  clear: (scopeKey) =>
    set((state) => {
      const { [scopeKey]: _removed, ...rest } = state.byScope;
      return { byScope: rest };
    }),
}));

