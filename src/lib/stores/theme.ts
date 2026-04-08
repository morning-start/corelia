import { writable } from 'svelte/store';

/**
 * 主题类型
 * - dark: 深色主题
 * - light: 浅色主题
 * - system: 跟随系统设置
 */
export type Theme = 'dark' | 'light' | 'system';

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
      if (typeof document !== 'undefined') {
        document.documentElement.setAttribute('data-theme', theme);
      }
      set(theme);
    },

    /**
     * 切换主题（深色 <-> 浅色）
     */
    toggle: () => {
      update(current => {
        const next = current === 'dark' ? 'light' : 'dark';
        if (typeof document !== 'undefined') {
          document.documentElement.setAttribute('data-theme', next);
        }
        return next;
      });
    }
  };
}

/** 主题状态管理实例 */
export const theme = createThemeStore();
