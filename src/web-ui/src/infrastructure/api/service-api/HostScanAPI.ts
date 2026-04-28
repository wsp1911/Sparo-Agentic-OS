import { api } from './ApiClient';
import { createTauriCommandError } from '../errors/TauriCommandError';

export interface StartHostScanRequest {
  requestId: string;
  parentSessionId: string;
  childSessionId: string;
  childSessionName?: string;
  modelId?: string;
}

export interface StartHostScanResponse {
  ok: boolean;
}

export class HostScanAPI {
  async startStream(request: StartHostScanRequest): Promise<StartHostScanResponse> {
    try {
      return await api.invoke<StartHostScanResponse>('start_host_scan_stream', { request });
    } catch (error) {
      throw createTauriCommandError('start_host_scan_stream', error, request);
    }
  }
}

export const hostScanAPI = new HostScanAPI();
