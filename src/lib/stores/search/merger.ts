import { get } from 'svelte/store';
import type { ExecutableItem } from './system';
import type { PluginSearchResult } from '$lib/plugins/types';

export interface ExtendedSearchResult {
  original: ExecutableItem;
  score: number;
  isPlugin: boolean;
}

export interface SystemSearchResult {
  original: ExecutableItem;
  score: number;
}

export function mergeResults(
  systemResults: SystemSearchResult[],
  pluginResults: PluginSearchResult[]
): ExtendedSearchResult[] {
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

  return [...extendedSystemResults, ...extendedPluginResults];
}

export function sortResults(results: ExtendedSearchResult[]): ExtendedSearchResult[] {
  return [...results].sort((a, b) => b.score - a.score);
}
