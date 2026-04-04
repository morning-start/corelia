import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { deepClone, safeGet } from '$lib/utils/helpers';

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

const DEFAULT_SETTINGS: Settings = {
  theme: 'dark',
  shortcut: {
    summon: 'Alt+Space'
  },
  behavior: {
    autoHide: true,
    autoHideDelay: 3000
  },
  startup: {
    enabled: false,
    minimizeToTray: true
  }
};

function createSettingsStore() {
  const { subscribe, set, update } = writable<Settings>(DEFAULT_SETTINGS);

  return {
    subscribe,

    /**
     * 从后端加载设置
     */
    async load(): Promise<void> {
      try {
        const loaded = await invoke<Settings>('load_settings');
        // 使用深拷贝确保响应式更新
        set(deepClone(loaded));
      } catch (error) {
        console.error('Failed to load settings:', error);
        // 使用默认设置
        set(deepClone(DEFAULT_SETTINGS));
      }
    },

    /**
     * 保存设置到后端
     */
    async save(settings: Settings): Promise<void> {
      try {
        // 使用深拷贝避免引用问题
        const cloned = deepClone(settings);
        await invoke('save_settings', { settings: cloned });
      } catch (error) {
        console.error('Failed to save settings:', error);
        throw error;
      }
    },

    /**
     * 重置为默认设置
     */
    async reset(): Promise<void> {
      set(deepClone(DEFAULT_SETTINGS));
      await this.save(DEFAULT_SETTINGS);
    },

    /**
     * 获取特定设置值
     */
    get<T>(path: string, defaultValue?: T): T | undefined {
      let value: Settings | undefined;
      subscribe(v => { value = v; })();
      // 安全地访问嵌套属性
      const parts = path.split('.');
      let result: unknown = value;
      for (const part of parts) {
        if (result && typeof result === 'object' && part in result) {
          result = (result as Record<string, unknown>)[part];
        } else {
          return defaultValue;
        }
      }
      return result as T;
    }
  };
}

export const settings = createSettingsStore();
