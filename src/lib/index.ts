/**
 * Corelia 前端库统一导出
 * 
 * 使用示例：
 * ```typescript
 * import { storeService, searchHistory, settings } from '$lib';
 * ```
 */

// Services - 服务层
export { storeService } from './services/store';
export type { StoreService } from './services/store';

export { startupService } from './services/startup';
export type { StartupService } from './services/startup';

// Stores - 状态管理
export { searchHistory } from './stores/history';
export type { HistoryItem } from './stores/history';

export { settings } from './stores/settings';
export type { Settings } from './stores/settings';

export { theme } from './stores/theme';
export type { Theme } from './stores/theme';

// Search - 搜索相关
export type { SearchItem } from './search/fuzzy';

// Utils - 工具函数
export * from './utils/errors';
export * from './utils/helpers';
