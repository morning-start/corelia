import { filter, type FilterResult } from 'fuzzy';
import { pinyin } from 'pinyin-pro';

export type { FilterResult };

export interface SearchItem {
  id: string;
  name: string;
  description: string;
  category: string;
}

function containsChinese(str: string): boolean {
  return /[\u4e00-\u9fa5]/.test(str);
}

const pinyinCache = new Map<string, string>();

function getPinyin(str: string): string {
  if (!containsChinese(str)) {
    return str;
  }
  const cached = pinyinCache.get(str);
  if (cached) return cached;
  const result = pinyin(str, { toneType: 'none' }).replace(/\s+/g, '');
  pinyinCache.set(str, result);
  return result;
}

export interface SearchIndex {
  item: SearchItem;
  searchText: string;
}

export class IncrementalSearchIndex {
  private index: Map<string, SearchIndex> = new Map();
  private items: SearchItem[] = [];

  buildIndex(items: SearchItem[]): void {
    this.items = items;
    this.index.clear();

    for (const item of items) {
      const namePinyin = getPinyin(item.name);
      const descPinyin = getPinyin(item.description);
      const searchText = `${item.name} ${namePinyin} ${item.description} ${descPinyin}`.toLowerCase();
      
      this.index.set(item.id, {
        item,
        searchText
      });
    }

    console.log(`[SearchIndex] 已构建索引，共 ${this.index.size} 项`);
  }

  addItem(item: SearchItem): void {
    const namePinyin = getPinyin(item.name);
    const descPinyin = getPinyin(item.description);
    const searchText = `${item.name} ${namePinyin} ${item.description} ${descPinyin}`.toLowerCase();
    
    this.index.set(item.id, { item, searchText });
    this.items.push(item);
  }

  removeItem(id: string): void {
    this.index.delete(id);
    this.items = this.items.filter(item => item.id !== id);
  }

  search(query: string): FilterResult<SearchItem>[] {
    const queryLower = query.toLowerCase();
    const queryHasChinese = containsChinese(query);
    const queryPinyin = queryHasChinese ? getPinyin(query) : query;
    const queryText = `${queryLower} ${queryPinyin.toLowerCase()}`;

    const matchedItems: SearchItem[] = [];
    
    for (const [_, indexEntry] of this.index) {
      if (indexEntry.searchText.includes(queryText)) {
        matchedItems.push(indexEntry.item);
      }
    }

    return filter(query, matchedItems, {
      extract: (item) => {
        const entry = this.index.get(item.id);
        return entry ? entry.searchText : `${item.name} ${item.description}`;
      },
    });
  }

  getIndexSize(): number {
    return this.index.size;
  }

  clear(): void {
    this.index.clear();
    this.items = [];
    pinyinCache.clear();
  }
}

let globalIndex: IncrementalSearchIndex | null = null;

export function getSearchIndex(): IncrementalSearchIndex {
  if (!globalIndex) {
    globalIndex = new IncrementalSearchIndex();
  }
  return globalIndex;
}

export function search(query: string, items: SearchItem[]): FilterResult<SearchItem>[] {
  const index = getSearchIndex();
  
  if (index.getIndexSize() !== items.length) {
    index.buildIndex(items);
  }

  return index.search(query);
}

export function generateTestData(count: number): SearchItem[] {
  const categories = ['系统', '插件', '历史'];
  const testItems = [
    { name: '文档', description: '文本文档' },
    { name: '图片', description: '图片文件' },
    { name: '视频', description: '视频文件' },
    { name: '音乐', description: '音乐文件' },
    { name: '下载', description: '下载文件夹' },
    { name: '设置', description: '系统设置' },
    { name: '计算器', description: '简单计算' },
    { name: '记事本', description: '文本编辑' },
  ];

  return Array.from({ length: count }, (_, i) => ({
    id: `item-${i}`,
    name: testItems[i % testItems.length].name,
    description: testItems[i % testItems.length].description,
    category: categories[i % 3],
  }));
}
