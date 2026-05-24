import { api } from '$lib/api';

export interface HistoryItem {
  query: string;
  timestamp: number;
  count: number;
}

interface SearchHistoryState {
  items: HistoryItem[];
  maxCapacity: number;
}

const DEFAULT_MAX_CAPACITY = 100;

let historyState = $state<SearchHistoryState>({
  items: [],
  maxCapacity: DEFAULT_MAX_CAPACITY,
});

const subscribers = new Set<(value: SearchHistoryState) => void>();

function notify() {
  subscribers.forEach(fn => fn(historyState));
}

function subscribe(this: any, run: (value: SearchHistoryState) => void): () => void {
  subscribers.add(run);
  run(historyState);
  return () => subscribers.delete(run);
}

async function init() {
  try {
    const stored = await api.store.load('search_history');
    if (stored && Array.isArray(stored)) {
      historyState = { items: stored as HistoryItem[], maxCapacity: historyState.maxCapacity };
      notify();
    }
  } catch (e) {
    console.error('Failed to load search history:', e);
  }
}

function setMaxCapacity(capacity: number) {
  historyState = { ...historyState, maxCapacity: capacity };
  notify();
}

function add(query: string) {
  if (!query.trim()) return;

  const existing = historyState.items.find(item => item.query === query);
  let newItems: HistoryItem[];

  if (existing) {
    existing.count++;
    existing.timestamp = Date.now();
    newItems = [...historyState.items];
  } else {
    newItems = [
      { query, timestamp: Date.now(), count: 1 },
      ...historyState.items
    ];
  }

  if (newItems.length > historyState.maxCapacity) {
    newItems = newItems
      .sort((a, b) => b.count - a.count || b.timestamp - a.timestamp)
      .slice(0, historyState.maxCapacity);
  }

  historyState = { ...historyState, items: newItems };
  notify();

  scheduleSave(newItems);
}

let writeTimer: ReturnType<typeof setTimeout> | null = null;

function scheduleSave(items: HistoryItem[]) {
  if (writeTimer) clearTimeout(writeTimer);
  writeTimer = setTimeout(() => {
    api.store.save('search_history', items).catch(console.error);
    writeTimer = null;
  }, 2000);
}

async function clear() {
  historyState = { items: [], maxCapacity: DEFAULT_MAX_CAPACITY };
  notify();
  await api.store.delete('search_history');
}

function getRecent(limit: number = 10): string[] {
  return historyState.items
    .sort((a, b) => b.timestamp - a.timestamp)
    .slice(0, limit)
    .map(item => item.query);
}

export const searchHistory = {
  subscribe,
  init,
  setMaxCapacity,
  add,
  clear,
  getRecent,
};