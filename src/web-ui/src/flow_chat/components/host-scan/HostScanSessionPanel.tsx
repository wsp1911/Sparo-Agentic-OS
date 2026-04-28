import React from 'react';
import { ChildSessionPanel, type ChildSessionPanelProps } from '../child-session/ChildSessionPanel';

export type HostScanSessionPanelProps = Omit<ChildSessionPanelProps, 'variant'>;

export const HostScanSessionPanel: React.FC<HostScanSessionPanelProps> = props => (
  <ChildSessionPanel {...props} variant="host_scan" />
);

HostScanSessionPanel.displayName = 'HostScanSessionPanel';
