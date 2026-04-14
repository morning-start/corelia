import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { deepClone } from '$lib/utils/helpers';
import type { SystemConfig } from '$lib/config';

const DEFAULT_SYSTEM_CONFIG: SystemConfig = {
  shortcut: {
    summon: 'Alt+Space'
  },
  startup: {
    enabled: false,
    minimizeToTray: true
  },
  advanced: {
    debugMode: false
  }
};

function createSystemStore() {
  const { subscribe, set } = writable<SystemConfig>(DEFAULT_SYSTEM_CONFIG);

  return {
    subscribe,

    /**
     * 从后端加载系统配置
     */
    async load(): Promise<void> {
      try {
        const config = await invoke<SystemConfig>('load_system_config');
        set(deepClone(config));
      } catch (error) {
        console.error('Failed to load system config:', error);
        set(deepClone(DEFAULT_SYSTEM_CONFIG));
      }
    },

    /**
     * 保存系统配置到后端
     */
    async save(config: SystemConfig): Promise<void> {
      try {
        const cloned = deepClone(config);
        await invoke('save_system_config', { config: cloned });
      } catch (error) {
        console.error('Failed to save system config:', error);
        throw error;
      }
    }
  };
}

export const system = createSystemStore();
