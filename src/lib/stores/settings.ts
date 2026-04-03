import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface Settings {
  theme: 'dark' | 'light' | 'system';
  shortcut: {
    summon: string;
  };
  behavior: {
    autoHide: boolean;
    autoHideDelay: number;
  };
  startup: {
    enabled: boolean;
    minimizeToTray: boolean;
  };
}

const defaultSettings: Settings = {
  theme: 'dark',
  shortcut: {
    summon: 'Alt+Space',
  },
  behavior: {
    autoHide: true,
    autoHideDelay: 3000,
  },
  startup: {
    enabled: false,
    minimizeToTray: true,
  },
};

function createSettingsStore() {
  const { subscribe, set, update } = writable<Settings>(defaultSettings);

  return {
    subscribe,
    set,
    update,
    async load() {
      try {
        const settings = await invoke<Settings>('load_settings');
        set(settings);
      } catch (e) {
        console.error('Failed to load settings:', e);
        set(defaultSettings);
      }
    },
    async save(settings: Settings) {
      try {
        await invoke('save_settings', { settings });
        set(settings);
      } catch (e) {
        console.error('Failed to save settings:', e);
      }
    },
    reset() {
      set(defaultSettings);
    },
  };
}

export const settings = createSettingsStore();
