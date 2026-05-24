import { writable, get } from 'svelte/store';
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

  private queryUnsubscribe: () => void;
  private systemResultsUnsubscribe: () => void;
  private pluginSearchPending = false;

  constructor() {
    this.systemProvider = new SystemSearchProvider(this.query);
    this.pluginHandler = new PluginSearchHandler();
    this.merger = new SearchResultMerger();

    this.queryUnsubscribe = this.query.subscribe($query => {
      this.pluginSearchPending = true;
      this.pluginHandler.search($query, () => {
        this.pluginSearchPending = false;
        this.mergeResults();
      });
    });

    this.systemResultsUnsubscribe = this.systemProvider.systemResults.subscribe(() => {
      if (!this.pluginSearchPending) {
        this.mergeResults();
      }
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
    const currentQuery = get(this.query);
    this.pluginHandler.search(currentQuery, () => {
      this.mergeResults();
    });
    return this.merger.getResults().length;
  }

  private mergeResults(): void {
    const systemResults = get(this.systemProvider.systemResults);
    this.merger.merge(systemResults, this.pluginHandler.pluginResults);
  }

  destroy(): void {
    this.queryUnsubscribe();
    this.systemResultsUnsubscribe();
    this.pluginHandler.cancel();
  }
}

export const searchStore = new SearchStore();