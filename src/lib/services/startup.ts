import { invoke } from '@tauri-apps/api/core';

export interface StartupService {
  enable(): Promise<void>;
  disable(): Promise<void>;
  isEnabled(): Promise<boolean>;
}

class TauriStartupService implements StartupService {
  async enable(): Promise<void> {
    await invoke('enable_autostart');
  }

  async disable(): Promise<void> {
    await invoke('disable_autostart');
  }

  async isEnabled(): Promise<boolean> {
    return await invoke<boolean>('is_autostart_enabled');
  }
}

export const startupService = new TauriStartupService();