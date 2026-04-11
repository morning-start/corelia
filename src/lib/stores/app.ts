import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { DEFAULT_APP_CONFIG } from '$lib/config';
import type { AppConfig } from '$lib/config';

function createAppStore() {
  const { subscribe, set } = writable<AppConfig>(DEFAULT_APP_CONFIG);

  return {
    subscribe,

    async load(): Promise<void> {
      try {
        const config = await invoke<AppConfig>('load_app_config');
        set({ ...config });
      } catch (error) {
        console.error('Failed to load app config:', error);
        set({ ...DEFAULT_APP_CONFIG });
      }
    },

    async save(config: AppConfig): Promise<void> {
      try {
        await invoke('save_app_config', { config: { ...config } });
      } catch (error) {
        console.error('Failed to save app config:', error);
        throw error;
      }
    },

    async clear(): Promise<void> {
      set({ ...DEFAULT_APP_CONFIG });
      try {
        await invoke('clear_app_config');
      } catch (error) {
        console.error('Failed to clear app config:', error);
        throw error;
      }
    },

    async addHistory(query: string): Promise<void> {
      const currentConfig = get({ subscribe });

      const existing = currentConfig.searchHistory.find(h => h.query === query);
      if (existing) {
        existing.count++;
        existing.timestamp = Date.now();
      } else {
        currentConfig.searchHistory.unshift({
          query,
          timestamp: Date.now(),
          count: 1
        });
        if (currentConfig.searchHistory.length > 100) {
          currentConfig.searchHistory = currentConfig.searchHistory.slice(0, 100);
        }
      }

      set(currentConfig);
      await this.save(currentConfig);
    },

    async updatePluginCache(pluginId: string, data: {
      version: string;
      loadTime: number;
    }): Promise<void> {
      const currentConfig = get({ subscribe });

      currentConfig.plugins.cache[pluginId] = {
        ...data,
        lastUsed: Date.now()
      };

      set(currentConfig);
      await this.save(currentConfig);
    }
  };
}

export const app = createAppStore();
