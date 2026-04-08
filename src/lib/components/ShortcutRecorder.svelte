<script lang="ts">
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

  /**
   * 处理键盘按下事件
   * @param event - 键盘事件对象
   */
  function handleKeyDown(event: KeyboardEvent) {
    if (!isRecording) return;

    event.preventDefault();
    event.stopPropagation();

    const keys: string[] = [];

    if (event.ctrlKey) keys.push('Ctrl');
    if (event.altKey) keys.push('Alt');
    if (event.shiftKey) keys.push('Shift');
    if (event.metaKey) keys.push('Meta');

    const key = event.key;
    if (key !== 'Control' && key !== 'Alt' && key !== 'Shift' && key !== 'Meta') {
      const displayKey = key.length === 1 ? key.toUpperCase() : key;
      keys.push(displayKey);
    }

    if (keys.length > 0) {
      recordedKeys = keys;
    }
  }

  /**
   * 处理键盘释放事件
   * @param event - 键盘事件对象
   */
  function handleKeyUp(event: KeyboardEvent) {
    if (!isRecording) return;

    if (recordedKeys.length > 0) {
      const shortcut = recordedKeys.join('+');
      isRecording = false;
      onChange?.(shortcut);
    }
  }

  /** 开始录制快捷键 */
  function startRecording() {
    isRecording = true;
    recordedKeys = [];
  }

  /** 处理失去焦点事件 */
  function handleBlur() {
    if (isRecording) {
      isRecording = false;
      recordedKeys = [];
    }
  }

  /** 处理点击事件 */
  function handleClick() {
    if (!isRecording) {
      startRecording();
    }
  }
</script>

<div class="shortcut-recorder">
  <button
    class="record-btn"
    class:recording={isRecording}
    onclick={handleClick}
    onblur={handleBlur}
    onkeydown={handleKeyDown}
    onkeyup={handleKeyUp}
  >
    {#if isRecording}
      <span class="recording-text">按下快捷键...</span>
    {:else if value}
      <span class="shortcut-value">{value}</span>
    {:else}
      <span class="placeholder">点击设置快捷键</span>
    {/if}
  </button>
  {#if value && !isRecording}
    <button class="clear-btn" onclick={() => onChange?.('')} aria-label="清除快捷键">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 6L6 18M6 6l12 12"/>
      </svg>
    </button>
  {/if}
</div>

<style>
  .shortcut-recorder {
    display: flex;
    align-items: center;
    gap: 8px;
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
  }

  .clear-btn:hover {
    background: var(--bg-hover);
    color: var(--text-color);
  }

  .clear-btn svg {
    width: 14px;
    height: 14px;
  }
</style>