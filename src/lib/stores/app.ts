import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

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
        set({ ...config });
      } catch (error) {
        console.error('Failed to load app config:', error);
        set({ ...DEFAULT_APP_CONFIG });
      }
    },

    /**
     * 保存应用配置到后端
     */
    async save(config: AppConfig): Promise<void> {
      try {
        await invoke('save_app_config', { config: { ...config } });
      } catch (error) {
        console.error('Failed to save app config:', error);
        throw error;
      }
    },

    /**
     * 清理应用配置
     */
    async clear(): Promise<void> {
      set({ ...DEFAULT_APP_CONFIG });
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
      const currentConfig = get({ subscribe });

      // 查找是否已存在
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
        // 限制历史记录数量
        if (currentConfig.searchHistory.length > 100) {
          currentConfig.searchHistory = currentConfig.searchHistory.slice(0, 100);
        }
      }

      set(currentConfig);
      await this.save(currentConfig);
    },

    /**
     * 更新插件缓存
     */
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
