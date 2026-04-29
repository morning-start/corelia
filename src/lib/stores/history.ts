import { api } from '$lib/api';
import { SEARCH_CONFIG, type UserConfig } from '$lib/config';

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

let state = $state<SearchHistoryState>({
  items: [],
  maxCapacity: DEFAULT_MAX_CAPACITY,
});

export const searchHistory = {
  get items() {
    return state.items;
  },

  get maxCapacity() {
    return state.maxCapacity;
  },

  async init() {
    try {
      const stored = await api.store.load('search_history');
      if (stored && Array.isArray(stored)) {
        state = { items: stored as HistoryItem[], maxCapacity: DEFAULT_MAX_CAPACITY };
      }
    } catch (e) {
      console.error('Failed to load search history:', e);
    }
  },

  setMaxCapacity(capacity: number) {
    state = { ...state, maxCapacity: capacity };
  },

  add(query: string) {
    if (!query.trim()) return;

    const existing = state.items.find(item => item.query === query);
    let newItems: HistoryItem[];

    if (existing) {
      existing.count++;
      existing.timestamp = Date.now();
      newItems = [...state.items];
    } else {
      newItems = [
        { query, timestamp: Date.now(), count: 1 },
        ...state.items
      ];
    }

    if (newItems.length > state.maxCapacity) {
      newItems = newItems
        .sort((a, b) => b.count - a.count || b.timestamp - a.timestamp)
        .slice(0, state.maxCapacity);
    }

    api.store.save('search_history', newItems).catch(console.error);

    state = { ...state, items: newItems };
  },

  async clear() {
    state = { items: [], maxCapacity: DEFAULT_MAX_CAPACITY };
    await api.store.delete('search_history');
  },

  getRecent(limit: number = 10): string[] {
    return state.items
      .sort((a, b) => b.timestamp - a.timestamp)
      .slice(0, limit)
      .map(item => item.query);
  }
};
