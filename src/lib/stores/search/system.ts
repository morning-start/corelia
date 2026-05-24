import { writable, derived, type Readable, type Writable } from 'svelte/store';
import { pinyin } from 'pinyin-pro';
import { search, buildSearchIndex, type SearchItem } from '$lib/search/fuzzy';
import { createSystemItems, type ExecutableItem } from '$lib/services/executor';

export class SystemSearchProvider {
  items: Writable<ExecutableItem[]>;
  systemResults: Readable<any[]>;

  constructor(query: Readable<string>) {
    const rawItems = createSystemItems();
    this.items = writable<ExecutableItem[]>(buildSearchIndex(rawItems));

    this.systemResults = derived(
      [query, this.items],
      ([$query, $items]) => {
        if (!$query.trim()) return [];
        return search($query, $items);
      }
    );
  }

  addItem(item: ExecutableItem) {
    this.items.update(items => [...items, { ...item, searchText: buildSearchText(item) }]);
  }

  removeItem(id: string) {
    this.items.update(items => items.filter(item => item.id !== id));
  }

  resetToDefaults() {
    this.items.set(buildSearchIndex(createSystemItems()));
  }
}

function buildSearchText(item: Pick<SearchItem, 'name' | 'description'>): string {
  const namePinyin = pinyin(item.name, { toneType: 'none' }).replace(/\s+/g, '');
  const descPinyin = pinyin(item.description, { toneType: 'none' }).replace(/\s+/g, '');
  return `${item.name} ${namePinyin} ${item.description} ${descPinyin}`;
}