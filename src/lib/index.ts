/**
 * Corelia 前端库统一导出
 */

export { api } from './api';

export { startupService } from './services/startup';
export type { StartupService } from './services/startup';

export { searchHistory } from './stores/history';
export type { HistoryItem } from './stores/history';

export { system } from './stores/system';
export type { SystemConfig } from './stores/system';

export { user } from './stores/user';

export { theme } from './stores/theme';
export type { Theme } from './stores/theme';

export type { UserConfig, AppConfig, SystemConfig as SystemConfigType } from './config';

export type { SearchItem } from './search/fuzzy';

export * from './utils/errors';
export * from './utils/helpers';
