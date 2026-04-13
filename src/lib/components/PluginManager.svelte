<script lang="ts">
  import { onMount } from 'svelte';
  import { pluginService } from '$lib/plugins/service';
  import type { PluginManifest } from '$lib/plugins/types';

  interface Props {
    /** 关闭面板回调函数 */
    onClose?: () => void;
  }

  let { onClose }: Props = $props();

  /** 插件列表 */
  let plugins = $state<PluginManifest[]>([]);
  /** 加载状态 */
  let loading = $state(true);
  /** 错误信息 */
  let error = $state<string | null>(null);
  /** 启用的插件 ID 集合 */
  let enabledPlugins = $state<Set<string>>(new Set());
  /** 正在操作的插件 ID */
  let operatingPlugin = $state<string | null>(null);

  onMount(async () => {
    await loadPlugins();
    await loadEnabledPlugins();
  });

  /** 加载插件列表 */
  async function loadPlugins() {
    loading = true;
    error = null;
    try {
      plugins = await pluginService.init();
      console.log('[PluginManager] 加载插件列表:', plugins);
    } catch (e) {
      error = `加载插件失败: ${e}`;
      console.error('[PluginManager]', error);
    } finally {
      loading = false;
    }
  }

  /** 从本地存储加载启用的插件列表 */
  async function loadEnabledPlugins() {
    try {
      const stored = localStorage.getItem('enabled_plugins');
      if (stored) {
        enabledPlugins = new Set(JSON.parse(stored));
      } else {
        // 默认全部启用
        enabledPlugins = new Set(plugins.map(p => p.name));
        saveEnabledPlugins();
      }
    } catch (e) {
      console.error('[PluginManager] 加载启用状态失败:', e);
    }
  }

  /** 保存启用的插件列表到本地存储 */
  function saveEnabledPlugins() {
    localStorage.setItem('enabled_plugins', JSON.stringify([...enabledPlugins]));
  }

  /** 切换插件启用状态 */
  async function togglePlugin(plugin: PluginManifest) {
    operatingPlugin = plugin.name;
    try {
      if (enabledPlugins.has(plugin.name)) {
        // 禁用：先卸载
        await pluginService.unload(plugin.name);
        enabledPlugins.delete(plugin.name);
      } else {
        // 启用：加载插件
        await pluginService.load(plugin.name);
        enabledPlugins.add(plugin.name);
      }
      enabledPlugins = new Set(enabledPlugins);
      saveEnabledPlugins();
      console.log(`[PluginManager] 插件 ${plugin.name} 已${enabledPlugins.has(plugin.name) ? '启用' : '禁用'}`);
    } catch (e) {
      console.error(`[PluginManager] 切换插件状态失败 (${plugin.name}):`, e);
      error = `操作失败: ${e}`;
    } finally {
      operatingPlugin = null;
    }
  }

  /** 重新加载插件 */
  async function reloadPlugin(plugin: PluginManifest) {
    operatingPlugin = plugin.name;
    try {
      await pluginService.unload(plugin.name);
      await pluginService.load(plugin.name);
      console.log(`[PluginManager] 插件 ${plugin.name} 已重新加载`);
    } catch (e) {
      console.error(`[PluginManager] 重新加载插件失败 (${plugin.name}):`, e);
      error = `重新加载失败: ${e}`;
    } finally {
      operatingPlugin = null;
    }
  }

  /** 获取插件状态图标 */
  function getPluginStatus(plugin: PluginManifest): { icon: string; color: string; label: string } {
    const isEnabled = enabledPlugins.has(plugin.name);
    const isOperating = operatingPlugin === plugin.name;

    if (isOperating) {
      return { icon: '⟳', color: 'var(--accent-color)', label: '操作中...' };
    }
    if (isEnabled) {
      return { icon: '●', color: '#22c55e', label: '已启用' };
    }
    return { icon: '○', color: 'var(--text-secondary)', label: '已禁用' };
  }

  /** 格式化前缀显示 */
  function formatPrefix(prefix: string | undefined): string {
    return prefix ? prefix : '—';
  }
</script>

<div class="plugin-manager">
  <div class="plugin-header">
    <h3>插件管理</h3>
    <span class="plugin-count">{plugins.length} 个插件</span>
  </div>

  {#if error}
    <div class="error-message">
      <span class="error-icon">⚠️</span>
      <span>{error}</span>
      <button class="error-dismiss" onclick={() => error = null}>×</button>
    </div>
  {/if}

  {#if loading}
    <div class="loading-state">
      <span class="loading-spinner">⟳</span>
      <span>正在加载插件...</span>
    </div>
  {:else if plugins.length === 0}
    <div class="empty-state">
      <span class="empty-icon">📦</span>
      <p>暂未安装任何插件</p>
      <p class="empty-hint">将插件放置到 plugins 目录即可自动加载</p>
    </div>
  {:else}
    <div class="plugin-list">
      {#each plugins as plugin}
        {@const status = getPluginStatus(plugin)}
        <div class="plugin-item" class:disabled={!enabledPlugins.has(plugin.name)}>
          <div class="plugin-info">
            <div class="plugin-logo">
              {#if plugin.logo}
                <img src={plugin.logo} alt={plugin.name} />
              {:else}
                <span class="plugin-logo-placeholder">📦</span>
              {/if}
            </div>
            <div class="plugin-details">
              <div class="plugin-name">
                {plugin.name}
                <span class="plugin-version">v{plugin.version}</span>
              </div>
              <div class="plugin-meta">
                <span class="plugin-prefix">触发: <code>{formatPrefix(plugin.prefix)}</code></span>
                {#if plugin.author}
                  <span class="plugin-author">by {plugin.author}</span>
                {/if}
              </div>
              {#if plugin.description}
                <div class="plugin-description">{plugin.description}</div>
              {/if}
            </div>
          </div>

          <div class="plugin-actions">
            <div class="plugin-status" style="color: {status.color}">
              <span class="status-icon">{status.icon}</span>
              <span class="status-label">{status.label}</span>
            </div>

            <div class="action-buttons">
              <button
                class="action-btn toggle-btn"
                class:enabled={enabledPlugins.has(plugin.name)}
                onclick={() => togglePlugin(plugin)}
                disabled={operatingPlugin === plugin.name}
                title={enabledPlugins.has(plugin.name) ? '禁用插件' : '启用插件'}
              >
                {enabledPlugins.has(plugin.name) ? '禁用' : '启用'}
              </button>

              {#if enabledPlugins.has(plugin.name)}
                <button
                  class="action-btn reload-btn"
                  onclick={() => reloadPlugin(plugin)}
                  disabled={operatingPlugin === plugin.name}
                  title="重新加载插件"
                >
                  ⟳
                </button>
              {/if}
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  <div class="plugin-footer">
    <button class="refresh-btn" onclick={loadPlugins} disabled={loading}>
      <span class:spinning={loading}>⟳</span>
      刷新插件列表
    </button>
  </div>
</div>

<style>
  .plugin-manager {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .plugin-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .plugin-header h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text-color);
  }

  .plugin-count {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .error-message {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 8px;
    color: #ef4444;
    font-size: 13px;
  }

  .error-dismiss {
    margin-left: auto;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    font-size: 16px;
    padding: 0 4px;
  }

  .loading-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 20px;
    gap: 12px;
    color: var(--text-secondary);
  }

  .loading-spinner {
    font-size: 24px;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .empty-icon {
    font-size: 48px;
  }

  .empty-hint {
    font-size: 12px;
    opacity: 0.7;
  }

  .plugin-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-height: 400px;
    overflow-y: auto;
  }

  .plugin-item {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    padding: 12px;
    background: var(--bg-hover);
    border-radius: 10px;
    transition: all 0.15s;
  }

  .plugin-item:hover {
    background: var(--bg-active);
  }

  .plugin-item.disabled {
    opacity: 0.6;
  }

  .plugin-info {
    display: flex;
    gap: 12px;
    flex: 1;
    min-width: 0;
  }

  .plugin-logo {
    width: 40px;
    height: 40px;
    border-radius: 8px;
    background: var(--bg-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    overflow: hidden;
  }

  .plugin-logo img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .plugin-logo-placeholder {
    font-size: 20px;
  }

  .plugin-details {
    flex: 1;
    min-width: 0;
  }

  .plugin-name {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 500;
    color: var(--text-color);
  }

  .plugin-version {
    font-size: 11px;
    font-weight: 400;
    color: var(--text-secondary);
    background: var(--bg-secondary);
    padding: 2px 6px;
    border-radius: 4px;
  }

  .plugin-meta {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 4px;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .plugin-prefix code {
    background: var(--bg-secondary);
    padding: 2px 6px;
    border-radius: 4px;
    font-family: monospace;
    color: var(--accent-color);
  }

  .plugin-description {
    margin-top: 6px;
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .plugin-actions {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 8px;
    flex-shrink: 0;
  }

  .plugin-status {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
  }

  .action-buttons {
    display: flex;
    gap: 6px;
  }

  .action-btn {
    padding: 4px 10px;
    font-size: 12px;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: transparent;
    color: var(--text-color);
    cursor: pointer;
    transition: all 0.15s;
  }

  .action-btn:hover:not(:disabled) {
    background: var(--bg-active);
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .toggle-btn.enabled {
    border-color: #f59e0b;
    color: #f59e0b;
  }

  .toggle-btn.enabled:hover:not(:disabled) {
    background: rgba(245, 158, 11, 0.1);
  }

  .reload-btn {
    padding: 4px 8px;
  }

  .plugin-footer {
    padding-top: 12px;
    border-top: 1px solid var(--border-color);
  }

  .refresh-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    width: 100%;
    padding: 10px;
    font-size: 13px;
    border: 1px dashed var(--border-color);
    border-radius: 8px;
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s;
  }

  .refresh-btn:hover:not(:disabled) {
    border-color: var(--accent-color);
    color: var(--accent-color);
  }

  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .refresh-btn .spinning {
    animation: spin 1s linear infinite;
  }
</style>
