import { writable, derived, type Readable, type Writable } from 'svelte/store';
import { search } from '$lib/search/fuzzy';
import { createSystemItems, type ExecutableItem } from '$lib/services/executor';

export class SystemSearchProvider {
  items: Writable<ExecutableItem[]>;
  systemResults: Readable<any[]>;

  constructor(query: Readable<string>) {
    this.items = writable<ExecutableItem[]>(createSystemItems());

    this.systemResults = derived(
      [query, this.items],
      ([$query, $items]) => {
        if (!$query.trim()) return [];
        return search($query, $items);
      }
    );
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