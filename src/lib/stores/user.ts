import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

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

/**
 * 创建用户配置状态管理
 * 管理用户级配置的加载、保存和更新
 */
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
        set({ ...config });
      } catch (error) {
        console.error('Failed to load user config:', error);
        set({ ...DEFAULT_USER_CONFIG });
      }
    },

    /**
     * 保存用户配置到后端
     */
    async save(config: UserConfig): Promise<void> {
      try {
        await invoke('save_user_config', { config: { ...config } });
        set({ ...config });
      } catch (error) {
        console.error('Failed to save user config:', error);
        throw error;
      }
    },

    /**
     * 重置用户配置为默认值
     */
    async reset(): Promise<void> {
      set({ ...DEFAULT_USER_CONFIG });
      try {
        await invoke('reset_user_config');
      } catch (error) {
        console.error('Failed to reset user config:', error);
        throw error;
      }
    },

    /**
     * 获取特定配置项
     */
    get<K extends keyof UserConfig>(key: K): UserConfig[K] {
      const state = get({ subscribe });
      return state[key];
    },

    /**
     * 更新指定路径的配置值
     * @param path - 点分隔的路径，如 'behavior.autoHide'
     * @param value - 要设置的值
     */
    async update(path: string, value: unknown): Promise<void> {
      const current = get({ subscribe });
      const newConfig = setNestedValue({ ...current }, path, value);
      await this.save(newConfig);
    }
  };
}

/**
 * 安全地设置嵌套对象属性
 */
function setNestedValue<T extends object>(
  obj: T,
  path: string,
  value: unknown
): T {
  const keys = path.split('.');
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let current: any = { ...obj };

  for (let i = 0; i < keys.length - 1; i++) {
    const key = keys[i];
    if (!(key in current) || typeof current[key] !== 'object') {
      current[key] = {};
    }
    current[key] = { ...current[key] };
    current = current[key];
  }

  current[keys.at(-1)!] = value;
  return obj;
}

/** 用户配置状态管理单例 */
export const user = createUserStore();
