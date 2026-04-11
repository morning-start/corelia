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
