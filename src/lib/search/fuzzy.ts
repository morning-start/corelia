import { filter, type FilterResult } from 'fuzzy';
import { pinyin } from 'pinyin-pro';

export interface SearchItem {
  id: string;
  name: string;
  description: string;
  category: string;
}

/** 判断字符串是否包含中文字符 */
function containsChinese(str: string): boolean {
  return /[\u4e00-\u9fa5]/.test(str);
}

/** 拼音索引缓存 */
const pinyinCache = new Map<string, string>();

/** 获取字符串的拼音（带缓存） */
function getPinyin(str: string): string {
  if (!containsChinese(str)) {
    return str; // 无中文，直接返回原字符串
  }
  const cached = pinyinCache.get(str);
  if (cached) return cached;
  const result = pinyin(str, { toneType: 'none' }).replace(/\s+/g, '');
  pinyinCache.set(str, result);
  return result;
}

export function search(query: string, items: SearchItem[]): FilterResult<SearchItem>[] {
  // 如果查询词不含中文，跳过拼音转换
  const queryHasChinese = containsChinese(query);
  const queryPinyin = queryHasChinese ? getPinyin(query) : query;

  return filter(query, items, {
    extract: (item) => {
      const namePinyin = getPinyin(item.name);
      const descPinyin = getPinyin(item.description);
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
