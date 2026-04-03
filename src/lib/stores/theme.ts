import { writable } from 'svelte/store';

export type Theme = 'dark' | 'light' | 'system';

function createThemeStore() {
  const { subscribe, set, update } = writable<Theme>('dark');

  return {
    subscribe,
    set: (theme: Theme) => {
      if (typeof document !== 'undefined') {
        document.documentElement.setAttribute('data-theme', theme);
      }
      set(theme);
    },
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

export const theme = createThemeStore();
