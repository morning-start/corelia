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

  let { value = $bindable(''), placeholder = '搜索...', onInput }: Props = $props();

  /**
   * 处理输入事件
   * @param event - 输入事件对象
   */
  function handleInput(event: Event) {
    const target = event.target as HTMLInputElement;
    value = target.value;
    onInput?.(value);
  }
</script>

<div class="search-box">
  <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
    <circle cx="11" cy="11" r="8"/>
    <path d="m21 21-4.35-4.35"/>
  </svg>
  <input
    type="text"
    {value}
    {placeholder}
    oninput={handleInput}
  />
  {#if value}
    <button class="clear-btn" aria-label="清除搜索" onclick={() => { value = ''; onInput?.(''); }}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 6L6 18M6 6l12 12"/>
      </svg>
    </button>
  {/if}
</div>

<style>
  .search-box {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    background: var(--bg-hover);
    border-radius: var(--radius);
    border: 1px solid var(--border-color);
  }

  .search-icon {
    width: 20px;
    height: 20px;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-color);
    font-size: 16px;
  }

  input::placeholder {
    color: var(--text-secondary);
  }

  .clear-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    padding: 0;
    background: transparent;
    border: none;
    cursor: pointer;
    color: var(--text-secondary);
    border-radius: 4px;
  }

  .clear-btn:hover {
    color: var(--text-color);
    background: var(--bg-active);
  }

  .clear-btn svg {
    width: 16px;
    height: 16px;
  }
</style>
