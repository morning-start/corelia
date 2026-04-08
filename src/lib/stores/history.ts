import { writable, get } from 'svelte/store';
import { storeService } from '$lib/services/store';

/**
 * 搜索历史项
 */
export interface HistoryItem {
  /** 搜索词 */
  query: string;
  /** 时间戳 */
  timestamp: number;
  /** 搜索次数 */
  count: number;
}

/** 搜索历史状态 */
interface SearchHistoryState {
  /** 历史记录列表 */
  items: HistoryItem[];
  /** 最大保存条数 */
  maxItems: number;
}

/**
 * 创建搜索历史状态管理
 * 管理搜索历史的添加、查询和清理
 */
function createHistoryStore() {
  const { subscribe, set, update } = writable<SearchHistoryState>({
    items: [],
    maxItems: 100
  });

  return {
    subscribe,

    /**
     * 初始化，加载持久化的历史记录
     */
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

    /**
     * 添加搜索词到历史
     * @param query - 搜索词
     */
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

    /**
     * 清空搜索历史
     */
    async clear() {
      set({ items: [], maxItems: 100 });
      await storeService.delete('search_history');
    },

    /**
     * 获取最近的历史记录
     * @param limit - 返回条数限制
     * @returns 历史搜索词列表
     */
    getRecent(limit: number = 10): string[] {
      const state = get({ subscribe });
      return state.items
        .sort((a, b) => b.timestamp - a.timestamp)
        .slice(0, limit)
        .map(item => item.query);
    }
  };
}

/** 搜索历史状态管理实例 */
export const searchHistory = createHistoryStore();
