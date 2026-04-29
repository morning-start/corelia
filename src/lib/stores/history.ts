/**
 * 搜索历史 Store
 *
 * 职责：
 * - 管理 UI 响应式状态
 * - 暴露给组件使用的响应式数据
 * - 委托业务逻辑给 historyService
 *
 * 注意：此 Store 仅负责 UI 状态管理
 * 业务逻辑和持久化由 historyService 负责
 */

import { historyService, type HistoryItem as ServiceHistoryItem } from '$lib/services/history';

export type HistoryItem = ServiceHistoryItem;

interface SearchHistoryState {
  items: HistoryItem[];
}

let state = $state<SearchHistoryState>({
  items: [],
});

export const searchHistory = {
  /**
   * 获取所有历史记录（响应式）
   */
  get items() {
    return state.items;
  },

  /**
   * 初始化 Store
   * 从 service 加载数据并同步到响应式状态
   */
  async init(): Promise<void> {
    await historyService.init();
    this.syncFromService();
  },

  /**
   * 设置最大容量
   *
   * @param capacity - 最大历史记录条数
   */
  setMaxCapacity(capacity: number): void {
    historyService.setMaxCapacity(capacity);
    this.syncFromService();
  },

  /**
   * 添加搜索记录
   *
   * @param query - 搜索词
   */
  add(query: string): void {
    historyService.add(query);
    this.syncFromService();
  },

  /**
   * 清空所有历史记录
   */
  async clear(): Promise<void> {
    await historyService.clear();
    this.syncFromService();
  },

  /**
   * 删除指定的历史记录
   *
   * @param query - 要删除的搜索词
   */
  remove(query: string): void {
    historyService.remove(query);
    this.syncFromService();
  },

  /**
   * 获取最近的搜索记录
   *
   * @param limit - 返回条数限制
   * @returns 搜索词列表
   */
  getRecent(limit: number = 10): string[] {
    return historyService.getRecent(limit);
  },

  /**
   * 内部方法：从 service 同步数据到响应式状态
   */
  syncFromService(): void {
    state = { items: historyService.getAll() };
  }
};
