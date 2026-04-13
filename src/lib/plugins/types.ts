// 插件元数据（来自 plugin.json）
export interface PluginManifest {
  name: string;
  version: string;
  type: 'quickjs';
  logo?: string;
  prefix?: string;
  main?: string;           // 入口文件，默认 index.js
  description?: string;
  author?: string;
  patches?: string[];      // WASM 依赖列表
  features?: FeatureConfig[];
}

export interface FeatureConfig {
  code: string;
  label: string;
  type: 'list' | 'text' | 'cmd';
  items?: FeatureItem[];
}

export interface FeatureItem {
  label: string;
  action: string;
  icon?: string;
}

// 插件运行时状态
export type PluginState =
  | 'MetaLoaded'
  | 'Loading'
  | 'Ready'
  | 'Cached'
  | 'Unloaded'
  | 'Error';

// 插件实例
export interface PluginInstance {
  manifest: PluginManifest;
  state: PluginState;
  vmId?: string;
  pluginDir: string;
  loadedAt?: number;
  lastUsed?: number;
}

// 插件搜索结果
export interface PluginSearchResult {
  pluginId: string;
  title: string;
  description: string;
  icon?: string;
  action: string;
}

// ==================== 扩展 API 类型定义 ====================

// utools.fetch 请求选项
export interface FetchOptions {
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'HEAD';
  headers?: Record<string, string>;
  body?: string;
  timeout?: number;
  credentials?: 'omit' | 'same-origin' | 'include';
}

// utools.fetch 响应
export interface FetchResponse {
  status: number;
  statusText: string;
  ok: boolean;
  headers: Record<string, string>;
  text(): Promise<string>;
  json(): Promise<unknown>;
  blob?(): Promise<Blob>;
  arrayBuffer?(): Promise<ArrayBuffer>;
}

// utools.process 执行结果
export interface ProcessResult {
  stdout: string;
  stderr: string;
  exitCode: number;
  code: number;
}

// utools.process spawn 选项
export interface SpawnOptions {
  cwd?: string;
  env?: Record<string, string>;
  timeout?: number;
}

// utools.process 对象
export interface ProcessAPI {
  spawn(command: string, args: string[], options?: SpawnOptions): Promise<ProcessResult>;
  exec(command: string, options?: SpawnOptions): Promise<ProcessResult>;
  getNativeId(): string;
  getAppName(): string;
  getAppVersion(): string;
}

// utools.getContext 返回的上下文
export interface PluginContext {
  code: string;
  type: 'none' | 'img' | 'files' | 'text';
  payload: unknown;
  refresh: boolean;
}

// utools.dialog.showOpenDialog 选项
export interface OpenDialogOptions {
  title?: string;
  defaultPath?: string;
  filters?: DialogFilter[];
  properties?: ('openFile' | 'openDirectory' | 'multiSelections')[];
}

export interface DialogFilter {
  name: string;
  extensions: string[];
}

// utools.dialog.showSaveDialog 选项
export interface SaveDialogOptions {
  title?: string;
  defaultPath?: string;
  filters?: DialogFilter[];
}

// utools.dialog.showMessageBox 选项
export interface MessageBoxOptions {
  type?: 'none' | 'info' | 'error' | 'question' | 'warning';
  title?: string;
  message: string;
  detail?: string;
  buttons?: string[];
}

// utools.dialog 对象
export interface DialogAPI {
  showOpenDialog(options?: OpenDialogOptions): Promise<string | string[] | null>;
  showSaveDialog(options?: SaveDialogOptions): Promise<string | null>;
  showMessageBox(options?: MessageBoxOptions): Promise<string>;
}

// ==================== 全局 utools 对象类型 ====================

export interface UtoolsAPI {
  // ==================== 存储 ====================
  dbStorage: {
    getItem(key: string): string | null;
    setItem(key: string, value: string): void;
    removeItem(key: string): void;
    getAll(): Record<string, string>;
    clear?(): void;
  };

  // ==================== 剪贴板 ====================
  clipboard: {
    readText(): string;
    writeText(text: string): void;
    copyText(text: string): void;
    copyImage(base64: string): void;
    getClipboardImage(): string | null;
    setClipboardImage?(base64: string): void;
    copyFile?(path: string): void;
    getCopyedFiles?(): string[];
  };

  // ==================== Shell ====================
  shell: {
    openPath(path: string): void;
    openExternal(url: string): void;
    showItemInFolder(path: string): void;
    beep?(): void;
  };

  // ==================== 窗口控制 ====================
  hideMainWindow(): void;
  showMainWindow(): void;
  setExpendHeight?(height: number): void;
  outPlugin?(): void;

  // ==================== 路径 ====================
  getPath(name: PathName): string;

  // ==================== 通知 ====================
  showNotification(title: string, body: string): void;

  // ==================== 文件系统 ====================
  fs: {
    readTextFile(path: string): string;
    writeTextFile(path: string, content: string): void;
    exists(path: string): boolean;
    isDir(path: string): boolean;
  };

  // ==================== HTTP 请求 ====================
  fetch(url: string, options?: FetchOptions): Promise<FetchResponse>;

  // ==================== 对话框 ====================
  dialog: DialogAPI;

  // ==================== 子进程 ====================
  process: ProcessAPI;

  // ==================== 上下文 ====================
  getContext(): PluginContext;
  setContext(payload: unknown): void;

  // ==================== 图片处理 ====================
  getImagePath?(base64: string, name: string): string;

  // ==================== 生命周期回调 ====================
  onPluginReady?(callback: () => void): void;
  onPluginOut?(callback: () => void): void;
  registerPluginFeature?(feature: unknown): void;

  // ==================== 事件监听 ====================
  onPluginEnter?(callback: (action?: unknown) => void): void;
  onDbPull?(callback: (data: unknown) => void): void;
  onMainPush?(callback: (data: unknown, selectCallback?: (item: unknown) => void) => void): void;

  // ==================== WASM 桥接 ====================
  wasm: WasmAPI;
}

// 路径名称类型
export type PathName =
  | 'home' | '~'
  | 'desktop'
  | 'document' | 'documents'
  | 'download' | 'downloads'
  | 'music'
  | 'picture' | 'pictures' | 'photo' | 'photos'
  | 'video' | 'videos'
  | 'temp' | 'tmp'
  | 'appdata'
  | 'localappdata' | 'appcache'
  | 'userdata'
  | 'config'
  | 'log' | 'logs'
  | 'resource' | 'resources'
  | 'exe' | 'exepath'
  | 'plugin' | 'pluginpath'
  | 'root'
  | 'cwd' | 'currentdir';

// 声明全局 window.utools
declare global {
  interface Window {
    utools: UtoolsAPI;
  }
}

// ==================== WASM 相关类型 ====================

// WASM API（注入到 QuickJS VM 的 utools.wasm 对象）
export interface WasmAPI {
  /**
   * 发起 WASM 函数调用
   * @param funcName 函数名，格式 "patchName.methodName"（如 "crypto.sha256"）
   * @param argsJson 参数的 JSON 字符串
   * @returns 请求 ID，用于通过 __wasm_get_result 轮询获取结果
   */
  __wasm_call(funcName: string, argsJson: string): string;

  /**
   * 轮询获取 WASM 调用结果（非阻塞）
   * @param requestId __wasm_call 返回的请求 ID
   * @returns JSON 字符串（WasmCallResult），null 表示结果尚未就绪
   */
  __wasm_get_result(requestId: string): string | null;

  /**
   * 获取所有已注册的 WASM 函数名列表
   * @returns 函数名数组
   */
  __wasm_available(): string[];

  /**
   * 检查指定 WASM 函数是否可用
   * @param funcName 函数名
   * @returns 是否可用
   */
  __wasm_has(funcName: string): boolean;
}

// WASM 函数信息（注册到后端 WasmBridge）
export interface WasmFunctionInfo {
  name: string;
  patch: string;
  param_count: number;
}

// WASM patch 加载请求（来自 Rust wasm-load-patch 事件）
export interface WasmLoadPatchPayload {
  pluginId: string;
  patchName: string;
  patchDir: string;
  pkgDir: string;
  wasmFile: string;
  jsFile: string;
  exportedFunctions: string[];
}

// WASM 函数调用请求（来自 Rust wasm-call 事件）
export interface WasmCallPayload {
  requestId: string;
  function: string;
  args: string;
}

// WASM 函数调用结果（WebView 返回给 Rust）
export interface WasmCallResult {
  requestId: string;
  success: boolean;
  result?: string;
  error?: string;
}
