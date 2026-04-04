<script lang="ts">
  import type { FilterResult } from 'fuzzy';
  import type { SearchItem } from '$lib/search/fuzzy';
  import HighlightedText from './HighlightedText.svelte';

  interface Props {
    results: FilterResult<SearchItem>[];
    selectedIndex?: number;
    showHistory?: boolean;
    historyItems?: string[];
    onSelect?: (item: SearchItem | string, index: number) => void;
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

  function handleSelect(item: SearchItem | string, index: number) {
    if (showHistory && typeof item === 'string') {
      onHistorySelect?.(item);
    } else {
      onSelect?.(item as SearchItem, index);
    }
  }
</script>

<div class="result-list">
  {#if showHistory && historyItems.length > 0}
    <div class="history-section">
      <div class="section-header">搜索历史</div>
      {#each historyItems as historyItem, index}
        <button
          class="result-item history-item"
          class:selected={index === selectedIndex}
          onclick={() => handleSelect(historyItem, index)}
        >
          <svg class="history-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <polyline points="12 6 12 12 16 14"/>
          </svg>
          <span class="history-text">{historyItem}</span>
        </button>
      {/each}
    </div>
  {/if}

  {#if !showHistory || results.length > 0}
    {#if showHistory && historyItems.length > 0}
      <div class="section-header">搜索结果</div>
    {/if}
    {#if results.length === 0 && !showHistory}
      <div class="empty">
        <span>暂无结果</span>
      </div>
    {:else}
      {#each results as result, index}
        <button
          class="result-item"
          class:selected={showHistory ? false : index === selectedIndex}
          onclick={() => handleSelect(result.original, index)}
        >
          <div class="result-content">
            <HighlightedText text={result.original.name} query={''} />
            <span class="result-desc">{result.original.description}</span>
          </div>
          <span class="result-category">{result.original.category}</span>
        </button>
      {/each}
    {/if}
  {/if}
</div>

<style>
  .result-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 300px;
    overflow-y: auto;
  }

  .section-header {
    padding: 8px 12px 4px;
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .history-section {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border-color);
    margin-bottom: 4px;
  }

  .history-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: transparent;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    text-align: left;
    transition: background 0.15s;
  }

  .history-item:hover,
  .history-item.selected {
    background: var(--bg-hover);
  }

  .history-icon {
    width: 16px;
    height: 16px;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .history-text {
    color: var(--text-color);
    font-size: 14px;
  }

  .empty {
    padding: 24px;
    text-align: center;
    color: var(--text-secondary);
  }

  .result-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    background: transparent;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    text-align: left;
    transition: background 0.15s;
  }

  .result-item:hover,
  .result-item.selected {
    background: var(--bg-hover);
  }

  .result-item.selected {
    background: var(--bg-active);
  }

  .result-content {
    display: flex;
    flex-direction: column;
    gap: 4px;
    overflow: hidden;
  }

  .result-name {
    color: var(--text-color);
    font-size: 14px;
    font-weight: 500;
  }

  .result-desc {
    color: var(--text-secondary);
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .result-category {
    color: var(--text-secondary);
    font-size: 11px;
    padding: 2px 8px;
    background: var(--bg-active);
    border-radius: 4px;
    flex-shrink: 0;
  }
</style>
