/**
 * 应用全局常量配置
 *
 * 统一管理应用级别的全局常量，确保前后端配置一致性
 */

export const WINDOW_CONFIG = {
  WIDTH: 600,
  HEIGHT: 420,
  MIN_WIDTH: 200,
  MIN_HEIGHT: 300,
  MAX_WIDTH: 1200,
  MAX_HEIGHT: 900,
} as const;

export const SEARCH_CONFIG = {
  /** 搜索结果防抖延迟 (ms) */
  DEBOUNCE_DELAY: 150,
  /** 历史记录保存防抖延迟 (ms) — 用户停止输入后多久保存 */
  HISTORY_SAVE_DELAY: 300,
  /** UI 显示的历史条数 */
  DISPLAYED_HISTORY_COUNT: 5,
} as const;

export const PERFORMANCE_CONFIG = {
  DELAY_UNPIN_MS: 100,
} as const;

export const DEFAULT_USER_CONFIG: UserConfig = {
  theme: 'dark',
  behavior: { autoHide: true, autoHideDelay: 3000 },
  window: { width: 600, height: 420 },
  search: { defaultCategory: 'all', maxResults: 20, maxHistoryCapacity: 100 },
};

export const DEFAULT_APP_CONFIG: AppConfig = {
  searchHistory: [],
  plugins: { cache: {}, enabled: [] },
  runtime: { lastState: {}, usageStats: { launchCount: 0, totalUsageTime: 0 } },
};

export const DEFAULT_SYSTEM_CONFIG: SystemConfig = {
  shortcut: { summon: 'Alt+Space' },
  startup: { enabled: false, minimizeToTray: true },
  advanced: { debugMode: false },
};

export interface UserConfig {
  theme: 'dark' | 'light' | 'system';
  behavior: { autoHide: boolean; autoHideDelay: number };
  window: { width: number; height: number };
  search: { defaultCategory: 'all' | 'plugins' | 'system'; maxResults: number; maxHistoryCapacity?: number };
}

export interface AppConfig {
  searchHistory: Array<{ query: string; timestamp: number; count: number }>;
  plugins: {
    cache: Record<string, { version: string; lastUsed: number; loadTime: number }>;
    enabled: string[];
  };
  runtime: {
    lastState: { lastQuery?: string; selectedPlugin?: string };
    usageStats: { launchCount: number; totalUsageTime: number };
  };
}

export interface SystemConfig {
  shortcut: { summon: string };
  startup: { enabled: boolean; minimizeToTray: boolean };
  advanced: { debugMode: boolean };
}
