<script lang="ts">
  /** 预设快捷键选项（被系统拦截无法通过键盘录制捕获的） */
  const PRESET_SHORTCUTS = ['Ctrl+Space', 'Alt+Space'] as const;

  /** 快捷键录制组件属性接口 */
  interface Props {
    /** 当前快捷键值 */
    value?: string;
    /** 快捷键变更回调 */
    onChange?: (shortcut: string) => void;
  }

  let { value = '', onChange }: Props = $props();

  /** 是否正在录制 */
  let isRecording = $state(false);
  /** 录制的按键列表 */
  let recordedKeys = $state<string[]>([]);

  /** 键盘事件处理函数引用 */
  let handleKeyDown: ((event: KeyboardEvent) => void) | undefined;
  let handleKeyUp: ((event: KeyboardEvent) => void) | undefined;

  /** 组件挂载时设置全局键盘事件监听 */
  function setupListeners() {
    // 定义键盘按下处理
    const onKeyDown = (event: KeyboardEvent) => {
      if (!isRecording) return;

      event.preventDefault();
      event.stopPropagation();

      const keys: string[] = [];

      if (event.ctrlKey) keys.push('Ctrl');
      if (event.altKey) keys.push('Alt');
      if (event.shiftKey) keys.push('Shift');
      if (event.metaKey) keys.push('Meta');

      const key = event.code; // 使用 code 而非 key，获得稳定的物理按键标识
      if (key.startsWith('Key') || key.startsWith('Digit') || key.startsWith('F')) {
        // KeyA, Digit1, F1 等 → 取后半部分
        keys.push(key.replace(/^(Key|Digit|F)/, ''));
      } else if (key === 'Space') {
        keys.push('Space');
      } else if (key.startsWith('Arrow')) {
        keys.push(key);
      } else if (key === 'Enter' || key === 'Tab' || key === 'Backspace'
                 || key === 'Delete' || key === 'Insert'
                 || key === 'Home' || key === 'End'
                 || key === 'PageUp' || key === 'PageDown') {
        keys.push(key);
      }

      // 只更新当捕获到有效按键时
      if (keys.length > 0) {
        recordedKeys = keys;
      }
    };

    // 定义键盘释放处理
    const onKeyUp = (event: KeyboardEvent) => {
      if (!isRecording || recordedKeys.length === 0) return;

      // 检查是否还有修饰键按住
      const hasModifier = event.ctrlKey || event.altKey || event.shiftKey || event.metaKey;
      const hasMainKey = recordedKeys.some(k => !['Ctrl', 'Alt', 'Shift', 'Meta'].includes(k));

      // 主键松开或没有主键时完成录制
      if (!hasModifier || !hasMainKey) {
        const shortcut = recordedKeys.join('+');
        stopRecording();
        onChange?.(shortcut);
      }
    };

    handleKeyDown = onKeyDown;
    handleKeyUp = onKeyUp;

    return () => {
      if (handleKeyDown) window.removeEventListener('keydown', handleKeyDown, { capture: true });
      if (handleKeyUp) window.removeEventListener('keyup', handleKeyUp, { capture: true });
    };
  }

  import { onMount } from 'svelte';
  onMount(setupListeners);

  /** 开始录制快捷键 */
  function startRecording() {
    isRecording = true;
    recordedKeys = [];
    window.addEventListener('keydown', handleKeyDown!, { capture: true });
    window.addEventListener('keyup', handleKeyUp!, { capture: true });
  }

  /** 停止录制 */
  function stopRecording() {
    isRecording = false;
    recordedKeys = [];
    window.removeEventListener('keydown', handleKeyDown!, { capture: true });
    window.removeEventListener('keyup', handleKeyUp!, { capture: true });
  }

  /** 处理失去焦点事件 */
  function handleBlur() {
    if (isRecording) stopRecording();
  }

  /** 选择预设快捷键 */
  function selectPreset(shortcut: string) {
    onChange?.(shortcut);
  }

  /** 判断当前值是否为预设选项 */
  function isPreset(value: string | undefined): value is typeof PRESET_SHORTCUTS[number] {
    return PRESET_SHORTCUTS.includes(value as any);
  }
</script>

<div class="shortcut-recorder">
  <button
    class="record-btn"
    class:recording={isRecording}
    tabindex="0"
    onclick={() => { if (!isRecording) startRecording(); }}
    onblur={handleBlur}
  >
    {#if isRecording}
      {#if recordedKeys.length > 0}
        <span class="recording-text">{recordedKeys.join(' + ')}</span>
      {:else}
        <span class="recording-text">按下快捷键...</span>
      {/if}
    {:else if value}
      <span class="shortcut-value">{value}</span>
    {:else}
      <span class="placeholder">点击设置快捷键</span>
    {/if}
  </button>

  {#if value && !isRecording && !isPreset(value)}
    <button class="clear-btn" onclick={() => onChange?.('')} aria-label="清除快捷键">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 6L6 18M6 6l12 12"/>
      </svg>
    </button>
  {/if}

  <!-- 预设快捷键选项 -->
  {#if !isRecording}
    <div class="presets">
      {#each PRESET_SHORTCUTS as preset}
        {@const active = value === preset}
        <button
          class="preset-btn"
          class:active={active}
          onclick={() => selectPreset(preset)}
          title="{preset}（系统保留组合键）"
        >
          {preset}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .shortcut-recorder {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .record-btn {
    min-width: 140px;
    padding: 8px 12px;
    background: var(--bg-hover);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    color: var(--text-color);
    font-size: 13px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .record-btn:hover {
    background: var(--bg-active);
  }

  .record-btn.recording {
    border-color: var(--accent-color);
    background: rgba(var(--accent-rgb, 59, 130, 246), 0.1);
  }

  .shortcut-value {
    font-family: monospace;
  }

  .placeholder {
    color: var(--text-secondary);
  }

  .recording-text {
    color: var(--accent-color);
    animation: pulse 1s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .clear-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .clear-btn:hover {
    background: var(--bg-hover);
    color: var(--text-color);
  }

  .clear-btn svg {
    width: 14px;
    height: 14px;
  }

  .presets {
    display: flex;
    gap: 4px;
  }

  .preset-btn {
    padding: 4px 10px;
    font-size: 12px;
    font-family: monospace;
    background: transparent;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s;
    white-space: nowrap;
  }

  .preset-btn:hover {
    border-color: var(--accent-color);
    color: var(--text-color);
    background: var(--bg-hover);
  }

  .preset-btn.active {
    border-color: var(--accent-color);
    color: var(--accent-color);
    background: rgba(var(--accent-rgb, 59, 130, 246), 0.1);
  }
</style>
