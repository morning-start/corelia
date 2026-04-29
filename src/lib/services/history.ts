/**
 * 历史记录管理服务
 *
 * 职责：
 * - 搜索历史记录的增删改查
 * - 历史记录的持久化
 * - 历史记录的排序和过滤
 *
 * 注意：此服务仅负责业务逻辑，不包含 UI 状态管理
 * UI 状态由对应的 Store 负责
 */

import { api } from '$lib/api';
import { SEARCH_CONFIG } from '$lib/config';

export interface HistoryItem {
  query: string;
  timestamp: number;
  count: number;
}

const STORAGE_KEY = 'search_history';
const DEFAULT_MAX_CAPACITY = 100;

class HistoryService {
  private items: HistoryItem[] = [];
  private maxCapacity: number = DEFAULT_MAX_CAPACITY;

  /**
   * 初始化历史记录服务
   * 从持久化存储加载历史记录
   */
  async init(): Promise<void> {
    try {
      const stored = await api.store.load(STORAGE_KEY);
      if (stored && Array.isArray(stored)) {
        this.items = stored as HistoryItem[];
      }
    } catch (e) {
      console.error('[HistoryService] 加载历史记录失败:', e);
    }
  }

  /**
   * 设置最大容量
   *
   * @param capacity - 最大历史记录条数
   */
  setMaxCapacity(capacity: number): void {
    this.maxCapacity = capacity;
    this.trimIfNeeded();
  }

  /**
   * 添加搜索记录
   *
   * @param query - 搜索词
   */
  add(query: string): void {
    if (!query.trim()) return;

    const existing = this.items.find(item => item.query === query);

    if (existing) {
      existing.count++;
      existing.timestamp = Date.now();
    } else {
      this.items.unshift({
        query,
        timestamp: Date.now(),
        count: 1
      });
    }

    this.trimIfNeeded();
    this.save();
  }

  /**
   * 清空所有历史记录
   */
  async clear(): Promise<void> {
    this.items = [];
    await api.store.delete(STORAGE_KEY);
  }

  /**
   * 获取最近的搜索记录
   *
   * @param limit - 返回条数限制
   * @returns 搜索词列表
   */
  getRecent(limit: number = 10): string[] {
    return [...this.items]
      .sort((a, b) => b.timestamp - a.timestamp)
      .slice(0, limit)
      .map(item => item.query);
  }

  /**
   * 获取所有历史记录（按使用频率和时间排序）
   *
   * @returns 历史记录列表
   */
  getAll(): HistoryItem[] {
    return [...this.items]
      .sort((a, b) => b.count - a.count || b.timestamp - a.timestamp);
  }

  /**
   * 删除指定的历史记录
   *
   * @param query - 要删除的搜索词
   */
  remove(query: string): void {
    this.items = this.items.filter(item => item.query !== query);
    this.save();
  }

  /**
   * 内部方法：如果超过容量则裁剪
   */
  private trimIfNeeded(): void {
    if (this.items.length > this.maxCapacity) {
      this.items = this.items
        .sort((a, b) => b.count - a.count || b.timestamp - a.timestamp)
        .slice(0, this.maxCapacity);
    }
  }

  /**
   * 内部方法：保存到持久化存储
   */
  private save(): void {
    api.store.save(STORAGE_KEY, this.items).catch(e => {
      console.error('[HistoryService] 保存历史记录失败:', e);
    });
  }
}

/** 历史记录服务单例 */
export const historyService = new HistoryService();
export default historyService;
