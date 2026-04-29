export type Theme = 'dark' | 'light' | 'system';

let mediaQuery: MediaQueryList | null = $state(null);
let mediaListener: ((e: MediaQueryListEvent) => void) | null = $state(null);

function getSystemTheme(): 'dark' | 'light' {
  if (typeof window === 'undefined') return 'dark';
  return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
}

function applyTheme(theme: Theme) {
  if (typeof document === 'undefined') return;
  const actualTheme = theme === 'system' ? getSystemTheme() : theme;
  document.documentElement.setAttribute('data-theme', actualTheme);
}

function setupMediaQuery() {
  if (typeof window === 'undefined') return;

  mediaQuery = window.matchMedia('(prefers-color-scheme: light)');
  mediaListener = (e: MediaQueryListEvent) => {
    applyTheme('system');
  };

  mediaQuery.addEventListener('change', mediaListener);
}

function cleanupMediaQuery() {
  if (mediaQuery && mediaListener) {
    mediaQuery.removeEventListener('change', mediaListener);
    mediaQuery = null;
    mediaListener = null;
  }
}

let currentTheme = $state<Theme>('dark');

export const theme = {
  get current() {
    return currentTheme;
  },

  set(value: Theme) {
    cleanupMediaQuery();
    applyTheme(value);

    if (value === 'system') {
      setupMediaQuery();
    }

    currentTheme = value;
  },

  toggle() {
    const next = currentTheme === 'dark' ? 'light' : 'dark';
    cleanupMediaQuery();
    applyTheme(next);
    currentTheme = next;
  }
};
