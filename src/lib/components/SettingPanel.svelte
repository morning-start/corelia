<script lang="ts">
  import { onMount } from 'svelte';
  import { theme, type Theme } from '$lib/stores/theme';
  import { system } from '$lib/stores/system';
  import { user } from '$lib/stores/user';
  import type { UserConfig, SystemConfig } from '$lib/config';
  import { startupService } from '$lib/services/startup';
  import { api } from '$lib/api';
  import ShortcutRecorder from '$lib/components/ShortcutRecorder.svelte';
  import PluginManager from '$lib/components/PluginManager.svelte';

  interface Props {
    /** 关闭面板回调函数 */
    onClose?: () => void;
  }

  let { onClose }: Props = $props();

  /** 当前激活的 Tab: 'general' | 'plugins' */
  let activeTab = $state<'general' | 'plugins'>('general');

  /** 系统级配置状态（快捷键、开机启动等） */
  let systemConfig: SystemConfig = $state($system);
  /** 用户级配置状态（主题、行为等） */
  let userConfig: UserConfig = $state($user);
  /** 开机自启动启用状态 */
  let startupEnabled = $state(false);

  onMount(async () => {
    // 加载系统配置和用户配置
    await Promise.all([
      system.load(),
      user.load()
    ]);

    // 获取开机自启动状态
    try {
      startupEnabled = await startupService.isEnabled();
    } catch (e) {
      console.error('Failed to load startup status:', e);
    }
  });

  /** 主题变更处理 */
  function handleThemeChange(newTheme: Theme) {
    theme.set(newTheme);
    user.update('theme', newTheme);
  }

  /** 关闭设置面板 */
  function handleClose() {
    onClose?.();
  }

  /** 失焦自动隐藏开关变更处理 */
  function handleAutoHideChange(event: Event) {
    const target = event.target as HTMLInputElement;
    user.update('behavior.autoHide', target.checked);
  }

  /** 开机自启动开关变更处理 */
  async function handleStartupChange(event: Event) {
    const target = event.target as HTMLInputElement;
    try {
      if (target.checked) {
        await startupService.enable();
        startupEnabled = true;
        systemConfig.startup.enabled = true;
        await system.save(systemConfig);
      } else {
        await startupService.disable();
        startupEnabled = false;
        systemConfig.startup.enabled = false;
        await system.save(systemConfig);
      }
    } catch (e) {
      console.error('Failed to toggle startup:', e);
      target.checked = startupEnabled;
    }
  }

  /** 快捷键变更处理 */
  async function handleShortcutChange(shortcut: string) {
    // 立即更新显示值（乐观更新）
    systemConfig.shortcut.summon = shortcut;

    try {
      if (shortcut) {
        await api.shortcut.register(shortcut);
      } else {
        await api.shortcut.unregisterAll();
      }

      // 持久化到后端
      await system.save(systemConfig);
    } catch (e) {
      console.error('Failed to register shortcut:', e);
      alert('快捷键注册失败，请重试');
      // 失败时回滚显示值
      await system.load();
    }
  }
</script>

<div class="setting-panel">
  <div class="setting-header">
    <h2>设置</h2>
    <button class="close-btn" aria-label="关闭" onclick={handleClose}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 6L6 18M6 6l12 12"/>
      </svg>
    </button>
  </div>

  <!-- Tab 切换 -->
  <div class="setting-tabs">
    <button
      class="tab-btn"
      class:active={activeTab === 'general'}
      onclick={() => activeTab = 'general'}
    >
      通用
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === 'plugins'}
      onclick={() => activeTab = 'plugins'}
    >
      插件管理
    </button>
  </div>

  <!-- 通用设置内容 -->
  {#if activeTab === 'general'}
    <div class="setting-content">
      <section class="setting-section">
        <h3>快捷键</h3>
        <div class="setting-item">
          <span class="setting-label">唤起窗口</span>
          <ShortcutRecorder
            value={systemConfig.shortcut.summon}
            onChange={handleShortcutChange}
          />
        </div>
      </section>

      <section class="setting-section">
        <h3>主题</h3>
        <div class="theme-options">
          <button
            class="theme-btn"
            class:active={$theme === 'dark'}
            onclick={() => handleThemeChange('dark')}
          >
            深色
          </button>
          <button
            class="theme-btn"
            class:active={$theme === 'light'}
            onclick={() => handleThemeChange('light')}
          >
            浅色
          </button>
          <button
            class="theme-btn"
            class:active={$theme === 'system'}
            onclick={() => handleThemeChange('system')}
          >
            跟随系统
          </button>
        </div>
      </section>

      <section class="setting-section">
        <h3>行为</h3>
        <div class="setting-item">
          <span class="setting-label">失焦自动隐藏</span>
          <input
            type="checkbox"
            checked={userConfig.behavior.autoHide}
            onchange={handleAutoHideChange}
          />
        </div>
        <div class="setting-item">
          <span class="setting-label">开机自启动</span>
          <input
            type="checkbox"
            checked={startupEnabled}
            onchange={handleStartupChange}
          />
        </div>
      </section>
    </div>
  {/if}

  <!-- 插件管理内容 -->
  {#if activeTab === 'plugins'}
    <div class="setting-content">
      <PluginManager />
    </div>
  {/if}
</div>

<style>
  .setting-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 16px;
  }

  .setting-tabs {
    display: flex;
    gap: 4px;
    margin-bottom: 20px;
    padding: 4px;
    background: var(--bg-hover);
    border-radius: 10px;
  }

  .tab-btn {
    flex: 1;
    padding: 8px 16px;
    font-size: 13px;
    font-weight: 500;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s;
  }

  .tab-btn:hover {
    color: var(--text-color);
  }

  .tab-btn.active {
    background: var(--bg-primary);
    color: var(--text-color);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .setting-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 24px;
  }

  h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: var(--text-color);
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    color: var(--text-secondary);
  }

  .close-btn:hover {
    background: var(--bg-hover);
    color: var(--text-color);
  }

  .close-btn svg {
    width: 18px;
    height: 18px;
  }

  .setting-content {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .setting-section h3 {
    margin: 0 0 12px 0;
    font-size: 13px;
    font-weight: 500;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .setting-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 0;
    border-bottom: 1px solid var(--border-color);
  }

  .setting-label {
    color: var(--text-color);
    font-size: 14px;
  }

  .theme-options {
    display: flex;
    gap: 8px;
  }

  .theme-btn {
    flex: 1;
    padding: 10px 16px;
    background: var(--bg-hover);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    color: var(--text-color);
    font-size: 13px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .theme-btn:hover {
    background: var(--bg-active);
  }

  .theme-btn.active {
    background: var(--accent-color);
    border-color: var(--accent-color);
  }

  input[type="checkbox"] {
    width: 40px;
    height: 20px;
    appearance: none;
    background: var(--bg-hover);
    border-radius: 10px;
    cursor: pointer;
    position: relative;
  }

  input[type="checkbox"]::before {
    content: '';
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    background: var(--text-secondary);
    border-radius: 50%;
    transition: all 0.15s;
  }

  input[type="checkbox"]:checked {
    background: var(--accent-color);
  }

  input[type="checkbox"]:checked::before {
    left: 22px;
    background: white;
  }
</style>
