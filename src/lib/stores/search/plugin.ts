import { writable, get, type Writable } from 'svelte/store';
import { pluginService } from '$lib/plugins/service';
import { SEARCH_CONFIG } from '$lib/config';
import type { PluginSearchResult } from '$lib/plugins/types';

export interface PluginSearchState {
  results: PluginSearchResult[];
  isSearching: boolean;
  error: string | null;
}

class PluginSearchStore {
  state: Writable<PluginSearchState>;
  private debounceTimer: ReturnType<typeof setTimeout> | null = null;

  constructor() {
    this.state = writable<PluginSearchState>({
      results: [],
      isSearching: false,
      error: null
    });
  }

  async search(query: string): Promise<PluginSearchResult[]> {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }

    if (!query.trim()) {
      this.state.update(s => ({ ...s, results: [], isSearching: false, error: null }));
      return [];
    }

    return new Promise((resolve) => {
      this.debounceTimer = setTimeout(async () => {
        this.state.update(s => ({ ...s, isSearching: true, error: null }));

        try {
          console.log(`[PluginSearchStore] 🔍 开始插件搜索: "${query}"`);

          const matchedPlugins = await pluginService.searchByPrefix(query);

          if (matchedPlugins.length === 0) {
            console.log('[PluginSearchStore] ℹ️ 未找到匹配的插件');
            this.state.update(s => ({ ...s, results: [], isSearching: false }));
            resolve([]);
            return;
          }

          console.log(`[PluginSearchStore] ✅ 找到 ${matchedPlugins.length} 个匹配插件:`,
            matchedPlugins.map(p => p.name));

          const searchPromises = matchedPlugins.map(async (plugin) => {
            try {
              const results = await pluginService.executeSearch(plugin.name, query);
              const enrichedResults: PluginSearchResult[] = results.map(r => ({
                ...r,
                pluginId: plugin.name,
                pluginName: plugin.description || plugin.name
              }));
              console.log(`[PluginSearchStore] ✅ 插件 ${plugin.name} 返回 ${results.length} 个结果`);
              return enrichedResults;
            } catch (e) {
              console.error(`[PluginSearchStore] ❌ 插件 ${plugin.name} 搜索失败:`, e);
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

          console.log(`[PluginSearchStore] ✅ 插件搜索完成，共 ${allPluginResults.length} 个结果`);
          this.state.update(s => ({ ...s, results: allPluginResults, isSearching: false }));
          resolve(allPluginResults);

        } catch (e) {
          console.error('[PluginSearchStore] ❌ 插件搜索过程出错:', e);
          this.state.update(s => ({ ...s, results: [], isSearching: false, error: String(e) }));
          resolve([]);
        }
      }, SEARCH_CONFIG.DEBOUNCE_DELAY);
    });
  }

  clear() {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }
    this.state.set({ results: [], isSearching: false, error: null });
  }

  getResults(): PluginSearchResult[] {
    return get(this.state).results;
  }
}

export const pluginSearchStore = new PluginSearchStore();
