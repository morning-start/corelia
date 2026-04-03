import { writable, derived, type Readable, type Writable } from 'svelte/store';
import { search, generateTestData, type SearchItem } from '$lib/search/fuzzy';

export type { SearchItem };

class SearchStore {
  query: Writable<string>;
  results: Readable<any[]>;
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

  setQuery(q: string) {
    this.query.set(q);
  }

  clearQuery() {
    this.query.set('');
  }
}

export const searchStore = new SearchStore();
