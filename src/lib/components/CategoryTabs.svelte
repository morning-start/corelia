<script lang="ts">
  /** 搜索分类类型 */
  type Category = 'all' | 'system' | 'plugin' | 'history';

  /** 分类标签组件属性接口 */
  interface Props {
    /** 当前选中的分类 */
    selected?: Category;
    /** 分类切换回调 */
    onSelect?: (category: Category) => void;
  }

  let { selected = $bindable('all' as Category), onSelect }: Props = $props();

  /** 分类配置列表 */
  const categories: { id: Category; label: string; icon: string }[] = [
    { id: 'all', label: '全部', icon: 'M4 6h16M4 12h16M4 18h16' },
    { id: 'system', label: '系统', icon: 'M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zm0 4v4l3 3' },
    { id: 'plugin', label: '插件', icon: 'M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5' },
    { id: 'history', label: '历史', icon: 'M12 8v4l3 3m6-3a9 9 0 1 1-18 0 9 9 0 0 1 18 0z' }
  ];

  /**
   * 切换分类
   */
  function selectCategory(cat: Category) {
    selected = cat;
    onSelect?.(cat);
  }
</script>

<div class="category-tabs">
  {#each categories as cat}
    <button
      class="tab"
      class:active={selected === cat.id}
      onclick={() => selectCategory(cat.id)}
    >
      <svg class="tab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d={cat.icon} />
      </svg>
      <span class="tab-label">{cat.label}</span>
    </button>
  {/each}
</div>

<style>
  .category-tabs {
    display: flex;
    gap: 4px;
    padding: 12px 20px;
    background: var(--bg-primary);
    border-bottom: 1px solid var(--border-subtle);
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--text-tertiary);
    font-size: var(--font-size-sm);
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .tab:hover {
    color: var(--text-secondary);
    background: var(--bg-hover);
  }

  .tab.active {
    color: var(--text-primary);
    background: var(--bg-secondary);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
  }

  .tab-icon {
    width: 14px;
    height: 14px;
  }

  .tab-label {
    font-weight: 500;
  }
</style>
