import { search, generateTestData, type SearchItem } from './fuzzy';

export async function performanceTest(iterations: number = 100): Promise<{
  avg: number;
  max: number;
  min: number;
  success: boolean;
}> {
  const items = generateTestData(1000);
  const times: number[] = [];

  for (let i = 0; i < iterations; i++) {
    const query = `搜索项 ${Math.floor(Math.random() * 1000)}`;
    const start = performance.now();
    search(query, items);
    const end = performance.now();
    times.push(end - start);
  }

  const avg = times.reduce((a, b) => a + b, 0) / times.length;
  const max = Math.max(...times);
  const min = Math.min(...times);
  const success = avg < 50;

  return { avg, max, min, success };
}

export function testBasicSearch(): boolean {
  const items: SearchItem[] = [
    { id: '1', name: '文档', description: '文本文档', category: '系统' },
    { id: '2', name: '图片', description: '图片文件', category: '系统' },
    { id: '3', name: '视频', description: '视频文件', category: '系统' },
  ];

  const results = search('wendang', items);
  return results.length > 0 && results[0].original.name === '文档';
}

export function testFuzzySearch(): boolean {
  const items: SearchItem[] = [
    { id: '1', name: '文档', description: '文本文档', category: '系统' },
    { id: '2', name: '图片', description: '图片文件', category: '系统' },
    { id: '3', name: '视频', description: '视频文件', category: '系统' },
  ];

  const results = search('wdd', items);
  return results.length > 0 && results[0].original.name === '文档';
}

export function testPinyinSearch(): boolean {
  const items: SearchItem[] = [
    { id: '1', name: '文档', description: '文本文档', category: '系统' },
    { id: '2', name: '图片', description: '图片文件', category: '系统' },
    { id: '3', name: '视频', description: '视频文件', category: '系统' },
  ];

  const results = search('wd', items);
  return results.length > 0 && results[0].original.name === '文档';
}
