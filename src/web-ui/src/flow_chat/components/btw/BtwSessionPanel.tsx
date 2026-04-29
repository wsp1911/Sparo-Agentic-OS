import React from 'react';
import { ChildSessionPanel, type ChildSessionPanelProps } from '../child-session/ChildSessionPanel';

export type BtwSessionPanelProps = Omit<ChildSessionPanelProps, 'variant'>;

export const BtwSessionPanel: React.FC<BtwSessionPanelProps> = props => (
  <ChildSessionPanel {...props} variant="btw" />
);

BtwSessionPanel.displayName = 'BtwSessionPanel';
