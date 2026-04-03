<script lang="ts">
  import type { FilterResult } from 'fuzzy';
  import type { SearchItem } from '$lib/search/fuzzy';

  interface Props {
    results: FilterResult<SearchItem>[];
    selectedIndex?: number;
    onSelect?: (item: SearchItem, index: number) => void;
  }

  let { results = [], selectedIndex = $bindable(-1), onSelect }: Props = $props();
</script>

<div class="result-list">
  {#if results.length === 0}
    <div class="empty">
      <span>暂无结果</span>
    </div>
  {:else}
    {#each results as result, index}
      <button
        class="result-item"
        class:selected={index === selectedIndex}
        onclick={() => onSelect?.(result.original, index)}
      >
        <div class="result-content">
          <span class="result-name">{result.original.name}</span>
          <span class="result-desc">{result.original.description}</span>
        </div>
        <span class="result-category">{result.original.category}</span>
      </button>
    {/each}
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

  .empty {
    padding: 24px;
    text-align: center;
    color: var(--text-secondary);
  }

  .result-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    background: transparent;
    border: none;
    border-radius: 8px;
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
  }

  .result-name {
    color: var(--text-color);
    font-size: 14px;
    font-weight: 500;
  }

  .result-desc {
    color: var(--text-secondary);
    font-size: 12px;
  }

  .result-category {
    color: var(--text-secondary);
    font-size: 11px;
    padding: 2px 8px;
    background: var(--bg-active);
    border-radius: 4px;
  }
</style>
