import { writable } from 'svelte/store';
import { search, type SearchItem } from '$lib/search/fuzzy';
import type { ExecutableItem } from '$lib/services/executor';
import { SystemSearchProvider } from './system';
import { PluginSearchHandler } from './plugin';
import { SearchResultMerger, type ExtendedSearchResult } from './merger';

export type { SearchItem };
export type { ExecutableItem };
export type { ExtendedSearchResult };

class SearchStore {
  query = writable('');
  systemProvider: SystemSearchProvider;
  pluginHandler: PluginSearchHandler;
  merger: SearchResultMerger;

  constructor() {
    this.systemProvider = new SystemSearchProvider(this.query);
    this.pluginHandler = new PluginSearchHandler();
    this.merger = new SearchResultMerger();

    this.query.subscribe($query => {
      this.pluginHandler.search($query, () => {
        this.mergeResults();
      });
    });

    this.systemProvider.systemResults.subscribe(() => {
      this.mergeResults();
    });
  }

  get systemResults() {
    return this.systemProvider.systemResults;
  }

  get items() {
    return this.systemProvider.items;
  }

  get results() {
    return this.merger.results;
  }

  setQuery(q: string) {
    this.query.set(q);
  }

  clearQuery() {
    this.query.set('');
    this.pluginHandler.clear();
    this.merger.clear();
  }

  addItem(item: ExecutableItem) {
    this.systemProvider.addItem(item);
  }

  removeItem(id: string) {
    this.systemProvider.removeItem(id);
  }

  resetToDefaults() {
    this.systemProvider.resetToDefaults();
  }

  async refreshPluginResults(): Promise<number> {
    let currentQuery: string = '';
    this.query.subscribe(v => currentQuery = v)();
    this.pluginHandler.search(currentQuery, () => {
      this.mergeResults();
    });
    return this.merger.getResults().length;
  }

  private mergeResults(): void {
    let currentSystemResults: any[] = [];
    this.systemProvider.systemResults.subscribe(v => currentSystemResults = v)();
    this.merger.merge(currentSystemResults, this.pluginHandler.pluginResults);
  }
}

export const searchStore = new SearchStore();