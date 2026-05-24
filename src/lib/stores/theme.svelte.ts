export type Theme = 'dark' | 'light' | 'system';

let currentTheme = $state<Theme>('dark');
let mediaQuery: MediaQueryList | null = null;
let mediaListener: ((e: MediaQueryListEvent) => void) | null = null;
const subscribers = new Set<(value: Theme) => void>();

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

function setTheme(theme: Theme) {
  cleanupMediaQuery();
  applyTheme(theme);
  if (theme === 'system') {
    setupMediaQuery();
  }
  currentTheme = theme;
  subscribers.forEach(fn => fn(theme));
}

function toggleTheme() {
  const next = currentTheme === 'dark' ? 'light' : 'dark';
  setTheme(next);
}

function subscribe(this: any, run: (value: Theme) => void): () => void {
  subscribers.add(run);
  run(currentTheme);
  return () => subscribers.delete(run);
}

export const theme = {
  subscribe,
  set: setTheme,
  toggle: toggleTheme,
};