<script lang="ts">
  /** 搜索框组件属性接口 */
  interface Props {
    /** 当前搜索值 */
    value?: string;
    /** 输入框占位文本 */
    placeholder?: string;
    /** 输入事件回调 */
    onInput?: (value: string) => void;
  }

  let { value = $bindable(''), placeholder = '搜索应用、命令或文件...', onInput }: Props = $props();

  /**
   * 处理输入事件
   */
  function handleInput(event: Event) {
    const target = event.target as HTMLInputElement;
    value = target.value;
    onInput?.(value);
  }

  /**
   * 清除输入内容
   */
  function clearInput() {
    value = '';
    onInput?.('');
  }
</script>

<div class="search-box">
  <div class="search-icon-wrapper">
    <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="11" cy="11" r="8"/>
      <path d="m21 21-4.3-4.3"/>
    </svg>
  </div>

  <input
    type="text"
    class="search-input"
    {value}
    {placeholder}
    oninput={handleInput}
    autocomplete="off"
    autocorrect="off"
    autocapitalize="off"
    spellcheck="false"
  />

  {#if value}
    <button class="clear-btn" onclick={clearInput} aria-label="清除">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M18 6 6 18"/>
        <path d="m6 6 12 12"/>
      </svg>
    </button>
  {/if}
</div>

<style>
  .search-box {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px 20px;
    background: var(--bg-primary);
    border-bottom: 1px solid var(--border-subtle);
  }

  .search-icon-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: var(--radius-sm);
    background: var(--bg-secondary);
    flex-shrink: 0;
  }

  .search-icon {
    width: 18px;
    height: 18px;
    color: var(--text-tertiary);
  }

  .search-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    font-size: 20px;
    font-weight: 400;
    font-family: inherit;
    color: var(--text-primary);
    min-width: 0;
    padding: 4px 0;
  }

  .search-input::placeholder {
    color: var(--text-muted);
  }

  /* 清除按钮 */
  .clear-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    padding: 0;
    background: var(--text-muted);
    border: none;
    border-radius: 50%;
    cursor: pointer;
    color: var(--bg-primary);
    transition: all 0.15s ease;
    flex-shrink: 0;
  }

  .clear-btn:hover {
    background: var(--text-tertiary);
    transform: scale(1.05);
  }

  .clear-btn svg {
    width: 14px;
    height: 14px;
  }
</style>
