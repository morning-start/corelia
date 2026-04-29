import { writable, derived, get, type Readable, type Writable } from 'svelte/store';
import { search, type SearchItem, type FilterResult } from '$lib/search/fuzzy';
import { createSystemItems, type ExecutableItem } from '$lib/services/executor';

export type { SearchItem };
export type { ExecutableItem };

export interface SystemSearchResult {
  original: ExecutableItem;
  score: number;
}

class SystemSearchStore {
  items: Writable<ExecutableItem[]>;
  results: Readable<SystemSearchResult[]>;

  constructor() {
    this.items = writable<ExecutableItem[]>(createSystemItems());
    this.results = derived(
      [writable(''), this.items],
      ([$query, $items], set) => {
        if (!$query.trim()) {
          set([]);
          return;
        }
        const searchResults = search($query, $items as unknown as SearchItem[]);
        const systemResults: SystemSearchResult[] = searchResults.map(result => ({
          original: result.original as unknown as ExecutableItem,
          score: result.score
        }));
        set(systemResults);
      }
    );
  }

  search(query: string): SystemSearchResult[] {
    if (!query.trim()) return [];
    const items = get(this.items);
    const searchResults = search(query, items as unknown as SearchItem[]);
    const systemResults: SystemSearchResult[] = searchResults.map(result => ({
      original: result.original as unknown as ExecutableItem,
      score: result.score
    }));
    return systemResults;
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
}

export const systemSearchStore = new SystemSearchStore();
