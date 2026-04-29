<script lang="ts">
  import { onMount } from 'svelte';
  import { pluginService, type PluginActionResult } from '$lib/plugins/service';
  import { pluginStore } from '$lib/plugins/store';
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

  // ====== 新增：详情面板状态 ======
  /** 选中的插件（显示详情） */
  let selectedPlugin = $state<PluginManifest | null>(null);
  /** 详情面板 Tab: details | data | test | vm */
  let detailTab = $state<'details' | 'data' | 'test' | 'vm'>('details');
  /** 插件数据键列表 */
  let pluginDataKeys = $state<string[]>([]);
  /** 选中的数据值 */
  let selectedDataValue = $state<string>('');
  /** 测试搜索词 */
  let testQuery = $state('');
  /** 测试搜索结果 */
  let testResults = $state<unknown[]>([]);
  /** 测试执行结果 */
  let testActionResult = $state<string>('');
  /** 是否正在测试 */
  let testing = $state(false);
  /** VM 缓存状态 */
  let vmCacheStatus = pluginService.getCacheStatus();

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
      // 更新 VM 缓存状态
      vmCacheStatus = pluginService.getCacheStatus();
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
        await pluginService.unload(plugin.name);
        enabledPlugins.delete(plugin.name);
      } else {
        await pluginService.load(plugin.name);
        enabledPlugins.add(plugin.name);
      }
      enabledPlugins = new Set(enabledPlugins);
      saveEnabledPlugins();
      vmCacheStatus = pluginService.getCacheStatus();
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
      vmCacheStatus = pluginService.getCacheStatus();
      console.log(`[PluginManager] 插件 ${plugin.name} 已重新加载`);
    } catch (e) {
      console.error(`[PluginManager] 重新加载插件失败 (${plugin.name}):`, e);
      error = `重新加载失败: ${e}`;
    } finally {
      operatingPlugin = null;
    }
  }

  /** 选中插件查看详情 */
  function selectPlugin(plugin: PluginManifest) {
    if (selectedPlugin?.name === plugin.name) {
      selectedPlugin = null; // 再次点击关闭详情
      return;
    }
    selectedPlugin = plugin;
    detailTab = 'details';
    testActionResult = '';
    testResults = [];
    testQuery = '';
    loadPluginData(plugin.name);
  }

  /** 加载插件数据键列表 */
  async function loadPluginData(pluginId: string) {
    try {
      // 尝试获取数据路径确认插件有数据
      const path = await pluginStore.getDataPath(pluginId);
      console.log(`[PluginManager] 插件数据目录: ${path}`);
      pluginDataKeys = ['(通过后端 API 读写)'];
      selectedDataValue = '';
    } catch {
      pluginDataKeys = [];
      selectedDataValue = '';
    }
  }

  /** 执行测试搜索 */
  async function runTestSearch() {
    if (!selectedPlugin || !testQuery.trim()) return;
    testing = true;
    testActionResult = '';
    try {
      const results = await pluginService.executeSearch(selectedPlugin.name, testQuery.trim());
      testResults = results;
    } catch (e) {
      testActionResult = `搜索错误: ${e}`;
    } finally {
      testing = false;
    }
  }

  /** 执行测试动作 */
  async function runTestAction(action: string) {
    if (!selectedPlugin) return;
    testing = true;
    testActionResult = '';
    try {
      const result: PluginActionResult = await pluginService.executeAction(selectedPlugin.name, action);
      testActionResult = JSON.stringify(result, null, 2);
    } catch (e) {
      testActionResult = `执行错误: ${e}`;
    } finally {
      testing = false;
    }
  }

  /** 获取插件状态图标 */
  function getPluginStatus(plugin: PluginManifest): { icon: string; color: string; label: string } {
    const isEnabled = enabledPlugins.has(plugin.name);
    const isOperating = operatingPlugin === plugin.name;

    if (isOperating) return { icon: '⟳', color: 'var(--accent-color)', label: '操作中...' };
    if (isEnabled) return { icon: '\u25CF', color: '#22c55e', label: '已启用' };
    return { icon: '\u25CB', color: 'var(--text-secondary)', label: '已禁用' };
  }

  /** 格式化前缀显示 */
  function formatPrefix(prefix: string | undefined): string {
    return prefix ? prefix : '\u2014';
  }

  /** 格式化文件大小 */
  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
</script>

<div class="plugin-manager">
  <div class="plugin-header">
    <h3>插件管理</h3>
    <span class="plugin-count">{plugins.length} 个插件{vmCacheStatus.size > 0 ? ` / ${vmCacheStatus.size} 个VM缓存` : ''}</span>
  </div>

  {#if error}
    <div class="error-message">
      <span class="error-icon">!</span>
      <span>{error}</span>
      <button class="error-dismiss" onclick={() => error = null}>&times;</button>
    </div>
  {/if}

  {#if loading}
    <div class="loading-state">
      <span class="loading-spinner">&#8635;</span>
      <span>正在扫描插件...</span>
    </div>
  {:else if plugins.length === 0}
    <div class="empty-state">
      <span class="empty-icon">&#128230;</span>
      <p>暂未安装任何插件</p>
      <p class="empty-hint">将插件放置到 plugins 目录即可自动加载</p>
    </div>
  {:else}
    <div class="main-content">
      <!-- 左侧：插件列表 -->
      <div class="plugin-list" class:has-detail={!!selectedPlugin}>
        {#each plugins as plugin}
          {@const status = getPluginStatus(plugin)}
          {@const isSelected = selectedPlugin?.name === plugin.name}
          <div
            class="plugin-item"
            class:disabled={!enabledPlugins.has(plugin.name)}
            class:selected={isSelected}
            onclick={() => selectPlugin(plugin)}
          >
            <div class="plugin-info">
              <div class="plugin-logo">
                {#if plugin.logo}
                  <img src={plugin.logo} alt={plugin.name} />
                {:else}
                  <span class="plugin-logo-placeholder">&#128230;</span>
                {/if}
              </div>
              <div class="plugin-details">
                <div class="plugin-name">
                  {plugin.name}
                  <span class="plugin-version">v{plugin.version}</span>
                </div>
                <div class="plugin-meta">
                  <span class="plugin-prefix">{formatPrefix(plugin.prefix)}</span>
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
                  onclick={(e) => { e.stopPropagation(); togglePlugin(plugin); }}
                  disabled={operatingPlugin === plugin.name}
                  title={enabledPlugins.has(plugin.name) ? '禁用插件' : '启用插件'}
                >
                  {enabledPlugins.has(plugin.name) ? '禁用' : '启用'}
                </button>

                {#if enabledPlugins.has(plugin.name)}
                  <button
                    class="action-btn reload-btn"
                    onclick={(e) => { e.stopPropagation(); reloadPlugin(plugin); }}
                    disabled={operatingPlugin === plugin.name}
                    title="重新加载插件"
                  >
                    &#8635;
                  </button>
                {/if}
              </div>
            </div>
          </div>
        {/each}
      </div>

      <!-- 右侧：详情面板 -->
      {#if selectedPlugin}
        <div class="detail-panel">
          <div class="detail-header">
            <h4>{selectedPlugin.name}</h4>
            <button class="close-detail" onclick={() => selectedPlugin = null}>&times;</button>
          </div>

          <!-- Tab 切换 -->
          <div class="detail-tabs">
            <button
              class="tab-btn"
              class:active={detailTab === 'details'}
              onclick={() => detailTab = 'details'}
            >详情</button>
            <button
              class="tab-btn"
              class:active={detailTab === 'test'}
              onclick={() => detailTab = 'test'}
            >测试</button>
            <button
              class="tab-btn"
              class:active={detailTab === 'data'}
              onclick={() => detailTab = 'data'}
            >数据</button>
            <button
              class="tab-btn"
              class:active={detailTab === 'vm'}
              onclick={() => { detailTab = 'vm'; vmCacheStatus = pluginService.getCacheStatus(); }}
            >VM</button>
          </div>

          <div class="detail-content">
            {#if detailTab === 'details'}
              <div class="info-grid">
                <div class="info-item"><label>名称</label><span>{selectedPlugin.name}</span></div>
                <div class="info-item"><label>版本</label><span>{selectedPlugin.version}</span></div>
                <div class="info-item"><label>类型</label><span>{selectedPlugin.type ?? 'quickjs'}</span></div>
                <div class="info-item"><label>触发前缀</label><span><code>{formatPrefix(selectedPlugin.prefix)}</code></span></div>
                <div class="info-item"><label>入口文件</label><span><code>{selectedPlugin.main ?? 'index.js'}</code></span></div>
                <div class="info-item"><label>作者</label><span>{selectedPlugin.author ?? '-'}</span></div>
              </div>
              {#if selectedPlugin.description}
                <div class="section">
                  <label>描述</label>
                  <p>{selectedPlugin.description}</p>
                </div>
              {/if}
              {#if selectedPlugin.features && selectedPlugin.features.length > 0}
                <div class="section">
                  <label>功能列表 ({selectedPlugin.features.length})</label>
                  {#each selectedPlugin.features as feature}
                    <div class="feature-card">
                      <span class="feature-code">{feature.code}</span>
                      <span class="feature-label">{feature.label}</span>
                      <span class="feature-type">{feature.type}</span>
                    </div>
                  {/each}
                </div>
              {/if}

            {:else if detailTab === 'test'}
              <div class="test-area">
                <div class="test-search">
                  <input
                    type="text"
                    bind:value={testQuery}
                    placeholder="输入关键词测试 onSearch..."
                    onkeydown={(e) => e.key === 'Enter' && runTestSearch()}
                  />
                  <button onclick={runTestSearch} disabled={testing || !testQuery.trim()}>
                    {testing ? '...' : '搜索'}
                  </button>
                </div>

                {#if testResults.length > 0}
                  <div class="test-results">
                    <label>搜索结果 ({testResults.length})</label>
                    {#each testResults as r}
                      {@const item = r as Record<string, unknown>}
                      <div class="result-item" onclick={() => runTestAction(String(item.action ?? ''))}>
                        <span class="result-icon">{item.icon ?? '&#128269;'}</span>
                        <div class="result-text">
                          <span class="result-title">{item.title}</span>
                          <span class="result-desc">{item.description}</span>
                        </div>
                      </div>
                    {/each}
                  </div>
                {/if}

                {#if testActionResult}
                  <div class="test-action-result">
                    <label>执行结果</label>
                    <pre>{testActionResult}</pre>
                  </div>
                {/if}
              </div>

            {:else if detailTab === 'data'}
              <div class="data-area">
                <p class="data-hint">插件数据存储在独立的数据隔离空间中</p>
                <div class="data-actions">
                  <button onclick={async () => {
                    if (!selectedPlugin) return;
                    try {
                      const size = await pluginStore.getDataSize(selectedPlugin.name);
                      selectedDataValue = `数据大小: ${formatSize(size)}\n\n数据路径: ${await pluginStore.getDataPath(selectedPlugin.name)}`;
                    } catch (e) {
                      selectedDataValue = `读取失败: ${e}`;
                    }
                  }}>查看数据大小</button>
                  <button class="danger" onclick={async () => {
                    if (!selectedPlugin || !confirm(`确定清除 ${selectedPlugin.name} 的所有数据？`)) return;
                    try {
                      await pluginStore.clearData(selectedPlugin.name);
                      selectedDataValue = '已清除所有数据';
                    } catch (e) {
                      selectedDataValue = `清除失败: ${e}`;
                    }
                  }}>清除所有数据</button>
                </div>
                {#if selectedDataValue}
                  <pre class="data-value">{selectedDataValue}</pre>
                {/if}
              </div>

            {:else if detailTab === 'vm'}
              <div class="vm-area">
                {#if vmCacheStatus.entries.length === 0}
                  <p class="vm-empty">当前无缓存的 VM 实例</p>
                {:else}
                  <div class="vm-list">
                    <label>已缓存 VM ({vmCacheStatus.entries.length}/10)</label>
                    {#each vmCacheStatus.entries as entry}
                      <div class="vm-item">
                        <span class="vm-plugin">{entry.pluginId}</span>
                        <span class="vm-id"><code>{entry.vmId.substring(0, 20)}...</code></span>
                        <span class="vm-time">{new Date(entry.lastUsed).toLocaleTimeString()}</span>
                      </div>
                    {/each}
                  </div>
                {/if}
              </div>
            {/if}
          </div>
        </div>
      {/if}
    </div>
  {/if}

  <div class="plugin-footer">
    <button class="refresh-btn" onclick={loadPlugins} disabled={loading}>
      <span class:spinning={loading}>&#8635;</span>
      刷新插件列表
    </button>
  </div>
</div>

<style>
  .plugin-manager {
    display: flex;
    flex-direction: column;
    gap: 16px;
    min-height: 0;
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

  .empty-icon { font-size: 48px; }
  .empty-hint { font-size: 12px; opacity: 0.7; }

  /* 主布局：左右分栏 */
  .main-content {
    display: flex;
    gap: 12px;
    min-height: 420px;
  }

  .plugin-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 450px;
    overflow-y: auto;
    flex: 1;
    min-width: 0;
  }

  .plugin-list.has-detail { max-width: 360px; }

  .plugin-item {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    padding: 10px 12px;
    background: var(--bg-hover);
    border-radius: 10px;
    transition: all 0.15s;
    cursor: pointer;
    border: 1.5px solid transparent;
  }

  .plugin-item:hover { background: var(--bg-active); }
  .plugin-item.disabled { opacity: 0.6; }
  .plugin-item.selected {
    border-color: var(--accent-color);
    background: color-mix(in srgb, var(--accent-color) 8%, var(--bg-hover));
  }

  .plugin-info {
    display: flex;
    gap: 10px;
    flex: 1;
    min-width: 0;
  }

  .plugin-logo {
    width: 36px;
    height: 36px;
    border-radius: 8px;
    background: var(--bg-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    overflow: hidden;
  }

  .plugin-logo img { width: 100%; height: 100%; object-fit: cover; }
  .plugin-logo-placeholder { font-size: 18px; }

  .plugin-details { flex: 1; min-width: 0; }

  .plugin-name {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    font-weight: 500;
    color: var(--text-color);
  }

  .plugin-version {
    font-size: 10px;
    font-weight: 400;
    color: var(--text-secondary);
    background: var(--bg-secondary);
    padding: 1px 5px;
    border-radius: 3px;
  }

  .plugin-meta {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 3px;
    font-size: 11px;
    color: var(--text-secondary);
  }

  .plugin-prefix code {
    background: var(--bg-secondary);
    padding: 1px 5px;
    border-radius: 3px;
    font-family: monospace;
    color: var(--accent-color);
    font-size: 10px;
  }

  .plugin-author { font-size: 11px; }
  .plugin-description { margin-top: 4px; font-size: 11px; color: var(--text-secondary); line-height: 1.3; display: -webkit-box; -webkit-line-clamp: 1; -webkit-box-orient: vertical; overflow: hidden; }

  .plugin-actions {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 6px;
    flex-shrink: 0;
  }

  .plugin-status { display: flex; align-items: center; gap: 3px; font-size: 11px; }
  .action-buttons { display: flex; gap: 4px; }

  .action-btn {
    padding: 3px 8px;
    font-size: 11px;
    border: 1px solid var(--border-color);
    border-radius: 5px;
    background: transparent;
    color: var(--text-color);
    cursor: pointer;
    transition: all 0.15s;
  }

  .action-btn:hover:not(:disabled) { background: var(--bg-active); }
  .action-btn:disabled { opacity: 0.5; cursor: not-allowed; }

  .toggle-btn.enabled { border-color: #f59e0b; color: #f59e0b; }
  .toggle-btn.enabled:hover:not(:disabled) { background: rgba(245, 158, 11, 0.1); }
  .reload-btn { padding: 3px 7px; }

  /* ====== 详情面板 ====== */
  .detail-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border-color);
    border-radius: 12px;
    overflow: hidden;
    min-width: 300px;
  }

  .detail-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-hover);
  }

  .detail-header h4 { margin: 0; font-size: 14px; font-weight: 600; }

  .close-detail {
    background: none; border: none; cursor: pointer; font-size: 18px;
    color: var(--text-secondary); line-height: 1;
  }

  .detail-tabs {
    display: flex;
    gap: 0;
    border-bottom: 1px solid var(--border-color);
  }

  .tab-btn {
    flex: 1; padding: 8px; font-size: 12px; border: none;
    background: transparent; color: var(--text-secondary);
    cursor: pointer; border-bottom: 2px solid transparent;
    transition: all 0.15s;
  }

  .tab-btn.active {
    color: var(--accent-color);
    border-bottom-color: var(--accent-color);
    font-weight: 500;
  }

  .detail-content {
    flex: 1;
    overflow-y: auto;
    padding: 12px 14px;
  }

  /* 详情 Tab */
  .info-grid { display: grid; grid-template-columns: auto 1fr; gap: 6px 12px; }
  .info-item label { font-size: 11px; color: var(--text-secondary); }
  .info-item span { font-size: 12px; }
  .info-item code { background: var(--bg-secondary); padding: 1px 5px; border-radius: 3px; font-size: 11px; }

  .section { margin-top: 14px; }
  .section label { display: block; font-size: 11px; font-weight: 600; color: var(--text-secondary); margin-bottom: 6px; text-transform: uppercase; letter-spacing: 0.05em; }
  .section p { font-size: 12px; line-height: 1.5; margin: 0; color: var(--text-muted); }

  .feature-card {
    display: flex; gap: 8px; align-items: center;
    padding: 6px 10px; background: var(--bg-hover); border-radius: 6px;
    margin-bottom: 4px; font-size: 12px;
  }

  .feature-code { font-family: monospace; background: var(--bg-secondary); padding: 1px 5px; border-radius: 3px; font-size: 11px; }
  .feature-label { flex: 1; }
  .feature-type { font-size: 10px; color: var(--text-secondary); opacity: 0.7; }

  /* 测试 Tab */
  .test-area { display: flex; flex-direction: column; gap: 10px; }

  .test-search { display: flex; gap: 6px; }
  .test-search input {
    flex: 1; padding: 7px 10px; border: 1px solid var(--border-color);
    border-radius: 6px; font-size: 12px; background: var(--bg-hover);
    color: var(--text-color); outline: none;
  }
  .test-search input:focus { border-color: var(--accent-color); }

  .test-search button {
    padding: 7px 14px; font-size: 12px; border: 1px solid var(--accent-color);
    border-radius: 6px; background: var(--accent-color); color: white;
    cursor: pointer; font-weight: 500;
  }

  .test-results label, .test-action-result label {
    display: block; font-size: 11px; font-weight: 600; color: var(--text-secondary); margin-bottom: 4px;
  }

  .result-item {
    display: flex; gap: 8px; padding: 6px 8px; border-radius: 6px;
    background: var(--bg-hover); cursor: pointer; transition: background 0.15s;
    font-size: 12px;
  }
  .result-item:hover { background: var(--bg-active); }
  .result-icon { font-size: 14px; }
  .result-text { display: flex; flex-direction: column; }
  .result-title { font-weight: 500; }
  .result-desc { font-size: 11px; color: var(--text-secondary); }

  .test-action-result pre {
    background: var(--bg-secondary); padding: 10px; border-radius: 6px;
    font-size: 11px; overflow-x: auto; max-height: 150px; overflow-y: auto;
  }

  /* 数据 Tab */
  .data-area { display: flex; flex-direction: column; gap: 10px; }
  .data-hint { font-size: 12px; color: var(--text-secondary); margin: 0; }

  .data-actions { display: flex; gap: 8px; }
  .data-actions button {
    padding: 7px 12px; font-size: 12px; border: 1px solid var(--border-color);
    border-radius: 6px; background: transparent; color: var(--text-color); cursor: pointer;
  }
  .data-actions button.danger { border-color: #ef4444; color: #ef4444; }

  .data-value {
    background: var(--bg-secondary); padding: 10px; border-radius: 6px;
    font-size: 11px; overflow-x: auto; max-height: 200px; overflow-y: auto;
  }

  /* VM Tab */
  .vm-area { display: flex; flex-direction: column; gap: 8px; }
  .vm-empty { font-size: 12px; color: var(--text-secondary); }
  .vm-list label { display: block; font-size: 11px; font-weight: 600; color: var(--text-secondary); margin-bottom: 4px; }

  .vm-item {
    display: flex; gap: 8px; align-items: center; padding: 6px 10px;
    background: var(--bg-hover); border-radius: 6px; font-size: 11px;
  }

  .vm-plugin { font-weight: 500; }
  .vm-id code { background: var(--bg-secondary); padding: 1px 4px; border-radius: 3px; font-size: 10px; }
  .vm-time { margin-left: auto; color: var(--text-secondary); font-size: 10px; }

  /* 底部 */
  .plugin-footer {
    padding-top: 12px;
    border-top: 1px solid var(--border-color);
  }

  .refresh-btn {
    display: flex; align-items: center; justify-content: center; gap: 6px;
    width: 100%; padding: 10px; font-size: 13px;
    border: 1px dashed var(--border-color); border-radius: 8px;
    background: transparent; color: var(--text-secondary); cursor: pointer;
  }

  .refresh-btn:hover:not(:disabled) { border-color: var(--accent-color); color: var(--accent-color); }
  .refresh-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .refresh-btn .spinning { animation: spin 1s linear infinite; }
</style>
