/**
 * 插件服务层
 * 封装所有与后端插件系统的交互逻辑
 *
 * 功能：
 * - 插件生命周期管理（扫描、加载、卸载）
 * - 插件搜索（按前缀匹配）
 * - 插件执行（调用 onSearch/onAction）
 * - VM 管理（创建/销毁/复用）
 */

import { invoke } from '@tauri-apps/api/core';
import type { PluginManifest, PluginSearchResult } from './types';

/** 插件动作执行结果 */
export interface PluginActionResult {
  type: 'text' | 'error' | 'html' | 'copy';
  message?: string;
  data?: unknown;
  [key: string]: unknown;
}

/**
 * VM 缓存项
 */
interface VmCacheEntry {
  vmId: string;
  pluginId: string;
  createdAt: number;
  lastUsedAt: number;
}

/**
 * 插件服务层单例
 *
 * 使用说明：
 * ```typescript
 * import { pluginService } from '$lib/plugins/service';
 *
 * // 初始化插件系统
 * const plugins = await pluginService.init();
 *
 * // 搜索插件
 * const results = await pluginService.searchByPrefix('hw');
 *
 * // 执行插件搜索
 * const searchResults = await pluginService.executeSearch('hello-world', 'hello');
 *
 * // 执行插件动作
 * const actionResult = await pluginService.executeAction('hello-world', 'sayHello');
 * ```
 */
class PluginService {
  /** 是否已初始化 */
  private initialized = false;

  /** VM 缓存池（避免重复创建） */
  private vmCache: Map<string, VmCacheEntry> = new Map();

  /** 最大缓存 VM 数量 */
  private readonly MAX_VM_CACHE_SIZE = 10;

  /** 已加载的插件 ID 集合（用于跳过冗余 load 调用） */
  private loadedPlugins: Set<string> = new Set();

  /**
   * 确保插件已加载（带本地状态缓存，跳过冗余 IPC）
   *
   * @param id - 插件 ID
   * @returns 插件状态字符串
   */
  private async ensureLoaded(id: string): Promise<string> {
    if (this.loadedPlugins.has(id)) {
      console.log(`[PluginService] ♻️ 插件 ${id} 已加载，跳过 load`);
      return 'Ready';
    }
    const state = await this.load(id);
    if (state === 'Ready') {
      this.loadedPlugins.add(id);
    }
    return state;
  }

  /**
   * 初始化插件系统（应用启动时调用）
   *
   * 执行流程：
   * 1. 扫描 plugins 目录发现所有插件
   * 2. 输出日志显示发现的插件数量
   * 3. 标记为已初始化
   *
   * @returns 插件元数据列表
   */
  async init(): Promise<PluginManifest[]> {
    if (!this.initialized) {
      console.log('[PluginService] 🚀 初始化插件系统...');

      // 扫描插件目录
      const plugins = await this.scan();
      console.log(`[PluginService] ✅ 发现 ${plugins.length} 个插件:`);
      plugins.forEach(p => console.log(`  - ${p.name} (${p.version})`));

      this.initialized = true;
      return plugins;
    }

    return this.list();
  }

  /**
   * 扫描插件目录
   *
   * 调用后端 scan_plugins 命令，返回所有插件的元数据。
   * 此操作仅读取 plugin.json，不会加载 JS 代码。
   *
   * @returns 插件元数据列表
   */
  async scan(): Promise<PluginManifest[]> {
    return await invoke<PluginManifest[]>('scan_plugins');
  }

  /**
   * 获取已注册的插件列表
   *
   * @returns 插件元数据列表
   */
  async list(): Promise<PluginManifest[]> {
    return await invoke<PluginManifest[]>('get_plugin_list');
  }

  /**
   * 加载指定插件
   *
   * 执行流程：
   * 1. 调用后端 load_plugin 命令
   * 2. 后端会：创建 QuickJS VM → 注入 utools API → 执行 index.js
   * 3. 返回插件状态和 VM ID，前端将 vm_id 缓存以复用
   *
   * @param id - 插件 ID（即 manifest.name）
   * @returns 插件状态字符串
   */
  async load(id: string): Promise<string> {
    console.log(`[PluginService] 📦 加载插件: ${id}`);
    const result = await invoke<{ state: string; vm_id?: string }>('load_plugin', { id });

    // 缓存后端返回的 vm_id（如果有的话），避免前端再重复创建 VM
    if (result.vm_id && !this.vmCache.has(id)) {
      const now = Date.now();
      this.vmCache.set(id, {
        vmId: result.vm_id,
        pluginId: id,
        createdAt: now,
        lastUsedAt: now
      });
      console.log(`[PluginService] 📌 已缓存后端创建的 VM: ${result.vm_id} (插件: ${id})`);
    }

    console.log(`[PluginService] ✅ 插件 ${id} 状态: ${result.state}`);
    return result.state;
  }

  /**
   * 卸载指定插件
   *
   * 会销毁关联的 QuickJS VM，释放内存资源。
   * 同时清理本地 VM 缓存。
   *
   * @param id - 插件 ID
   */
  async unload(id: string): Promise<void> {
    console.log(`[PluginService] 🗑️ 卸载插件: ${id}`);

    // 清理 VM 缓存
    this.clearVmCache(id);

    // 清理本地已加载状态
    this.loadedPlugins.delete(id);

    await invoke('unload_plugin', { id });
    console.log(`[PluginService] ✅ 插件 ${id} 已卸载`);
  }

  /**
   * 根据前缀搜索匹配的插件
   *
   * 支持双向模糊匹配：
   * - 输入 "h" 可以匹配到 prefix 为 "hw" 的插件
   * - 输入 "hw" 也可以匹配到 prefix 为 "hw" 的插件
   *
   * @param prefix - 搜索前缀
   * @returns 匹配的插件列表
   */
  async searchByPrefix(prefix: string): Promise<PluginManifest[]> {
    return await invoke<PluginManifest[]>('find_plugins_by_prefix', { prefix });
  }

  /**
   * 在指定插件中执行搜索
   *
   * 执行流程：
   * 1. 确保插件已加载（带本地缓存，跳过冗余 IPC）
   * 2. 获取或创建该插件的 VM
   * 3. 在 VM 中调用 onSearch(query) 函数
   * 4. 解析返回的 JSON 结果
   *
   * @param pluginId - 插件 ID
   * @param query - 用户输入的搜索词
   * @returns 插件搜索结果列表
   */
  async executeSearch(pluginId: string, query: string): Promise<PluginSearchResult[]> {
    console.log(`[PluginService] 🔍 执行插件搜索: ${pluginId}, query="${query}"`);

    try {
      const result = await this.executeInVm(
        pluginId,
        'onSearch',
        query,
        [],
        'search'
      );

      // 如果返回的是错误对象，抛出异常
      const raw = result as unknown as Record<string, unknown>;
      if (raw.error) {
        throw new Error(String(raw.error));
      }

      console.log(`[PluginService] ✅ 插件 ${pluginId} 返回 ${(result as unknown[]).length} 个结果`);
      return result as PluginSearchResult[];

    } catch (e) {
      console.error(`[PluginService] ❌ 插件 ${pluginId} 搜索失败:`, e);
      return [];
    }
  }

  /**
   * 执行插件动作
   *
   * @param pluginId - 插件 ID
   * @param action - 动作名称（如 "sayHello"、"testStorage"）
   * @returns 动作执行结果
   */
  async executeAction(pluginId: string, action: string, data?: unknown): Promise<PluginActionResult> {
    console.log(`[PluginService] ⚡ 执行插件动作: ${pluginId}.${action}`);

    try {
      const defaultResult: PluginActionResult = { type: 'error', message: '无返回值' };
      const result = await this.executeInVm<PluginActionResult>(
        pluginId,
        'onAction',
        JSON.stringify({ action, data }),
        defaultResult,
        'action'
      );

      console.log(`[PluginService] ✅ 插件 ${pluginId}.${action} 执行完成:`, (result as unknown as Record<string, unknown>)?.type ?? 'unknown');
      return result;

    } catch (e) {
      console.error(`[PluginService] ❌ 插件 ${pluginId}.${action} 执行失败:`, e);
      return {
        type: 'error',
        message: `执行失败: ${e}`
      };
    }
  }

  /**
   * 在插件 VM 中安全地调用指定函数（统一执行入口）
   *
   * 安全性：参数通过 JSON.stringify/JSON.parse 双向序列化传入，
   * 彻底防止 JS 代码注入攻击。
   *
   * @param pluginId - 插件 ID
   * @param fnName - 要调用的函数名（如 "onSearch"、"onAction"）
   * @param arg - 传递给函数的参数（将被 JSON 安全编码）
   * @param defaultResult - 函数不存在时的默认返回值
   * @param kind - 调用类型标识（用于日志，如 "search"/"action"）
   * @returns 反序列化后的函数返回值
   */
  private async executeInVm<T>(
    pluginId: string,
    fnName: string,
    arg: string,
    defaultResult: T,
    kind: string
  ): Promise<T> {
    await this.ensureLoaded(pluginId);

    const vmId = await this.getOrCreateVm(pluginId);

    // 使用 JSON.stringify 安全编码参数，在 VM 内部通过 JSON.parse 解码
    // 这样无论 arg 包含什么特殊字符都不会导致代码注入
    const safeArg = JSON.stringify(arg);
    const safeDefault = JSON.stringify(defaultResult);
    const code = `
      (function() {
        try {
          if (typeof ${fnName} === 'function') {
            var result = ${fnName}(JSON.parse(${safeArg}));
            return JSON.stringify(result != null ? result : ${safeDefault});
          } else {
            console.warn('[${pluginId}] ${fnName} 函数不存在');
            return JSON.stringify(${safeDefault});
          }
        } catch (e) {
          console.error('[${pluginId}] ${fnName} 执行错误:', e);
          return JSON.stringify({ error: e.message || String(e), _kind: '${kind}' });
        }
      })()
    `;

    const rawResult = await invoke<string>('quickjs_execute', { vmId, code });
    return JSON.parse(rawResult);
  }

  /**
   * 获取或创建插件的 VM（带缓存机制）
   *
   * 缓存策略：
   * - 首次调用时创建新 VM 并缓存
   * - 后续调用直接使用缓存的 VM
   * - 当缓存超过上限时，清除最久未使用的 VM
   *
   * @param pluginId - 插件 ID
   * @returns VM ID
   */
  private async getOrCreateVm(pluginId: string): Promise<string> {
    // 检查缓存中是否已有该插件的 VM
    const cached = this.vmCache.get(pluginId);
    if (cached) {
      // 更新最后使用时间
      cached.lastUsedAt = Date.now();
      console.log(`[PluginService] ♻️ 复用缓存的 VM: ${cached.vmId} (插件: ${pluginId})`);
      return cached.vmId;
    }

    // 缓存未命中，检查是否需要清理
    if (this.vmCache.size >= this.MAX_VM_CACHE_SIZE) {
      this.evictOldestVm();
    }

    // 创建新的 VM
    const vmId = await invoke<string>('quickjs_create_vm');

    // 注入 utools API 到新 VM
    try {
      await invoke('inject_apis_to_vm', { vmId, pluginId });
      console.log(`[PluginService] ✅ 创建新 VM 并注入 API: ${vmId} (插件: ${pluginId})`);
    } catch (e) {
      console.warn(`[PluginService] ⚠️ API 注入失败（可能不影响基本功能）:`, e);
    }

    // 加入缓存
    const now = Date.now();
    this.vmCache.set(pluginId, {
      vmId,
      pluginId,
      createdAt: now,
      lastUsedAt: now
    });

    return vmId;
  }

  /**
   * 清除最久未使用的 VM（LRU 淘汰策略）
   */
  private evictOldestVm(): void {
    let oldestKey: string | null = null;
    let oldestTime = Infinity;

    for (const [key, entry] of this.vmCache.entries()) {
      if (entry.lastUsedAt < oldestTime) {
        oldestTime = entry.lastUsedAt;
        oldestKey = key;
      }
    }

    if (oldestKey) {
      const entry = this.vmCache.get(oldestKey)!;
      console.log(`[PluginService] 🗑️ 淘汰最久未使用的 VM: ${entry.vmId} (插件: ${oldestKey})`);

      // 尝试销毁后端 VM
      invoke('quickjs_destroy_vm', { vmId: entry.vmId }).catch(e => {
        console.warn(`[PluginService] ⚠️ 销毁 VM 失败:`, e);
      });

      // 从缓存中移除
      this.vmCache.delete(oldestKey);
    }
  }

  /**
   * 清除指定插件的 VM 缓存
   *
   * @param pluginId - 插件 ID
   */
  private clearVmCache(pluginId: string): void {
    const cached = this.vmCache.get(pluginId);
    if (cached) {
      // 尝试销毁后端 VM
      invoke('quickjs_destroy_vm', { vmId: cached.vmId }).catch(e => {
        console.warn(`[PluginService] ⚠️ 销毁 VM 失败:`, e);
      });

      this.vmCache.delete(pluginId);
      console.log(`[PluginService] 🧹 已清除插件 ${pluginId} 的 VM 缓存`);
    }
  }

  /**
   * 清除所有 VM 缓存（应用退出时调用）
   */
  clearAllVmCache(): void {
    console.log(`[PluginService] 🧹 清除所有 VM 缓存 (共 ${this.vmCache.size} 个)...`);

    for (const [pluginId, entry] of this.vmCache.entries()) {
      invoke('quickjs_destroy_vm', { vmId: entry.vmId }).catch(e => {
        console.warn(`[PluginService] ⚠️ 销毁 VM 失败 (${pluginId}):`, e);
      });
    }

    this.vmCache.clear();
    console.log('[PluginService] ✅ 所有 VM 缓存已清除');
  }

  /**
   * 获取当前缓存状态（用于调试）
   *
   * @returns 缓存的 VM 数量和详情
   */
  getCacheStatus(): { size: number; entries: Array<{ pluginId: string; vmId: string; lastUsed: number }> } {
    return {
      size: this.vmCache.size,
      entries: Array.from(this.vmCache.entries()).map(([key, value]) => ({
        pluginId: key,
        vmId: value.vmId,
        lastUsed: value.lastUsedAt
      }))
    };
  }
}

/** 插件服务层单例实例 */
export const pluginService = new PluginService();
export default pluginService;
