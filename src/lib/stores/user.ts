import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { deepClone } from '$lib/utils/helpers';

export interface UserConfig {
  theme: 'dark' | 'light' | 'system';
  behavior: {
    autoHide: boolean;
    autoHideDelay: number;
  };
  window: {
    width: number;
    height: number;
  };
  search: {
    defaultCategory: 'all' | 'plugins' | 'system';
    maxResults: number;
  };
}

const DEFAULT_USER_CONFIG: UserConfig = {
  theme: 'dark',
  behavior: {
    autoHide: true,
    autoHideDelay: 3000
  },
  window: {
    width: 600,
    height: 400
  },
  search: {
    defaultCategory: 'all',
    maxResults: 20
  }
};

function createUserStore() {
  const { subscribe, set } = writable<UserConfig>(DEFAULT_USER_CONFIG);

  return {
    subscribe,

    /**
     * 从后端加载用户配置
     */
    async load(): Promise<void> {
      try {
        const config = await invoke<UserConfig>('load_user_config');
        set(deepClone(config));
      } catch (error) {
        console.error('Failed to load user config:', error);
        set(deepClone(DEFAULT_USER_CONFIG));
      }
    },

    /**
     * 保存用户配置到后端
     */
    async save(config: UserConfig): Promise<void> {
      try {
        const cloned = deepClone(config);
        await invoke('save_user_config', { config: cloned });
      } catch (error) {
        console.error('Failed to save user config:', error);
        throw error;
      }
    },

    /**
     * 重置用户配置为默认值
     */
    async reset(): Promise<void> {
      set(deepClone(DEFAULT_USER_CONFIG));
      try {
        await invoke('reset_user_config');
      } catch (error) {
        console.error('Failed to reset user config:', error);
        throw error;
      }
    },

    /**
     * 更新单个配置项
     */
    async update(path: string, value: unknown): Promise<void> {
      let config: UserConfig;
      subscribe(c => { config = c; })();
      
      // 深拷贝并更新
      const newConfig = deepClone(config);
      const keys = path.split('.');
      let current: any = newConfig;
      for (let i = 0; i < keys.length - 1; i++) {
        current = current[keys[i]];
      }
      current[keys.at(-1)!] = value;
      
      set(newConfig);
      await this.save(newConfig);
    }
  };
}

export const user = createUserStore();
