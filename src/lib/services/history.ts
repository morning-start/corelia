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
const DEBOUNCE_DELAY = 500;

class HistoryService {
  private items: HistoryItem[] = [];
  private maxCapacity: number = DEFAULT_MAX_CAPACITY;
  private pendingSave: boolean = false;
  private saveTimer: ReturnType<typeof setTimeout> | null = null;

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

  setMaxCapacity(capacity: number): void {
    this.maxCapacity = capacity;
    this.trimIfNeeded();
    this.scheduleSave();
  }

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
    this.scheduleSave();
  }

  async clear(): Promise<void> {
    this.items = [];
    this.cancelPendingSave();
    await api.store.delete(STORAGE_KEY);
  }

  getRecent(limit: number = 10): string[] {
    return [...this.items]
      .sort((a, b) => b.timestamp - a.timestamp)
      .slice(0, limit)
      .map(item => item.query);
  }

  getAll(): HistoryItem[] {
    return [...this.items]
      .sort((a, b) => b.count - a.count || b.timestamp - a.timestamp);
  }

  remove(query: string): void {
    this.items = this.items.filter(item => item.query !== query);
    this.scheduleSave();
  }

  private trimIfNeeded(): void {
    if (this.items.length > this.maxCapacity) {
      this.items = this.items
        .sort((a, b) => b.count - a.count || b.timestamp - a.timestamp)
        .slice(0, this.maxCapacity);
    }
  }

  private scheduleSave(): void {
    if (this.pendingSave) return;
    
    this.pendingSave = true;
    this.saveTimer = setTimeout(() => {
      this.flushSave();
    }, DEBOUNCE_DELAY);
  }

  private cancelPendingSave(): void {
    if (this.saveTimer) {
      clearTimeout(this.saveTimer);
      this.saveTimer = null;
    }
    this.pendingSave = false;
  }

  private flushSave(): void {
    this.pendingSave = false;
    this.saveTimer = null;
    this.save();
  }

  private save(): void {
    api.store.save(STORAGE_KEY, this.items).catch(e => {
      console.error('[HistoryService] 保存历史记录失败:', e);
    });
  }
}

export const historyService = new HistoryService();
export default historyService;
