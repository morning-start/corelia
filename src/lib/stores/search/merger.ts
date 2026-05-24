import { writable, get, type Writable } from 'svelte/store';
import type { ExecutableItem } from '$lib/services/executor';
import type { PluginSearchResult } from '$lib/plugins/types';

export interface ExtendedSearchResult {
  original: ExecutableItem;
  score: number;
  isPlugin?: boolean;
}

export class SearchResultMerger {
  results: Writable<ExtendedSearchResult[]>;

  constructor() {
    this.results = writable<ExtendedSearchResult[]>([]);
  }

  merge(
    systemResults: any[],
    pluginResults: PluginSearchResult[]
  ): void {
    const extendedSystemResults: ExtendedSearchResult[] = systemResults.map(r => ({
      original: r.original,
      score: r.score,
      isPlugin: false
    }));

    const extendedPluginResults: ExtendedSearchResult[] = pluginResults.map((r, index) => {
      const executableItem: ExecutableItem = {
        id: `plugin_${r.pluginId}_${r.action}_${index}`,
        name: r.title,
        description: r.description,
        category: 'plugin',
        type: 'plugin',
        target: r.pluginId || 'unknown',
        args: [r.action],
        hideWindow: true,
      };

      return {
        original: executableItem,
        score: 0.8 + (index * 0.01),
        isPlugin: true
      };
    });

    const merged = [...extendedSystemResults, ...extendedPluginResults];
    this.results.set(merged);
  }

  clear(): void {
    this.results.set([]);
  }

  getResults(): ExtendedSearchResult[] {
    return get(this.results);
  }
}