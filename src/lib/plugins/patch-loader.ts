/**
 * WASM Patch 加载器
 *
 * 在 WebView 侧加载 WASM 模块，并将导出函数注册到后端 WasmBridge。
 *
 * 架构：
 * Rust (loader.rs) ──emit "wasm-load-patch"──→ WebView (PatchLoader)
 *                                                      │
 *                                                      ├── 初始化 WebAssembly
 *                                                      ├── 读取导出函数签名
 *                                                      └── invoke "wasm_register_functions" → Rust
 *
 * 调用流程：
 * QuickJS VM ──__wasm_call──→ Rust ──emit "wasm-call"──→ WebView ──执行 WASM 函数──→ 返回结果
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { convertFileSrc } from '@tauri-apps/api/core';

import type {
  WasmLoadPatchPayload,
  WasmFunctionInfo,
  WasmCallResult,
} from './types';

// ==================== 内部类型 ====================

/** WASM 模块实例（导出的函数集合） */
interface WasmModuleInstance {
  [funcName: string]: (...args: unknown[]) => unknown;
}

/** 已加载的 patch 记录 */
interface LoadedPatch {
  patchName: string;
  pluginId: string;
  module: WasmModuleInstance;
  functions: string[];
  loadTime: number;
}

/** 加载状态记录 */
interface LoadState {
  status: 'loading' | 'ready' | 'failed';
  retryCount: number;
  lastError?: string;
}

// ==================== PatchLoader 核心实现 ====================

class PatchLoader {
  /** 已加载的 patch 模块 */
  private loadedPatches: Map<string, LoadedPatch> = new Map();

  /** 加载状态跟踪 */
  private loadStates: Map<string, LoadState> = new Map();

  /** 事件监听取消函数 */
  private unlisteners: UnlistenFn[] = [];

  /** 是否已初始化 */
  private initialized = false;

  /** 最大重试次数 */
  private readonly MAX_RETRY_COUNT = 3;

  /**
   * 初始化 PatchLoader，注册事件监听
   */
  async init(): Promise<void> {
    if (this.initialized) {
      console.log('[PatchLoader] 已初始化，跳过');
      return;
    }

    console.log('[PatchLoader] 🚀 初始化...');

    // 监听 Rust 发出的 wasm-load-patch 事件
    const unlistenLoad = await listen<WasmLoadPatchPayload>('wasm-load-patch', (event) => {
      this.handleLoadPatch(event.payload);
    });
    this.unlisteners.push(unlistenLoad);

    // 监听 Rust 发出的 wasm-call 事件（QuickJS VM 调用 WASM 函数）
    const unlistenCall = await listen<{ requestId: string; function: string; args: string }>(
      'wasm-call',
      (event) => {
        this.handleWasmCall(event.payload);
      }
    );
    this.unlisteners.push(unlistenCall);

    this.initialized = true;
    console.log('[PatchLoader] ✅ 初始化完成，已注册事件监听');
  }

  /**
   * 处理 Rust 发来的 WASM patch 加载请求
   */
  private async handleLoadPatch(payload: WasmLoadPatchPayload): Promise<void> {
    const { pluginId, patchName, pkgDir, wasmFile, jsFile } = payload;

    console.log(`[PatchLoader] 📦 加载 WASM patch: ${patchName} (插件: ${pluginId})`);

    // 检查是否已加载
    if (this.loadedPatches.has(patchName)) {
      console.log(`[PatchLoader] ♻️ patch ${patchName} 已加载，跳过`);
      return;
    }

    // 检查加载状态，避免重复加载
    const existingState = this.loadStates.get(patchName);
    if (existingState?.status === 'loading') {
      console.log(`[PatchLoader] ⏳ patch ${patchName} 正在加载中，等待完成`);
      return;
    }

    // 设置加载状态
    this.loadStates.set(patchName, {
      status: 'loading',
      retryCount: existingState?.retryCount || 0
    });

    try {
      // 将文件路径转换为 WebView 可访问的 URL
      const wasmUrl = convertFileSrc(wasmFile);
      const jsUrl = convertFileSrc(jsFile);

      console.log(`[PatchLoader] 📄 WASM URL: ${wasmUrl}`);
      console.log(`[PatchLoader] 📄 JS URL: ${jsUrl}`);

      // 动态导入 WASM 胶水 JS 模块
      const wasmModule = await this.loadWasmModule(patchName, jsUrl, wasmUrl);

      if (!wasmModule) {
        // 所有加载策略均失败，上报降级状态
        const errorMsg = '所有 WASM 加载策略均失败';
        console.error(`[PatchLoader] ❌ ${errorMsg}: ${patchName}`);
        await this.handleLoadFailure(patchName, errorMsg);
        return;
      }

      // 收集导出的函数
      const functions = this.extractFunctions(wasmModule, patchName);

      console.log(`[PatchLoader] ✅ patch ${patchName} 加载成功，${functions.length} 个函数:`,
        functions.map(f => f.name));

      // 缓存已加载的模块
      this.loadedPatches.set(patchName, {
        patchName,
        pluginId,
        module: wasmModule,
        functions: functions.map(f => f.name),
        loadTime: Date.now()
      });

      // 更新加载状态为成功
      this.loadStates.set(patchName, {
        status: 'ready',
        retryCount: 0
      });

      // 注册函数到后端 WasmBridge
      await invoke('wasm_register_functions', {
        patch: patchName,
        functions,
      });

      console.log(`[PatchLoader] ✅ patch ${patchName} 函数已注册到后端`);

    } catch (e) {
      console.error(`[PatchLoader] ❌ 加载 patch ${patchName} 失败:`, e);
      await this.handleLoadFailure(patchName, String(e));
    }
  }

  /**
   * 处理加载失败，增加重试计数和降级策略
   */
  private async handleLoadFailure(patchName: string, error: string): Promise<void> {
    const currentState = this.loadStates.get(patchName) || {
      status: 'failed',
      retryCount: 0
    };

    const newRetryCount = currentState.retryCount + 1;

    this.loadStates.set(patchName, {
      status: 'failed',
      retryCount: newRetryCount,
      lastError: error
    });

    // 如果还可以重试，记录日志
    if (newRetryCount < this.MAX_RETRY_COUNT) {
      console.warn(`[PatchLoader] ⚠️ patch ${patchName} 加载失败 (${newRetryCount}/${this.MAX_RETRY_COUNT})`);
    } else {
      console.error(`[PatchLoader] 🚨 patch ${patchName} 已达最大重试次数`);
    }

    // 上报错误
    await this.reportPatchError(patchName, error);
  }

  /**
   * 上报 patch 加载错误到后端（用于监控和降级通知）
   */
  private async reportPatchError(patchName: string, error: string): Promise<void> {
    console.warn(`[PatchLoader] ⚠️ 上报 patch 加载错误: ${patchName} - ${error}`);
    try {
      // 通过事件通知后端 patch 加载失败，而非抛出异常
      // 这样 Rust 侧可以选择继续加载插件（降级模式）或提示用户
      await invoke('wasm_store_call_result', {
        result: {
          requestId: `patch_error_${patchName}_${Date.now()}`,
          success: false,
          error: `[PatchLoader] patch ${patchName} 加载失败: ${error}`,
        },
      });
    } catch (reportErr) {
      console.error(`[PatchLoader] ❌ 上报 patch 错误也失败了:`, reportErr);
    }
  }

  /**
   * 动态加载 WASM 模块
   *
   * 使用动态 import() 加载胶水 JS，再初始化 WASM。
   * 失败时自动降级到 fetch + WebAssembly API 直接加载。
   */
  private async loadWasmModule(
    patchName: string,
    jsUrl: string,
    wasmUrl: string
  ): Promise<WasmModuleInstance | null> {
    try {
      // 策略 1：尝试动态导入胶水 JS
      // wasm-pack 生成的模块有 default 导出（异步初始化函数）
      const module = await import(/* @vite-ignore */ jsUrl);

      // 调用初始化函数加载 WASM
      if (typeof module.default === 'function') {
        // 新版 wasm-pack 输出格式：export default async function init()
        try {
          await module.default(wasmUrl);
          console.log(`[PatchLoader] ✅ WASM 模块初始化成功 (策略 1): ${patchName}`);
        } catch (initErr) {
          console.warn(`[PatchLoader] ⚠️ WASM 初始化失败 (${patchName})，尝试策略 2:`, initErr);
          throw initErr; // 抛出后由外层 catch 进入回退策略
        }
      } else if (typeof module.initSync === 'function') {
        // 同步初始化格式
        try {
          const response = await fetch(wasmUrl);
          if (!response.ok) {
            throw new Error(`fetch WASM 失败: ${response.status}`);
          }
          const buffer = await response.arrayBuffer();
          module.initSync(new Uint8Array(buffer));
          console.log(`[PatchLoader] ✅ WASM 模块同步初始化成功 (策略 1b): ${patchName}`);
        } catch (initErr) {
          console.warn(`[PatchLoader] ⚠️ WASM 同步初始化失败 (${patchName})，尝试策略 2:`, initErr);
          throw initErr;
        }
      } else if (module.wasm !== undefined) {
        // 模块已经初始化
        console.log(`[PatchLoader] ✅ WASM 模块已预初始化 (策略 1c): ${patchName}`);
      }

      return module as WasmModuleInstance;
    } catch (e) {
      console.error(`[PatchLoader] ❌ 策略 1 失败 (${patchName})，启用策略 2:`, e);

      // 策略 2：尝试使用 fetch + WebAssembly API 直接加载
      return this.loadWasmFallback(patchName, wasmUrl);
    }
  }

  /**
   * 回退方案：直接使用 WebAssembly API 加载 WASM
   *
   * 如果胶水 JS 无法通过 import() 加载（CORS、路径问题等），
   * 直接 fetch WASM 二进制并编译
   */
  private async loadWasmFallback(
    patchName: string,
    wasmUrl: string
  ): Promise<WasmModuleInstance | null> {
    try {
      console.log(`[PatchLoader] 🔄 尝试策略 2 (回退): ${patchName}`);

      const response = await fetch(wasmUrl);
      if (!response.ok) {
        console.error(`[PatchLoader] ❌ fetch WASM 失败: ${response.status} ${response.statusText}`);
        return null;
      }

      const bytes = await response.arrayBuffer();

      // 先尝试编译模块以捕获格式错误
      let wasmModule: WebAssembly.Module;
      try {
        wasmModule = await WebAssembly.compile(bytes);
      } catch (compileErr) {
        console.error(`[PatchLoader] ❌ WASM 编译失败:`, compileErr);
        return null;
      }

      // 实例化模块
      let instance: WebAssembly.Instance;
      try {
        instance = await WebAssembly.instantiate(wasmModule, {});
      } catch (instantiateErr) {
        console.error(`[PatchLoader] ❌ WASM 实例化失败:`, instantiateErr);
        return null;
      }

      // 将 WASM 导出包装为可调用函数
      const wrappedModule: WasmModuleInstance = {};
      const exports = instance.exports;

      for (const [name, exp] of Object.entries(exports)) {
        if (typeof exp === 'function') {
          // 直接暴露 WASM 导出函数（带运行时错误隔离）
          wrappedModule[name] = (...args: unknown[]) => {
            try {
              return (exp as Function)(...args);
            } catch (e) {
              console.error(`[PatchLoader] WASM 函数 ${name} 执行失败:`, e);
              // 返回 null 而非抛出，避免破坏整个调用链
              return null;
            }
          };
        }
      }

      console.log(`[PatchLoader] ✅ WASM 策略 2 加载成功: ${patchName}`);
      return wrappedModule;
    } catch (e) {
      console.error(`[PatchLoader] ❌ WASM 策略 2 也失败 (${patchName}):`, e);
      return null;
    }
  }

  /**
   * 从 WASM 模块中提取可调用函数列表
   */
  private extractFunctions(
    module: WasmModuleInstance,
    patchName: string
  ): WasmFunctionInfo[] {
    const functions: WasmFunctionInfo[] = [];

    for (const [name, value] of Object.entries(module)) {
      // 只导出函数，跳过内部属性和初始化函数
      if (typeof value !== 'function') continue;
      if (name.startsWith('__')) continue;  // 跳过内部函数
      if (name === 'default') continue;     // 跳过模块默认导出
      if (name === 'initSync') continue;    // 跳过初始化函数

      // 推断参数数量（不精确，但足够用）
      const paramCount = value.length || 0;

      functions.push({
        name: `${patchName}.${name}`,
        patch: patchName,
        param_count: paramCount,
      });
    }

    return functions;
  }

  /**
   * 处理 QuickJS VM 发来的 WASM 函数调用请求
   */
  private async handleWasmCall(
    payload: { requestId: string; function: string; args: string }
  ): Promise<void> {
    const { requestId, function: funcName, args: argsJson } = payload;

    console.log(`[PatchLoader] 📞 WASM 函数调用: ${funcName}`);

    try {
      // 解析函数名: "patchName.funcName"
      const [patchName, methodName] = funcName.split('.');

      if (!patchName || !methodName) {
        throw new Error(`无效的函数名格式: ${funcName}`);
      }

      // 查找已加载的模块（如果未加载，尝试降级处理）
      const patch = this.loadedPatches.get(patchName);
      if (!patch) {
        console.warn(`[PatchLoader] ⚠️ patch 未加载: ${patchName}，返回降级结果`);
        await this.storeErrorResult(requestId, `patch 未加载: ${patchName}`);
        return;
      }

      const func = patch.module[methodName];
      if (typeof func !== 'function') {
        console.warn(`[PatchLoader] ⚠️ 函数不存在: ${methodName} (patch: ${patchName})`);
        await this.storeErrorResult(requestId, `函数不存在: ${methodName}`);
        return;
      }

      // 解析参数（宽容解析，非数组时包装为数组）
      let parsedArgs: unknown[];
      try {
        parsedArgs = JSON.parse(argsJson);
        if (!Array.isArray(parsedArgs)) {
          parsedArgs = [parsedArgs];
        }
      } catch {
        parsedArgs = [argsJson];
      }

      // 执行 WASM 函数（带运行时错误隔离）
      let result: unknown;
      try {
        result = func(...parsedArgs);
      } catch (execErr) {
        console.error(`[PatchLoader] ❌ WASM 函数执行异常: ${funcName}`, execErr);
        await this.storeErrorResult(requestId, `执行异常: ${execErr}`);
        return;
      }

      // 序列化结果并通过 invoke 存入后端 WasmBridge
      let resultStr: string;
      try {
        resultStr = JSON.stringify(result ?? null);
      } catch (serializeErr) {
        console.error(`[PatchLoader] ❌ 结果序列化失败:`, serializeErr);
        await this.storeErrorResult(requestId, `结果序列化失败: ${serializeErr}`);
        return;
      }

      const resultEntry: WasmCallResult = {
        requestId,
        success: true,
        result: resultStr,
      };

      await invoke('wasm_store_call_result', { result: resultEntry });

      console.log(`[PatchLoader] ✅ WASM 函数调用完成: ${funcName}`);

    } catch (e) {
      console.error(`[PatchLoader] ❌ WASM 函数调用未处理异常: ${funcName}`, e);
      await this.storeErrorResult(requestId, String(e));
    }
  }

  /**
   * 存储错误结果到后端 WasmBridge（错误隔离辅助方法）
   */
  private async storeErrorResult(requestId: string, error: string): Promise<void> {
    try {
      await invoke('wasm_store_call_result', {
        result: {
          requestId,
          success: false,
          error,
        },
      });
    } catch (storeErr) {
      console.error(`[PatchLoader] ❌ 存储错误结果也失败了:`, storeErr);
    }
  }

  /**
   * 卸载指定 patch
   */
  async unloadPatch(patchName: string): Promise<void> {
    const patch = this.loadedPatches.get(patchName);
    if (!patch) {
      console.warn(`[PatchLoader] ⚠️ patch 未加载: ${patchName}`);
      return;
    }

    // 从缓存中移除
    this.loadedPatches.delete(patchName);
    // 清除加载状态
    this.loadStates.delete(patchName);

    // 通知后端注销
    try {
      await invoke('wasm_unregister_patch', { patch: patchName });
    } catch (e) {
      console.warn(`[PatchLoader] ⚠️ 后端注销 patch 失败:`, e);
    }

    console.log(`[PatchLoader] 🗑️ patch ${patchName} 已卸载`);
  }

  /**
   * 销毁 PatchLoader，清理所有资源
   */
  async destroy(): Promise<void> {
    // 取消所有事件监听
    for (const unlisten of this.unlisteners) {
      unlisten();
    }
    this.unlisteners = [];

    // 注销所有 patch
    for (const patchName of this.loadedPatches.keys()) {
      await this.unloadPatch(patchName);
    }

    this.initialized = false;
    console.log('[PatchLoader] 🧹 已销毁');
  }

  /**
   * 获取已加载的 patch 列表（调试用）
   */
  getLoadedPatches(): string[] {
    return Array.from(this.loadedPatches.keys());
  }

  /**
   * 获取指定 patch 的函数列表（调试用）
   */
  getPatchFunctions(patchName: string): string[] {
    return this.loadedPatches.get(patchName)?.functions ?? [];
  }

  /**
   * 获取 patch 加载状态（调试用）
   */
  getPatchState(patchName: string): LoadState | undefined {
    return this.loadStates.get(patchName);
  }
}

/** PatchLoader 单例 */
export const patchLoader = new PatchLoader();
export default patchLoader;
