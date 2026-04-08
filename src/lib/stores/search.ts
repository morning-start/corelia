import { writable, derived, type Readable, type Writable } from 'svelte/store';
import { search, generateTestData, type SearchItem } from '$lib/search/fuzzy';

/** 导出搜索项类型 */
export type { SearchItem };

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
  items: Writable<SearchItem[]>;

  constructor() {
    this.items = writable<SearchItem[]>(generateTestData(100));
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
}

/** 搜索状态管理实例 */
export const searchStore = new SearchStore();
