import { writable, derived, type Readable, type Writable } from 'svelte/store';
import { search, type SearchItem } from '$lib/search/fuzzy';
import { createSystemItems, type ExecutableItem } from '$lib/services/executor';
import { pluginService } from '$lib/plugins/service';
import type { PluginSearchResult } from '$lib/plugins/types';

/** 导出搜索项类型 */
export type { SearchItem };
export type { ExecutableItem };

/**
 * 扩展的搜索结果类型
 * 包含系统内置项和插件结果
 */
export interface ExtendedSearchResult {
  /** 原始模糊匹配结果 */
  original: ExecutableItem;
  /** 匹配得分 */
  score: number;
  /** 是否来自插件 */
  isPlugin?: boolean;
}

/**
 * 搜索状态管理类
 * 管理搜索查询、搜索结果和搜索数据项
 *
 * 搜索流程：
 * 1. 用户输入 → setQuery()
 * 2. 同步执行系统内置项模糊搜索
 * 3. 异步执行插件前缀匹配和 onSearch()
 * 4. 合并两类结果并更新 results store
 */
class SearchStore {
  /** 搜索查询词 */
  query: Writable<string>;
  /** 搜索结果（派生自 query 和 items）- 仅包含系统内置项 */
  systemResults: Readable<any[]>;
  /** 最终搜索结果（系统 + 插件） */
  results: Writable<ExtendedSearchResult[]>;
  /** 搜索数据项列表（仅系统内置项） */
  items: Writable<ExecutableItem[]>;
  /** 插件搜索结果缓存 */
  private pluginResults: PluginSearchResult[] = [];
  /** 防抖定时器 ID */
  private debounceTimer: ReturnType<typeof setTimeout> | null = null;

  constructor() {
    // 使用真实的系统内置项替代测试数据
    this.items = writable<ExecutableItem[]>(createSystemItems());
    this.query = writable('');
    this.results = writable<ExtendedSearchResult[]>([]);
    this.systemResults = derived(
      [this.query, this.items],
      ([$query, $items]) => {
        if (!$query.trim()) return [];
        return search($query, $items);
      }
    );

    // 监听查询变化，触发异步插件搜索
    this.query.subscribe($query => {
      this.performPluginSearch($query);
    });

    // 监听系统结果变化，合并最终结果
    this.systemResults.subscribe($systemResults => {
      this.mergeResults();
    });
  }

  /**
   * 设置搜索查询词
   * @param q - 查询词
   */
  setQuery(q: string) {
    this.query.set(q);
  }

  /**
   * 清空搜索查询词
   */
  clearQuery() {
    this.query.set('');
    this.pluginResults = [];
    this.results.set([]);
  }

  /**
   * 添加搜索项
   * @param item - 要添加的搜索项
   */
  addItem(item: ExecutableItem) {
    this.items.update(items => [...items, item]);
  }

  /**
   * 移除搜索项
   * @param id - 要移除的项 ID
   */
  removeItem(id: string) {
    this.items.update(items => items.filter(item => item.id !== id));
  }

  /**
   * 重置为系统默认项
   */
  resetToDefaults() {
    this.items.set(createSystemItems());
  }

  /**
   * 异步执行插件搜索
   *
   * 使用防抖机制避免频繁调用：
   * - 用户连续输入时，只执行最后一次搜索
   * - 防抖延迟 150ms（平衡响应速度和性能）
   *
   * @param query - 当前搜索词
   */
  private async performPluginSearch(query: string): Promise<void> {
    // 清除之前的防抖定时器
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }

    // 如果查询为空，清空插件结果
    if (!query.trim()) {
      this.pluginResults = [];
      this.mergeResults();
      return;
    }

    // 设置新的防抖定时器
    this.debounceTimer = setTimeout(async () => {
      try {
        console.log(`[SearchStore] 🔍 开始插件搜索: "${query}"`);

        // 1. 根据前缀查找匹配的插件
        const matchedPlugins = await pluginService.searchByPrefix(query);

        if (matchedPlugins.length === 0) {
          console.log('[SearchStore] ℹ️ 未找到匹配的插件');
          this.pluginResults = [];
          this.mergeResults();
          return;
        }

        console.log(`[SearchStore] ✅ 找到 ${matchedPlugins.length} 个匹配插件:`,
          matchedPlugins.map(p => p.name));

        // 2. 对每个匹配的插件执行 onSearch
        const allPluginResults: PluginSearchResult[] = [];

        for (const plugin of matchedPlugins) {
          try {
            const results = await pluginService.executeSearch(plugin.name, query);

            // 将插件结果标记来源
            const enrichedResults = results.map(r => ({
              ...r,
              pluginId: plugin.name,
              pluginName: plugin.description || plugin.name
            }));

            allPluginResults.push(...enrichedResults);
            console.log(`[SearchStore] ✅ 插件 ${plugin.name} 返回 ${results.length} 个结果`);
          } catch (e) {
            console.error(`[SearchStore] ❌ 插件 ${plugin.name} 搜索失败:`, e);
            // 单个插件失败不影响其他插件
          }
        }

        // 3. 更新插件结果缓存
        this.pluginResults = allPluginResults;
        console.log(`[SearchStore] ✅ 插件搜索完成，共 ${allPluginResults.length} 个结果`);

        // 4. 触发结果合并
        this.mergeResults();

      } catch (e) {
        console.error('[SearchStore] ❌ 插件搜索过程出错:', e);
        this.pluginResults = [];
        this.mergeResults();
      }
    }, 150); // 150ms 防抖延迟
  }

  /**
   * 合并系统搜索结果和插件搜索结果
   *
   * 合并策略：
   * - 系统内置项在前，插件结果在后
   * - 保持各自的原始排序
   * - 为插件结果创建统一的 ExecutableItem 格式
   */
  private mergeResults(): void {
    // 获取当前系统结果
    let currentSystemResults: any[] = [];
    const unsubscribe = this.systemResults.subscribe(value => {
      currentSystemResults = value;
    });
    unsubscribe();

    // 转换系统结果格式
    const extendedSystemResults: ExtendedSearchResult[] = currentSystemResults.map(r => ({
      original: r.original,
      score: r.score,
      isPlugin: false
    }));

    // 转换插件结果格式
    const extendedPluginResults: ExtendedSearchResult[] = this.pluginResults.map((r, index) => {
      // 将 PluginSearchResult 转换为 ExecutableItem 格式
      const executableItem: ExecutableItem = {
        id: `plugin_${r.pluginId}_${r.action}_${index}`,
        name: r.title,
        description: r.description,
        category: 'plugin', // 标记为插件类别
        type: 'plugin', // 执行类型为 plugin
        target: r.pluginId || 'unknown',
        args: [r.action], // 将 action 作为参数传递
        hideWindow: true, // 执行后默认隐藏窗口
        // 扩展字段：存储插件相关信息
        ...(r as any)
      };

      return {
        original: executableItem,
        score: 0.8 + (index * 0.01), // 插件结果给予较高基础分
        isPlugin: true
      };
    });

    // 合并结果
    const mergedResults = [...extendedSystemResults, ...extendedPluginResults];

    // 更新结果 store
    this.results.set(mergedResults);

    if (mergedResults.length > 0) {
      console.log(`[SearchStore] 📊 结果合并完成: 系统 ${extendedSystemResults.length} 个 + 插件 ${extendedPluginResults.length} 个 = 总计 ${mergedResults.length} 个`);
    }
  }

  /**
   * 手动刷新插件搜索结果（用于外部触发）
   *
   * @returns 当前的合并结果数量
   */
  async refreshPluginResults(): Promise<number> {
    const currentQuery = this.getQueryValue();
    await this.performPluginSearch(currentQuery);

    let count = 0;
    const unsubscribe = this.results.subscribe(results => {
      count = results.length;
    });
    unsubscribe();

    return count;
  }

  /**
   * 获取当前查询词值（同步）
   *
   * @returns 当前查询词
   */
  private getQueryValue(): string {
    let value = '';
    const unsubscribe = this.query.subscribe(v => {
      value = v;
    });
    unsubscribe();
    return value;
  }
}

/** 搜索状态管理实例 */
export const searchStore = new SearchStore();
