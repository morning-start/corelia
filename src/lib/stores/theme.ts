import { writable } from 'svelte/store';

/**
 * 主题类型
 * - dark: 深色主题
 * - light: 浅色主题
 * - system: 跟随系统设置
 */
export type Theme = 'dark' | 'light' | 'system';

/** 媒体查询监听器引用（用于清理） */
let mediaQuery: MediaQueryList | null = null;
let mediaListener: ((e: MediaQueryListEvent) => void) | null = null;

/**
 * 根据系统偏好获取实际主题
 */
function getSystemTheme(): 'dark' | 'light' {
  if (typeof window === 'undefined') return 'dark';
  return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
}

/**
 * 应用主题到 DOM
 */
function applyTheme(theme: Theme) {
  if (typeof document === 'undefined') return;
  const actualTheme = theme === 'system' ? getSystemTheme() : theme;
  document.documentElement.setAttribute('data-theme', actualTheme);
}

/**
 * 创建主题状态管理
 * 管理应用的主题设置，包括深色/浅色/跟随系统三种模式
 */
function createThemeStore() {
  const { subscribe, set, update } = writable<Theme>('dark');

  return {
    subscribe,

    /**
     * 设置主题
     * @param theme - 要设置的主题类型
     */
    set: (theme: Theme) => {
      // 清理旧的监听器
      cleanupMediaQuery();

      applyTheme(theme);

      // 如果是 system 模式，监听系统主题变化
      if (theme === 'system') {
        setupMediaQuery();
      }

      set(theme);
    },

    /**
     * 切换主题（深色 <-> 浅色）
     */
    toggle: () => {
      update(current => {
        const next = current === 'dark' ? 'light' : 'dark';
        cleanupMediaQuery();
        applyTheme(next);
        set(next);
        return next;
      });
    }
  };
}

/**
 * 设置 system 模式的媒体查询监听
 */
function setupMediaQuery() {
  if (typeof window === 'undefined') return;

  mediaQuery = window.matchMedia('(prefers-color-scheme: light)');
  mediaListener = (e: MediaQueryListEvent) => {
    // 仅在 system 模式下响应系统变化
    applyTheme('system');
  };

  mediaQuery.addEventListener('change', mediaListener);
}

/**
 * 清理媒体查询监听
 */
function cleanupMediaQuery() {
  if (mediaQuery && mediaListener) {
    mediaQuery.removeEventListener('change', mediaListener);
    mediaQuery = null;
    mediaListener = null;
  }
}

/** 主题状态管理实例 */
export const theme = createThemeStore();
