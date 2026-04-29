import { writable, derived, get, type Readable, type Writable } from 'svelte/store';
import { search, type SearchItem } from '$lib/search/fuzzy';
import { createSystemItems, type ExecutableItem } from '$lib/services/executor';
import { pluginService } from '$lib/plugins/service';
import { SEARCH_CONFIG } from '$lib/config';
import type { PluginSearchResult } from '$lib/plugins/types';
import { mergeResults, type ExtendedSearchResult } from './merger';

export type { SearchItem };
export type { ExecutableItem };
export type { ExtendedSearchResult };

class SearchStore {
  query: Writable<string>;
  systemResults: Readable<any[]>;
  results: Writable<ExtendedSearchResult[]>;
  items: Writable<ExecutableItem[]>;
  private pluginResults: PluginSearchResult[] = [];
  private debounceTimer: ReturnType<typeof setTimeout> | null = null;

  constructor() {
    this.items = writable<ExecutableItem[]>(createSystemItems());
    this.query = writable('');
    this.results = writable<ExtendedSearchResult[]>([]);

    this.systemResults = derived(
      [this.query, this.items],
      ([$query, $items]) => {
        if (!$query.trim()) return [];
        return search($query, $items);
      }
    );

    this.query.subscribe($query => {
      this.performPluginSearch($query);
    });

    this.systemResults.subscribe(() => {
      this.mergeResults();
    });
  }

  setQuery(q: string) {
    this.query.set(q);
  }

  clearQuery() {
    this.query.set('');
    this.pluginResults = [];
    this.results.set([]);
  }

  addItem(item: ExecutableItem) {
    this.items.update(items => [...items, item]);
  }

  removeItem(id: string) {
    this.items.update(items => items.filter(item => item.id !== id));
  }

  resetToDefaults() {
    this.items.set(createSystemItems());
  }

  private async performPluginSearch(query: string): Promise<void> {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }

    if (!query.trim()) {
      this.pluginResults = [];
      this.mergeResults();
      return;
    }

    this.debounceTimer = setTimeout(async () => {
      try {
        console.log(`[SearchStore] 🔍 开始插件搜索: "${query}"`);

        const matchedPlugins = await pluginService.searchByPrefix(query);

        if (matchedPlugins.length === 0) {
          console.log('[SearchStore] ℹ️ 未找到匹配的插件');
          this.pluginResults = [];
          this.mergeResults();
          return;
        }

        console.log(`[SearchStore] ✅ 找到 ${matchedPlugins.length} 个匹配插件:`,
          matchedPlugins.map(p => p.name));

        const searchPromises = matchedPlugins.map(async (plugin) => {
          try {
            const results = await pluginService.executeSearch(plugin.name, query);
            const enrichedResults: PluginSearchResult[] = results.map(r => ({
              ...r,
              pluginId: plugin.name,
              pluginName: plugin.description || plugin.name
            }));
            console.log(`[SearchStore] ✅ 插件 ${plugin.name} 返回 ${results.length} 个结果`);
            return enrichedResults;
          } catch (e) {
            console.error(`[SearchStore] ❌ 插件 ${plugin.name} 搜索失败:`, e);
            return [] as PluginSearchResult[];
          }
        });

        const settledResults = await Promise.allSettled(searchPromises);
        const allPluginResults: PluginSearchResult[] = [];
        for (const result of settledResults) {
          if (result.status === 'fulfilled') {
            allPluginResults.push(...result.value);
          }
        }
        console.log(`[SearchStore] ✅ 插件搜索完成，共 ${allPluginResults.length} 个结果`);
        this.pluginResults = allPluginResults;
        this.mergeResults();

      } catch (e) {
        console.error('[SearchStore] ❌ 插件搜索过程出错:', e);
        this.pluginResults = [];
        this.mergeResults();
      }
    }, SEARCH_CONFIG.DEBOUNCE_DELAY);
  }

  private mergeResults(): void {
    const currentSystemResults = get(this.systemResults);
    const mergedResults = mergeResults(currentSystemResults, this.pluginResults);
    this.results.set(mergedResults);
  }

  async refreshPluginResults(): Promise<number> {
    const currentQuery = get(this.query);
    await this.performPluginSearch(currentQuery);
    return get(this.results).length;
  }
}

export const searchStore = new SearchStore();

export { mergeResults } from './merger';
export { systemSearchStore } from './system';
export { pluginSearchStore } from './plugin';
