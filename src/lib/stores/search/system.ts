import { writable, derived, get, type Readable, type Writable } from 'svelte/store';
import { search, type SearchItem } from '$lib/search/fuzzy';
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
        set(search($query, $items));
      }
    );
  }

  search(query: string): SystemSearchResult[] {
    if (!query.trim()) return [];
    const items = get(this.items);
    return search(query, items);
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
