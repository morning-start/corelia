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

function setNestedValue<T extends object>(obj: T, path: string, value: unknown): T {
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

export const user = createUserStore();
