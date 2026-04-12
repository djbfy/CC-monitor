// useLaunchSession.ts — Phase 4: real Tauri invoke
import { invoke } from '@tauri-apps/api/core';
import { CcResult } from '../types';

export function useLaunchSession() {
  return async (workDir: string): Promise<CcResult<string>> => {
    try {
      const id = await invoke<string>('launch_session', { workDir });
      return { ok: true, data: id };
    } catch (e: unknown) {
      return { ok: false, error: String(e) };
    }
  };
}
