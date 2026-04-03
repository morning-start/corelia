import { filter, type FilterResult } from 'fuzzy';
import { pinyin } from 'pinyin-pro';

export interface SearchItem {
  id: string;
  name: string;
  description: string;
  category: string;
}

export function search(query: string, items: SearchItem[]): FilterResult<SearchItem>[] {
  const queryPinyin = pinyin(query, { toneType: 'none' }).replace(/\s+/g, '');

  return filter(query, items, {
    extract: (item) => {
      const namePinyin = pinyin(item.name, { toneType: 'none' }).replace(/\s+/g, '');
      const descPinyin = pinyin(item.description, { toneType: 'none' }).replace(/\s+/g, '');
      return `${item.name} ${namePinyin} ${item.description} ${descPinyin}`;
    },
  });
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
