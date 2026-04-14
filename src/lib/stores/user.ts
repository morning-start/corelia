import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { DEFAULT_USER_CONFIG } from '$lib/config';
import type { UserConfig } from '$lib/config';

function createUserStore() {
  const { subscribe, set } = writable<UserConfig>(DEFAULT_USER_CONFIG);

  return {
    subscribe,

    async load(): Promise<void> {
      try {
        const config = await invoke<UserConfig>('load_user_config');
        set({ ...config });
      } catch (error) {
        console.error('Failed to load user config:', error);
        set({ ...DEFAULT_USER_CONFIG });
      }
    },

    async save(config: UserConfig): Promise<void> {
      try {
        await invoke('save_user_config', { config: { ...config } });
        set({ ...config });
      } catch (error) {
        console.error('Failed to save user config:', error);
        throw error;
      }
    },

    async reset(): Promise<void> {
      set({ ...DEFAULT_USER_CONFIG });
      try {
        await invoke('reset_user_config');
      } catch (error) {
        console.error('Failed to reset user config:', error);
        throw error;
      }
    },

    get<K extends keyof UserConfig>(key: K): UserConfig[K] {
      const state = get({ subscribe });
      return state[key];
    },

    async update(path: string, value: unknown): Promise<void> {
      const current = get({ subscribe });
      const newConfig = setNestedValue({ ...current }, path, value);
      await this.save(newConfig);
    }
  };
}

function setNestedValue<T extends Record<string, unknown>>(obj: T, path: string, value: unknown): T {
  const keys = path.split('.');
  // 从根对象浅拷贝开始，逐层深入时也拷贝每层，确保不可变更新
  let root: Record<string, unknown> = { ...obj };
  let current = root;

  for (let i = 0; i < keys.length - 1; i++) {
    const key = keys[i];
    const next = current[key];
    if (!next || typeof next !== 'object' || Array.isArray(next)) {
      current[key] = {};
    } else {
      current[key] = { ...(next as Record<string, unknown>) };
    }
    current = current[key] as Record<string, unknown>;
  }

  current[keys.at(-1)!] = value;
  return root as T;
}

export const user = createUserStore();
