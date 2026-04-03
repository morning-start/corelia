<script lang="ts">
  import { theme, type Theme } from '$lib/stores/theme';
  import { settings, type Settings } from '$lib/stores/settings';

  interface Props {
    onClose?: () => void;
  }

  let { onClose }: Props = $props();

  let currentSettings: Settings = $state($settings);

  function handleThemeChange(newTheme: Theme) {
    theme.set(newTheme);
    currentSettings.theme = newTheme;
    settings.save(currentSettings);
  }

  function handleClose() {
    onClose?.();
  }

  function handleAutoHideChange(event: Event) {
    const target = event.target as HTMLInputElement;
    currentSettings.behavior.autoHide = target.checked;
    settings.save(currentSettings);
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

  <div class="setting-content">
    <section class="setting-section">
      <h3>快捷键</h3>
      <div class="setting-item">
        <span class="setting-label">唤起窗口</span>
        <button class="shortcut-btn">Alt + Space</button>
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
          checked={currentSettings.behavior.autoHide}
          onchange={handleAutoHideChange}
        />
      </div>
    </section>
  </div>
</div>

<style>
  .setting-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 16px;
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

  .shortcut-btn {
    padding: 6px 12px;
    background: var(--bg-hover);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    color: var(--text-color);
    font-size: 13px;
    cursor: pointer;
  }

  .shortcut-btn:hover {
    background: var(--bg-active);
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
