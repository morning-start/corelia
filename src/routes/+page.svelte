<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { theme } from '$lib/stores/theme';
  import { settings } from '$lib/stores/settings';
  import { searchHistory } from '$lib/stores/history';
  import { searchStore } from '$lib/stores/search';
  import SearchBox from '$lib/components/SearchBox.svelte';
  import ResultList from '$lib/components/ResultList.svelte';
  import TitleBar from '$lib/components/TitleBar.svelte';
  import SettingPanel from '$lib/components/SettingPanel.svelte';
  import CategoryTabs from '$lib/components/CategoryTabs.svelte';
  import '$lib/styles/themes.css';

  const appWindow = getCurrentWindow();

  let showSettings = $state(false);
  let selectedIndex = $state(-1);
  let queryValue = $state('');
  let resultsValue = $state<any[]>([]);
  let historyItems = $state<string[]>([]);
  let selectedCategory = $state<'all' | 'system' | 'plugin' | 'history'>('all');

  let unsubQuery: (() => void) | undefined;
  let unsubResults: (() => void) | undefined;
  let unsubHistory: (() => void) | undefined;

  onMount(() => {
    settings.load().then(() => {
      theme.set($settings.theme);
    });
    searchHistory.init();

    unsubQuery = searchStore.query.subscribe(v => queryValue = v);
    unsubResults = searchStore.results.subscribe(v => {
      resultsValue = selectedCategory === 'all' ? v : v.filter((r: any) => r.original.category === selectedCategory);
    });
    unsubHistory = searchHistory.subscribe(state => {
      historyItems = state.items.slice(0, 5).map(item => item.query);
    });

    invoke("register_shortcut_cmd").then(() => {
      console.log("全局快捷键注册成功");
    }).catch((e) => {
      console.error("快捷键注册失败:", e);
    });

    // 监听窗口焦点变化，实现失焦自动隐藏
    const unlistenFocus = appWindow.onFocusChanged(async ({ payload: focused }) => {
      console.log('窗口焦点变化:', focused);
      if (!focused && $settings.behavior.autoHide) {
        console.log('窗口失焦，自动隐藏');
        await appWindow.hide();
      }
    });

    return () => {
      unsubQuery?.();
      unsubResults?.();
      unsubHistory?.();
      // 取消监听
      unlistenFocus.then(unlisten => unlisten());
    };
  });

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      if (showSettings) {
        showSettings = false;
      } else {
        appWindow.hide();
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

  function handleSearchInput(query: string) {
    searchStore.setQuery(query);
    selectedIndex = -1;
    if (query) {
      searchHistory.add(query);
    }
  }

  function handleSelectItem(item: any, index: number) {
    console.log('Selected item:', item);
    if (item.action) {
      item.action();
    }
  }

  function handleHistorySelect(query: string) {
    searchStore.setQuery(query);
  }

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
