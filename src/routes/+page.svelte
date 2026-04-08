<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { theme } from '$lib/stores/theme';
  import { settings, type Settings } from '$lib/stores/settings';
  import { searchHistory } from '$lib/stores/history';
  import { searchStore } from '$lib/stores/search';
  import SearchBox from '$lib/components/SearchBox.svelte';
  import ResultList from '$lib/components/ResultList.svelte';
  import TitleBar from '$lib/components/TitleBar.svelte';
  import SettingPanel from '$lib/components/SettingPanel.svelte';
  import CategoryTabs from '$lib/components/CategoryTabs.svelte';
  import '$lib/styles/themes.css';

  /** Tauri 主窗口实例 */
  const appWindow = getCurrentWindow();

  /** 是否显示设置面板 */
  let showSettings = $state(false);
  /** 当前选中的结果索引 */
  let selectedIndex = $state(-1);
  /** 搜索查询词 */
  let queryValue = $state('');
  /** 搜索结果列表 */
  let resultsValue = $state<any[]>([]);
  /** 搜索历史列表 */
  let historyItems = $state<string[]>([]);
  /** 当前选中的分类 */
  let selectedCategory = $state<'all' | 'system' | 'plugin' | 'history'>('all');
  /** 当前设置状态 */
  let currentSettings: Settings = $state<Settings>({
    theme: 'dark',
    shortcut: { summon: 'Alt+Space' },
    behavior: { autoHide: true, autoHideDelay: 3000 },
    startup: { enabled: false, minimizeToTray: true }
  });

  /** 取消订阅函数列表 */
  let unsubQuery: (() => void) | undefined;
  let unsubResults: (() => void) | undefined;
  let unsubHistory: (() => void) | undefined;
  let unsubSettings: (() => void) | undefined;

  /**
   * 组件挂载时初始化
   * - 加载设置
   * - 初始化搜索历史
   * - 注册全局快捷键
   * - 监听窗口焦点变化
   */
  onMount(() => {
    // 加载设置
    settings.load().then(() => {
      const loaded = settings.get('theme');
      if (loaded) theme.set(loaded as any);
    });

    // 订阅设置变化
    unsubSettings = settings.subscribe((s) => {
      currentSettings = s;
    });

    // 初始化搜索历史
    searchHistory.init();

    // 订阅搜索查询和结果
    unsubQuery = searchStore.query.subscribe(v => queryValue = v);
    unsubResults = searchStore.results.subscribe(v => {
      resultsValue = selectedCategory === 'all' ? v : v.filter((r: any) => r.original.category === selectedCategory);
    });
    unsubHistory = searchHistory.subscribe(state => {
      historyItems = state.items.slice(0, 5).map(item => item.query);
    });

    // 注册全局快捷键
    invoke("register_shortcut_cmd").then(() => {
      console.log("全局快捷键注册成功");
    }).catch((e) => {
      console.error("快捷键注册失败:", e);
    });

    // 监听窗口焦点变化，实现失焦自动隐藏
    const unlistenFocus = appWindow.onFocusChanged(async ({ payload: focused }) => {
      console.log('窗口焦点变化:', focused);
      if (!focused && currentSettings.behavior.autoHide) {
        console.log('窗口失焦，自动隐藏');
        await invoke('hide_window');
      }
    });

    // 清理函数
    return () => {
      unsubQuery?.();
      unsubResults?.();
      unsubHistory?.();
      unsubSettings?.();
      unlistenFocus.then(unlisten => unlisten());
    };
  });

  /**
   * 处理键盘事件
   * - Escape: 关闭设置面板或隐藏窗口
   * - ArrowUp/Down: 选择上/下一个结果
   * - Enter: 确认选择
   */
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      if (showSettings) {
        showSettings = false;
      } else {
        invoke('hide_window').catch(console.error);
      }
    }
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      if (selectedIndex < resultsValue.length - 1) {
        selectedIndex++;
      }
    }
    if (event.key === 'ArrowUp') {
      event.preventDefault();
      if (selectedIndex > 0) {
        selectedIndex--;
      }
    }
    if (event.key === 'Enter' && selectedIndex >= 0) {
      const result = resultsValue[selectedIndex];
      if (result) {
        console.log('Selected:', result.original);
      }
    }
  }

  /**
   * 处理搜索输入
   */
  function handleSearchInput(query: string) {
    searchStore.setQuery(query);
    selectedIndex = -1;
    if (query) {
      searchHistory.add(query);
    }
  }

  /**
   * 处理选择结果项
   */
  function handleSelectItem(item: any, index: number) {
    console.log('Selected item:', item);
    if (item.action) {
      item.action();
    }
  }

  /**
   * 处理历史记录选择
   */
  function handleHistorySelect(query: string) {
    searchStore.setQuery(query);
  }

  /**
   * 处理分类切换
   */
  function handleCategoryChange(category: 'all' | 'system' | 'plugin' | 'history') {
    selectedCategory = category;
    searchStore.results.subscribe(v => {
      resultsValue = category === 'all' ? v : v.filter((r: any) => r.original.category === category);
    });
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="window-container"
>
  <TitleBar onSettingsClick={() => showSettings = true} />

  <main>
    {#if showSettings}
      <SettingPanel onClose={() => showSettings = false} />
    {:else}
      <div class="search-container">
        <SearchBox
          value={queryValue}
          placeholder="搜索..."
          onInput={handleSearchInput}
        />
        {#if !queryValue}
          <ResultList
            results={[]}
            {selectedIndex}
            showHistory={true}
            {historyItems}
            onHistorySelect={handleHistorySelect}
          />
        {:else}
          <CategoryTabs
            selected={selectedCategory}
            onSelect={handleCategoryChange}
          />
          <ResultList
            results={resultsValue}
            {selectedIndex}
            onSelect={handleSelectItem}
          />
        {/if}
      </div>
    {/if}
  </main>
</div>

<style>
  :global(html, body) {
    margin: 0;
    padding: 0;
    background: transparent;
    overflow: hidden;
  }

  :global(body) {
    background: transparent;
  }

  .window-container {
    background: var(--bg-color);
    border-radius: var(--radius);
    box-shadow: var(--shadow);
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  main {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 12px;
    overflow: hidden;
  }

  .search-container {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
</style>
