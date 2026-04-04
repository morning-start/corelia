import { writable, get } from 'svelte/store';
import { storeService } from '$lib/services/store';

export interface HistoryItem {
  query: string;
  timestamp: number;
  count: number;
}

interface SearchHistoryState {
  items: HistoryItem[];
  maxItems: number;
}

function createHistoryStore() {
  const { subscribe, set, update } = writable<SearchHistoryState>({
    items: [],
    maxItems: 100
  });

  return {
    subscribe,

    async init() {
      try {
        const stored = await storeService.load('search_history');
        if (stored && Array.isArray(stored)) {
          set({ items: stored, maxItems: 100 });
        }
      } catch (e) {
        console.error('Failed to load search history:', e);
      }
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

        if (newItems.length > history.maxItems) {
          newItems = newItems
            .sort((a, b) => b.count - a.count || b.timestamp - a.timestamp)
            .slice(0, history.maxItems);
        }

        storeService.save('search_history', newItems).catch(console.error);

        return { ...history, items: newItems };
      });
    },

    async clear() {
      set({ items: [], maxItems: 100 });
      await storeService.delete('search_history');
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
