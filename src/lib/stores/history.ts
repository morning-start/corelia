import { writable, get } from 'svelte/store';
import { api } from '$lib/api';
import { SEARCH_CONFIG, type UserConfig } from '$lib/config';

export interface HistoryItem {
  query: string;
  timestamp: number;
  count: number;
}

interface SearchHistoryState {
  items: HistoryItem[];
  /** 历史记录存储最大容量 */
  maxCapacity: number;
}

/** 默认历史容量，与 UserConfig.search.maxHistoryCapacity 保持一致 */
const DEFAULT_MAX_CAPACITY = 100;

function createHistoryStore() {
  const { subscribe, set, update } = writable<SearchHistoryState>({
    items: [],
    maxCapacity: DEFAULT_MAX_CAPACITY,
  });

  return {
    subscribe,

    async init() {
      try {
        const stored = await api.store.load('search_history');
        if (stored && Array.isArray(stored)) {
          set({ items: stored as HistoryItem[], maxCapacity: DEFAULT_MAX_CAPACITY });
        }
      } catch (e) {
        console.error('Failed to load search history:', e);
      }
    },

    /** 设置最大容量（从 UserConfig 加载后调用） */
    setMaxCapacity(capacity: number) {
      update(state => ({ ...state, maxCapacity: capacity }));
    },

    add(query: string) {
      if (!query.trim()) return;

      update(history => {
        const existing = history.items.find(item => item.query === query);
        let newItems: HistoryItem[];

        if (existing) {
          existing.count++;
          existing.timestamp = Date.now();
          newItems = [...history.items];
        } else {
          newItems = [
            { query, timestamp: Date.now(), count: 1 },
            ...history.items
          ];
        }

        // 超出容量时按使用频率+时间裁剪
        if (newItems.length > history.maxCapacity) {
          newItems = newItems
            .sort((a, b) => b.count - a.count || b.timestamp - a.timestamp)
            .slice(0, history.maxCapacity);
        }

        api.store.save('search_history', newItems).catch(console.error);

        return { ...history, items: newItems };
      });
    },

    async clear() {
      set({ items: [], maxCapacity: DEFAULT_MAX_CAPACITY });
      await api.store.delete('search_history');
    },

    getRecent(limit: number = 10): string[] {
      const state = get({ subscribe });
      return state.items
        .sort((a, b) => b.timestamp - a.timestamp)
        .slice(0, limit)
        .map(item => item.query);
    }
  };
}

export const searchHistory = createHistoryStore();
