<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { LogicalSize } from "@tauri-apps/api/dpi";
  import { WINDOW_CONFIG, SEARCH_CONFIG } from '$lib/config';
  import { theme } from '$lib/stores/theme';
  import { system } from '$lib/stores/system';
  import { user } from '$lib/stores/user';
  import { searchHistory } from '$lib/stores/history';
  import { searchStore, type ExecutableItem, type ExtendedSearchResult } from '$lib/stores/search';
  import { resultExecutor } from '$lib/services/executor';
  import { pluginService } from '$lib/plugins/service';
  import { patchLoader } from '$lib/plugins/patch-loader';
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
  let resultsValue = $state<ExtendedSearchResult[]>([]);
  /** 搜索历史列表 */
  let historyItems = $state<string[]>([]);
  /** 当前选中的分类 */
  let selectedCategory = $state<'all' | 'system' | 'plugin' | 'history'>('all');

  /** 本地状态：用于 UI 绑定的用户配置快照 */
  let userConfigSnapshot = $state({ autoHide: true, autoHideDelay: 3000 });

  /** 取消订阅函数列表 */
  let unsubQuery: (() => void) | undefined;
  let unsubResults: (() => void) | undefined;
  let unsubHistory: (() => void) | undefined;
  let unsubUser: (() => void) | undefined;

  /** 历史记录保存防抖定时器（用户停止输入后才记录） */
  let historySaveTimer: ReturnType<typeof setTimeout> | null = null;

  onMount(() => {
    // 设置窗口尺寸（使用 LogicalSize 获得更好的 DPI 支持）
    appWindow.setSize(new LogicalSize(WINDOW_CONFIG.WIDTH, WINDOW_CONFIG.HEIGHT)).catch(console.error);

    // 设置最小和最大尺寸限制
    appWindow.setMinSize(new LogicalSize(WINDOW_CONFIG.MIN_WIDTH, WINDOW_CONFIG.MIN_HEIGHT)).catch(console.error);
    appWindow.setMaxSize(new LogicalSize(WINDOW_CONFIG.MAX_WIDTH, WINDOW_CONFIG.MAX_HEIGHT)).catch(console.error);

    // 加载系统配置和用户配置
    Promise.all([
      system.load(),
      user.load()
    ]).then(() => {
      const loadedTheme = user.get('theme');
      if (loadedTheme) theme.set(loadedTheme as 'dark' | 'light' | 'system');

      // 将历史容量配置同步到 history store
      const searchCfg = user.get('search');
      if (searchCfg?.maxHistoryCapacity) {
        searchHistory.setMaxCapacity(searchCfg.maxHistoryCapacity);
      }
    });

    // 订阅配置变化
    unsubUser = user.subscribe((s) => {
      userConfigSnapshot = { ...s.behavior };
    });

    // 初始化搜索历史
    searchHistory.init();

    // 🔥 初始化插件系统（扫描并加载插件元数据）
    pluginService.init().then((plugins) => {
      console.log(`[+page] ✅ 插件系统初始化完成，发现 ${plugins.length} 个插件`);
      plugins.forEach(p => console.log(`[+page]   - ${p.name}: ${p.description}`));

      // 🔥 初始化 WASM Patch 加载器（监听 Rust 的 WASM 加载请求）
      patchLoader.init().then(() => {
        console.log('[+page] ✅ WASM Patch 加载器初始化完成');
      }).catch((e) => {
        console.error('[+page] ❌ WASM Patch 加载器初始化失败:', e);
      });
    }).catch((e) => {
      console.error('[+page] ❌ 插件系统初始化失败:', e);
    });

    // 订阅搜索查询
    unsubQuery = searchStore.query.subscribe(v => queryValue = v);

    // 🔥 订阅合并后的搜索结果（包含系统项 + 插件项）
    unsubResults = searchStore.results.subscribe(v => {
      resultsValue = filterResultsByCategory(v);
    });
    unsubHistory = searchHistory.subscribe(state => {
      historyItems = state.items.slice(0, SEARCH_CONFIG.DISPLAYED_HISTORY_COUNT).map(item => item.query);
    });

    // 注册全局快捷键
    invoke("register_shortcut_cmd").then(() => {
      console.log("全局快捷键注册成功");
    }).catch((e) => {
      console.error("快捷键注册失败:", e);
    });

    // 监听窗口焦点变化
    const unlistenFocus = appWindow.onFocusChanged(async ({ payload: focused }) => {
      if (!focused && userConfigSnapshot.autoHide) {
        await invoke('hide_window');
      }
    });

    // 清理函数
    return () => {
      unsubQuery?.();
      unsubResults?.();
      unsubHistory?.();
      unsubUser?.();
      unlistenFocus.then(unlisten => unlisten());
    };
  });

  /**
   * 处理键盘事件
   */
  async function handleKeydown(event: KeyboardEvent) {
    // 快捷键录制期间，不处理全局键盘事件（让 ShortcutRecorder 捕获）
    const recorder = document.querySelector('.shortcut-recorder .record-btn.recording');
    if (recorder) return;

    if (event.key === 'Escape') {
      if (showSettings) {
        showSettings = false;
      } else {
        await invoke('hide_window').catch(console.error);
      }
      return;
    }

    // 方向键导航
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      const totalItems = queryValue.length === 0
        ? historyItems.length
        : resultsValue.length;
      if (selectedIndex < totalItems - 1) {
        selectedIndex++;
      }
      return;
    }

    if (event.key === 'ArrowUp') {
      event.preventDefault();
      if (selectedIndex > 0) {
        selectedIndex--;
      }
      return;
    }

    // Enter 执行选中项
    if (event.key === 'Enter') {
      event.preventDefault();

      // 按回车时立即保存搜索词到历史（不等防抖）
      if (queryValue.trim()) {
        if (historySaveTimer) clearTimeout(historySaveTimer);
        searchHistory.add(queryValue);
      }

      // 如果有搜索词，执行搜索结果
      if (queryValue.length > 0 && selectedIndex >= 0) {
        const result = resultsValue[selectedIndex];
        if (result?.original) {
          await executeItem(result.original as ExecutableItem);
        }
      }
      // 如果没有搜索词，执行历史记录
      else if (queryValue.length === 0 && selectedIndex >= 0) {
        const historyQuery = historyItems[selectedIndex];
        if (historyQuery) {
          handleHistorySelect(historyQuery);
        }
      }
      return;
    }
  }

  /**
   * 执行搜索项
   * @param item - 要执行的项
   */
  async function executeItem(item: ExecutableItem) {
    const result = await resultExecutor.execute(item);

    if (result.success) {
      console.log('执行成功:', result.message);
      // 清空搜索词
      searchStore.clearQuery();
      selectedIndex = -1;
    } else {
      console.error('执行失败:', result.message);
      // 可以在这里添加错误提示 UI
    }
  }

  /**
   * 处理搜索输入（统一管理搜索触发和历史记录）
   *
   * 数据流：SearchBox.oninput → handleSearchInput → [搜索防抖] → searchStore.setQuery
   *                                              → [历史防抖 300ms] → searchHistory.add
   */
  function handleSearchInput(query: string) {
    // 立即更新 queryValue（通过 searchStore.query subscribe 同步）
    // 注意：这里立即 setQuery 保证 UI 实时响应，搜索结果由 SearchStore 内部 debounce 控制
    searchStore.setQuery(query);
    selectedIndex = -1;

    if (!query.trim()) {
      // 空输入：清除历史保存定时器
      if (historySaveTimer) { clearTimeout(historySaveTimer); historySaveTimer = null; }
      return;
    }

    // 搜索历史：用户停止输入后才保存（避免记录打字中间态）
    if (historySaveTimer) clearTimeout(historySaveTimer);
    historySaveTimer = setTimeout(() => {
      searchHistory.add(query);
    }, SEARCH_CONFIG.HISTORY_SAVE_DELAY);
  }

  /**
   * 处理选择结果项（鼠标点击）
   */
  async function handleSelectItem(item: ExecutableItem) {
    console.log('Selected item:', item);
    await executeItem(item);
  }

  /**
   * 处理历史记录选择
   */
  function handleHistorySelect(query: string) {
    searchStore.setQuery(query);
  }

  /**
   * 根据当前分类过滤搜索结果
   */
  function filterResultsByCategory(results: ExtendedSearchResult[]): ExtendedSearchResult[] {
    if (selectedCategory === 'all' || selectedCategory === 'history') return results;
    if (selectedCategory === 'plugin') return results.filter(r => r.isPlugin === true);
    if (selectedCategory === 'system') return results.filter(r => r.isPlugin !== true);
    return results;
  }

  /**
   * 处理分类切换
   */
  function handleCategoryChange(category: 'all' | 'system' | 'plugin' | 'history') {
    selectedCategory = category;
    // 立即用当前结果重新过滤
    resultsValue = filterResultsByCategory(resultsValue);
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="window-container">
  <TitleBar onSettingsClick={() => showSettings = !showSettings} />

  <main>
    <div class="search-container">
      <SearchBox
        value={queryValue}
        onInput={handleSearchInput}
      />

      <CategoryTabs
        selected={selectedCategory}
        onSelect={handleCategoryChange}
      />

      <ResultList
        results={resultsValue}
        selectedIndex={selectedIndex}
        showHistory={queryValue.length === 0}
        historyItems={historyItems}
        onSelect={handleSelectItem}
        onHistorySelect={handleHistorySelect}
      />
    </div>
  </main>

  {#if showSettings}
    <SettingPanel onClose={() => showSettings = false} />
  {/if}
</div>
