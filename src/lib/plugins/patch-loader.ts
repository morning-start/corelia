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

/** WASM 模块实例 */
interface WasmModuleInstance {
  [funcName: string]: (...args: any[]) => any;
}

/** 已加载的 patch 记录 */
interface LoadedPatch {
  patchName: string;
  pluginId: string;
  module: WasmModuleInstance;
  functions: string[];
}

// ==================== PatchLoader 核心实现 ====================

class PatchLoader {
  /** 已加载的 patch 模块 */
  private loadedPatches: Map<string, LoadedPatch> = new Map();

  /** 事件监听取消函数 */
  private unlisteners: UnlistenFn[] = [];

  /** 是否已初始化 */
  private initialized = false;

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

    try {
      // 将文件路径转换为 WebView 可访问的 URL
      const wasmUrl = convertFileSrc(wasmFile);
      const jsUrl = convertFileSrc(jsFile);

      console.log(`[PatchLoader] 📄 WASM URL: ${wasmUrl}`);
      console.log(`[PatchLoader] 📄 JS URL: ${jsUrl}`);

      // 动态导入 WASM 胶水 JS 模块
      const wasmModule = await this.loadWasmModule(patchName, jsUrl, wasmUrl);

      if (!wasmModule) {
        console.error(`[PatchLoader] ❌ 加载 WASM 模块失败: ${patchName}`);
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
      });

      // 注册函数到后端 WasmBridge
      await invoke('wasm_register_functions', {
        patch: patchName,
        functions,
      });

      console.log(`[PatchLoader] ✅ patch ${patchName} 函数已注册到后端`);

    } catch (e) {
      console.error(`[PatchLoader] ❌ 加载 patch ${patchName} 失败:`, e);
    }
  }

  /**
   * 动态加载 WASM 模块
   *
   * 使用动态 import() 加载胶水 JS，再初始化 WASM
   */
  private async loadWasmModule(
    patchName: string,
    jsUrl: string,
    wasmUrl: string
  ): Promise<WasmModuleInstance | null> {
    try {
      // 尝试动态导入胶水 JS
      // wasm-pack 生成的模块有 default 导出（异步初始化函数）
      const module = await import(/* @vite-ignore */ jsUrl);

      // 调用初始化函数加载 WASM
      if (typeof module.default === 'function') {
        // 新版 wasm-pack 输出格式：export default async function init()
        await module.default(wasmUrl);
        console.log(`[PatchLoader] ✅ WASM 模块初始化成功: ${patchName}`);
      } else if (typeof module.initSync === 'function') {
        // 同步初始化格式
        const response = await fetch(wasmUrl);
        const buffer = await response.arrayBuffer();
        module.initSync(new Uint8Array(buffer));
        console.log(`[PatchLoader] ✅ WASM 模块同步初始化成功: ${patchName}`);
      } else if (module.wasm !== undefined) {
        // 模块已经初始化
        console.log(`[PatchLoader] ✅ WASM 模块已预初始化: ${patchName}`);
      }

      return module as WasmModuleInstance;
    } catch (e) {
      console.error(`[PatchLoader] ❌ 动态导入 WASM 模块失败 (${patchName}):`, e);

      // 回退：尝试使用 fetch + WebAssembly API 直接加载
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
      console.log(`[PatchLoader] 🔄 尝试回退加载: ${patchName}`);

      const response = await fetch(wasmUrl);
      const bytes = await response.arrayBuffer();
      const { instance } = await WebAssembly.instantiate(bytes, {});

      // 将 WASM 导出包装为可调用函数
      const wrappedModule: WasmModuleInstance = {};
      const exports = instance.exports;

      for (const [name, exp] of Object.entries(exports)) {
        if (typeof exp === 'function') {
          // 直接暴露 WASM 导出函数
          wrappedModule[name] = (...args: any[]) => {
            try {
              return (exp as Function)(...args);
            } catch (e) {
              console.error(`[PatchLoader] WASM 函数 ${name} 执行失败:`, e);
              throw e;
            }
          };
        }
      }

      console.log(`[PatchLoader] ✅ WASM 回退加载成功: ${patchName}`);
      return wrappedModule;
    } catch (e) {
      console.error(`[PatchLoader] ❌ WASM 回退加载也失败 (${patchName}):`, e);
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

      // 查找已加载的模块
      const patch = this.loadedPatches.get(patchName);
      if (!patch) {
        throw new Error(`patch 未加载: ${patchName}`);
      }

      const func = patch.module[methodName];
      if (typeof func !== 'function') {
        throw new Error(`函数不存在: ${methodName} (patch: ${patchName})`);
      }

      // 解析参数
      let parsedArgs: any[];
      try {
        parsedArgs = JSON.parse(argsJson);
        if (!Array.isArray(parsedArgs)) {
          parsedArgs = [parsedArgs];
        }
      } catch {
        parsedArgs = [argsJson];
      }

      // 执行 WASM 函数
      const result = func(...parsedArgs);

      // 序列化结果并通过 invoke 存入后端 WasmBridge
      const resultEntry: WasmCallResult = {
        requestId,
        success: true,
        result: JSON.stringify(result ?? null),
      };

      await invoke('wasm_store_call_result', { result: resultEntry });

      console.log(`[PatchLoader] ✅ WASM 函数调用完成: ${funcName}`);

    } catch (e) {
      console.error(`[PatchLoader] ❌ WASM 函数调用失败: ${funcName}`, e);

      // 返回错误
      const resultEntry: WasmCallResult = {
        requestId,
        success: false,
        error: String(e),
      };

      await invoke('wasm_store_call_result', { result: resultEntry });
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
}

/** PatchLoader 单例 */
export const patchLoader = new PatchLoader();
export default patchLoader;
