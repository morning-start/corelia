<script lang="ts">
  import type { FilterResult } from 'fuzzy';
  import type { ExecutableItem } from '$lib/services/executor';

  /** 结果列表组件属性接口 */
  interface Props {
    /** 搜索结果列表 */
    results: FilterResult<ExecutableItem>[];
    /** 当前选中的索引 */
    selectedIndex?: number;
    /** 是否显示搜索历史 */
    showHistory?: boolean;
    /** 历史记录列表 */
    historyItems?: string[];
    /** 结果项选择回调 */
    onSelect?: (item: ExecutableItem, index: number) => void;
    /** 历史记录选择回调 */
    onHistorySelect?: (query: string) => void;
  }

  let {
    results = [],
    selectedIndex = $bindable(-1),
    showHistory = false,
    historyItems = [],
    onSelect,
    onHistorySelect
  }: Props = $props();

  /**
   * 获取分类图标
   */
  function getCategoryIcon(category: string): string {
    switch (category) {
      case '系统': return 'M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zm0 4v4l3 3';
      case '插件': return 'M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5';
      case '文件': return 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6z';
      case '应用': return 'M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5';
      default: return 'M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20z';
    }
  }

  /**
   * 获取分类颜色
   */
  function getCategoryColor(category: string): { bg: string; icon: string } {
    switch (category) {
      case '系统': return { bg: 'rgba(59, 130, 246, 0.15)', icon: '#60a5fa' };
      case '插件': return { bg: 'rgba(168, 85, 247, 0.15)', icon: '#c084fc' };
      case '文件': return { bg: 'rgba(34, 197, 94, 0.15)', icon: '#4ade80' };
      case '应用': return { bg: 'rgba(236, 72, 153, 0.15)', icon: '#f472b6' };
      default: return { bg: 'rgba(255, 255, 255, 0.1)', icon: '#a1a1aa' };
    }
  }

  /**
   * 处理历史记录选择
   */
  function handleHistorySelect(historyItem: string, index: number) {
    onHistorySelect?.(historyItem);
  }

  /**
   * 处理搜索结果项选择
   */
  function handleResultSelect(item: ExecutableItem, index: number) {
    onSelect?.(item, index);
  }
</script>

<div class="result-list">
  <!-- 搜索历史 -->
  {#if showHistory && historyItems.length > 0}
    <div class="section">
      <div class="section-header">
        <span class="section-title">最近搜索</span>
      </div>
      <div class="section-content">
        {#each historyItems as historyItem, index}
          <button
            class="result-item"
            class:selected={selectedIndex === index}
            onclick={() => handleHistorySelect(historyItem, index)}
          >
            <div class="icon-wrapper history">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="10"/>
                <polyline points="12 6 12 12 16 14"/>
              </svg>
            </div>
            <span class="item-title">{historyItem}</span>
            <span class="shortcut-hint">↵</span>
          </button>
        {/each}
      </div>
    </div>
  {/if}

  <!-- 搜索结果 -->
  {#if results.length > 0}
    {#if showHistory && historyItems.length > 0}
      <div class="divider"></div>
    {/if}

    <div class="section">
      {#if showHistory}
        <div class="section-header">
          <span class="section-title">搜索结果</span>
          <span class="result-count">{results.length} 个结果</span>
        </div>
      {/if}

      <div class="section-content">
        {#each results as result, index}
          {@const item = result.original}
          {@const colors = getCategoryColor(item.category)}
          {@const actualIndex = showHistory ? index + historyItems.length : index}
          <button
            class="result-item"
            class:selected={selectedIndex === actualIndex}
            onclick={() => handleResultSelect(item, actualIndex)}
          >
            <div class="icon-wrapper" style="background: {colors.bg}; color: {colors.icon};">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d={getCategoryIcon(item.category)}/>
              </svg>
            </div>
            <div class="item-content">
              <span class="item-title">{item.name}</span>
              <span class="item-subtitle">{item.description}</span>
            </div>
            <span class="item-category">{item.category}</span>
          </button>
        {/each}
      </div>
    </div>
  {:else if !showHistory}
    <!-- 空状态 -->
    <div class="empty-state">
      <div class="empty-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="11" cy="11" r="8"/>
          <path d="m21 21-4.3-4.3"/>
        </svg>
      </div>
      <span class="empty-text">未找到匹配结果</span>
      <span class="empty-hint">尝试其他关键词或检查拼写</span>
    </div>
  {/if}
</div>

<style>
  .result-list {
    display: flex;
    flex-direction: column;
    max-height: 380px;
    overflow-y: auto;
    background: var(--bg-primary);
  }

  .section {
    display: flex;
    flex-direction: column;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 20px 6px;
  }

  .section-title {
    font-size: var(--font-size-xs);
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .result-count {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
  }

  .section-content {
    display: flex;
    flex-direction: column;
    padding: 0 12px 8px;
    gap: 2px;
  }

  .divider {
    height: 1px;
    background: var(--border-subtle);
    margin: 4px 12px;
  }

  /* 结果项 */
  .result-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    text-align: left;
    transition: all 0.1s ease;
    width: 100%;
  }

  .result-item:hover {
    background: var(--bg-hover);
  }

  .result-item.selected {
    background: var(--bg-selected);
  }

  .result-item.selected .item-title {
    color: var(--color-primary-light);
  }

  /* 图标 */
  .icon-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: var(--radius-sm);
    flex-shrink: 0;
  }

  .icon-wrapper svg {
    width: 18px;
    height: 18px;
  }

  .icon-wrapper.history {
    background: var(--bg-secondary);
    color: var(--text-tertiary);
  }

  /* 内容 */
  .item-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .item-title {
    font-size: var(--font-size-base);
    font-weight: 400;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .item-subtitle {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* 分类标签 */
  .item-category {
    font-size: var(--font-size-xs);
    font-weight: 500;
    color: var(--text-muted);
    flex-shrink: 0;
    padding: 2px 8px;
    background: var(--bg-secondary);
    border-radius: 4px;
  }

  /* 快捷键提示 */
  .shortcut-hint {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    opacity: 0;
    transition: opacity 0.15s;
    padding: 2px 6px;
    background: var(--bg-secondary);
    border-radius: 4px;
  }

  .result-item:hover .shortcut-hint,
  .result-item.selected .shortcut-hint {
    opacity: 1;
  }

  /* 空状态 */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px 20px;
    gap: 12px;
  }

  .empty-icon {
    width: 48px;
    height: 48px;
    color: var(--text-muted);
  }

  .empty-icon svg {
    width: 100%;
    height: 100%;
  }

  .empty-text {
    font-size: var(--font-size-base);
    font-weight: 500;
    color: var(--text-secondary);
  }

  .empty-hint {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }
</style>
