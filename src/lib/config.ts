/**
 * 应用全局常量配置
 * 
 * 统一管理应用级别的全局常量，确保配置一致性
 */

/**
 * 窗口配置
 */
export const WINDOW_CONFIG = {
  /** 窗口宽度（逻辑像素，受 DPI 缩放影响） */
  WIDTH: 600,
  /** 窗口高度（逻辑像素，受 DPI 缩放影响）- 适配搜索框 + 分类 + 5 条历史记录 */
  HEIGHT: 420,
  /** 最小宽度 */
  MIN_WIDTH: 200,
  /** 最小高度 */
  MIN_HEIGHT: 300,
  /** 最大宽度 */
  MAX_WIDTH: 1200,
  /** 最大高度 */
  MAX_HEIGHT: 900,
} as const;

/**
 * 搜索配置
 */
export const SEARCH_CONFIG = {
  /** 搜索历史最大条目数 */
  MAX_HISTORY_ITEMS: 5,
  /** 搜索防抖延迟（毫秒） */
  DEBOUNCE_DELAY: 150,
} as const;

/**
 * 性能配置
 */
export const PERFORMANCE_CONFIG = {
  /** 延迟取消置顶时间（毫秒） */
  DELAY_UNPIN_MS: 100,
} as const;
