import { writable, derived, type Readable, type Writable } from 'svelte/store';
import { search, type SearchItem } from '$lib/search/fuzzy';
import { createSystemItems, type ExecutableItem } from '$lib/services/executor';

/** 导出搜索项类型 */
export type { SearchItem };
export type { ExecutableItem };

/**
 * 搜索状态管理类
 * 管理搜索查询、搜索结果和搜索数据项
 */
class SearchStore {
  /** 搜索查询词 */
  query: Writable<string>;
  /** 搜索结果（派生自 query 和 items） */
  results: Readable<any[]>;
  /** 搜索数据项列表 */
  items: Writable<ExecutableItem[]>;

  constructor() {
    // 使用真实的系统内置项替代测试数据
    this.items = writable<ExecutableItem[]>(createSystemItems());
    this.query = writable('');
    this.results = derived(
      [this.query, this.items],
      ([$query, $items]) => {
        if (!$query.trim()) return [];
        return search($query, $items);
      }
    );
  }

  /**
   * 设置搜索查询词
   * @param q - 查询词
   */
  setQuery(q: string) {
    this.query.set(q);
  }

  /**
   * 清空搜索查询词
   */
  clearQuery() {
    this.query.set('');
  }

  /**
   * 添加搜索项
   * @param item - 要添加的搜索项
   */
  addItem(item: ExecutableItem) {
    this.items.update(items => [...items, item]);
  }

  /**
   * 移除搜索项
   * @param id - 要移除的项 ID
   */
  removeItem(id: string) {
    this.items.update(items => items.filter(item => item.id !== id));
  }

  /**
   * 重置为系统默认项
   */
  resetToDefaults() {
    this.items.set(createSystemItems());
  }
}

/** 搜索状态管理实例 */
export const searchStore = new SearchStore();
