import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { deepClone } from '$lib/utils/helpers';

export interface AppConfig {
  searchHistory: Array<{
    query: string;
    timestamp: number;
    count: number;
  }>;
  plugins: {
    cache: Record<string, {
      version: string;
      lastUsed: number;
      loadTime: number;
    }>;
    enabled: string[];
  };
  runtime: {
    lastState: {
      lastQuery?: string;
      selectedPlugin?: string;
    };
    usageStats: {
      launchCount: number;
      totalUsageTime: number;
    };
  };
}

const DEFAULT_APP_CONFIG: AppConfig = {
  searchHistory: [],
  plugins: {
    cache: {},
    enabled: []
  },
  runtime: {
    lastState: {},
    usageStats: {
      launchCount: 0,
      totalUsageTime: 0
    }
  }
};

function createAppStore() {
  const { subscribe, set } = writable<AppConfig>(DEFAULT_APP_CONFIG);

  return {
    subscribe,

    /**
     * 从后端加载应用配置
     */
    async load(): Promise<void> {
      try {
        const config = await invoke<AppConfig>('load_app_config');
        set(deepClone(config));
      } catch (error) {
        console.error('Failed to load app config:', error);
        set(deepClone(DEFAULT_APP_CONFIG));
      }
    },

    /**
     * 保存应用配置到后端
     */
    async save(config: AppConfig): Promise<void> {
      try {
        const cloned = deepClone(config);
        await invoke('save_app_config', { config: cloned });
      } catch (error) {
        console.error('Failed to save app config:', error);
        throw error;
      }
    },

    /**
     * 清理应用配置
     */
    async clear(): Promise<void> {
      set(deepClone(DEFAULT_APP_CONFIG));
      try {
        await invoke('clear_app_config');
      } catch (error) {
        console.error('Failed to clear app config:', error);
        throw error;
      }
    },

    /**
     * 添加搜索历史
     */
    async addHistory(query: string): Promise<void> {
      let config: AppConfig;
      subscribe(c => { config = c; })();
      
      // 查找是否已存在
      const existing = config.searchHistory.find(h => h.query === query);
      if (existing) {
        existing.count++;
        existing.timestamp = Date.now();
      } else {
        config.searchHistory.unshift({
          query,
          timestamp: Date.now(),
          count: 1
        });
        // 限制历史记录数量
        if (config.searchHistory.length > 100) {
          config.searchHistory = config.searchHistory.slice(0, 100);
        }
      }
      
      set(config);
      await this.save(config);
    },

    /**
     * 更新插件缓存
     */
    async updatePluginCache(pluginId: string, data: {
      version: string;
      loadTime: number;
    }): Promise<void> {
      let config: AppConfig;
      subscribe(c => { config = c; })();
      
      config.plugins.cache[pluginId] = {
        ...data,
        lastUsed: Date.now()
      };
      
      set(config);
      await this.save(config);
    }
  };
}

export const app = createAppStore();
