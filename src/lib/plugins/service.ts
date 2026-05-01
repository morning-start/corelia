/**
 * 插件服务层
 * 封装所有与后端插件系统的交互逻辑
 *
 * 功能：
 * - 插件生命周期管理（扫描、加载、卸载）
 * - 插件搜索（按前缀匹配）
 * - 插件执行（调用 onSearch/onAction）
 *
 * 注意：VM 生命周期完全由后端管理，前端不做任何 VM 缓存，
 * 避免前后端缓存不一致导致的问题。
 */

import { invoke } from '@tauri-apps/api/core';
import { toast } from '$lib/components/Toast.svelte';
import type { PluginManifest, PluginSearchResult } from './types';

/** 插件动作执行结果 */
export interface PluginActionResult {
  type: 'text' | 'error' | 'html' | 'copy';
  message?: string;
  data?: unknown;
  [key: string]: unknown;
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
    try {
      const result = await invoke<{ state: string; vm_id?: string }>('load_plugin', { id });

      console.log(`[PluginService] ✅ 插件 ${id} 状态: ${result.state}`);
      if (result.state === 'ready' || result.state === 'cached') {
        toast.success(`插件 "${id}" 加载成功`);
      }
      return result.state;
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      console.error(`[PluginService] ❌ 插件 ${id} 加载失败:`, error);
      toast.error(`插件 "${id}" 加载失败: ${errorMsg}`);
      throw error;
    }
  }

  /**
   * 卸载指定插件
   *
   * 会销毁关联的 QuickJS VM，释放内存资源。
   *
   * @param id - 插件 ID
   */
  async unload(id: string): Promise<void> {
    console.log(`[PluginService] 🗑️ 卸载插件: ${id}`);
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
   * 1. 确保插件已加载
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
   * VM 由后端统一管理，前端仅通过后端命令执行代码。
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
    await this.load(pluginId);

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

    const rawResult = await invoke<string>('plugin_execute', { pluginId, code });
    return JSON.parse(rawResult);
  }

  /**
   * 获取插件健康状态
   *
   * @returns 活跃 VM 数量和空 entries（前端不再缓存 VM）
   */
  async getCacheStatus(): Promise<{ size: number; entries: Array<{ pluginId: string; vmId: string; lastUsed: number }> }> {
    return {
      size: 0,
      entries: []
    };
  }
}

/** 插件服务层单例实例 */
export const pluginService = new PluginService();
export default pluginService;
