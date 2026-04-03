<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { theme } from '$lib/stores/theme';
  import { searchStore } from '$lib/stores/search';
  import SearchBox from '$lib/components/SearchBox.svelte';
  import ResultList from '$lib/components/ResultList.svelte';
  import TitleBar from '$lib/components/TitleBar.svelte';
  import SettingPanel from '$lib/components/SettingPanel.svelte';
  import '$lib/styles/themes.css';

  const appWindow = getCurrentWindow();

  let showSettings = $state(false);
  let selectedIndex = $state(-1);
  let queryValue = $state('');
  let resultsValue = $state<any[]>([]);

  onMount(() => {
    const unsubQuery = searchStore.query.subscribe(v => queryValue = v);
    const unsubResults = searchStore.results.subscribe(v => resultsValue = v);

    invoke("register_shortcut_cmd").then(() => {
      console.log("全局快捷键注册成功");
    }).catch((e) => {
      console.error("快捷键注册失败:", e);
    });

    return () => {
      unsubQuery();
      unsubResults();
    };
  });

  async function handleBlur() {
    await appWindow.hide();
  }

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
  }

  function handleSelectItem(item: any, index: number) {
    console.log('Selected item:', item);
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="window-container"
  role="button"
  tabindex="-1"
  onblur={handleBlur}
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
        {#if queryValue}
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
