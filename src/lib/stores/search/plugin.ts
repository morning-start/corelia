import type { PluginSearchResult } from '$lib/plugins/types';
import { pluginService } from '$lib/plugins/service';
import { SEARCH_CONFIG } from '$lib/config';

export class PluginSearchHandler {
  pluginResults: PluginSearchResult[] = [];
  private debounceTimer: ReturnType<typeof setTimeout> | null = null;

  private getDebouncedQuery(
    query: string,
    onResults: (results: PluginSearchResult[]) => void
  ): void {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }

    if (!query.trim()) {
      this.pluginResults = [];
      onResults([]);
      return;
    }

    this.debounceTimer = setTimeout(async () => {
      try {
        const matchedPlugins = await pluginService.searchByPrefix(query);

        if (matchedPlugins.length === 0) {
          this.pluginResults = [];
          onResults([]);
          return;
        }

        const searchPromises = matchedPlugins.map(async (plugin) => {
          try {
            const results = await pluginService.executeSearch(plugin.name, query);
            return results.map(r => ({
              ...r,
              pluginId: plugin.name,
              pluginName: plugin.description || plugin.name
            }));
          } catch (e) {
            console.error(`[PluginSearch] 插件 ${plugin.name} 搜索失败:`, e);
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

        this.pluginResults = allPluginResults;
        onResults(allPluginResults);
      } catch (e) {
        console.error('[PluginSearch] 搜索过程出错:', e);
        this.pluginResults = [];
        onResults([]);
      }
    }, SEARCH_CONFIG.DEBOUNCE_DELAY);
  }

  search(query: string, callback: (results: PluginSearchResult[]) => void): void {
    this.getDebouncedQuery(query, callback);
  }

  cancel(): void {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
      this.debounceTimer = null;
    }
  }

  clear(): void {
    this.cancel();
    this.pluginResults = [];
  }
}