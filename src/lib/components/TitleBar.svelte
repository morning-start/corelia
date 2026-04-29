<script lang="ts">
  /** 标题栏组件属性接口 */
  interface Props {
    /** 设置按钮点击回调 */
    onSettingsClick?: () => void;
  }

  let { onSettingsClick }: Props = $props();
</script>

<!-- 
  拖拽实现说明：
  1. 整个header添加 data-tauri-drag-region 属性
  2. 按钮需要设置 data-tauri-drag-region="false" 来排除拖拽
  3. 使用 -webkit-app-region: drag/no-drag CSS 属性作为备选
-->
<header class="title-bar" data-tauri-drag-region>
  <!-- 左侧拖拽区域 -->
  <div class="drag-area" data-tauri-drag-region></div>

  <!-- 右侧操作按钮 -->
  <div class="actions" data-tauri-drag-region="false">
    <!-- 设置按钮 -->
    <button 
      class="action-btn" 
      onclick={onSettingsClick} 
      aria-label="设置"
      data-tauri-drag-region="false"
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="3"/>
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
      </svg>
    </button>
  </div>
</header>

<style>
  .title-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 32px;
    padding: 0 12px;
    background: var(--bg-primary);
    border-bottom: 1px solid var(--border-subtle);
    user-select: none;
    -webkit-user-select: none;
    /* 拖拽方案：使用 -webkit-app-region */
    -webkit-app-region: drag;
  }

  .drag-area {
    flex: 1;
    height: 100%;
    min-width: 100px;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
    /* 按钮区域不参与拖拽 */
    -webkit-app-region: no-drag;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    color: var(--text-tertiary);
    transition: all 0.15s ease;
    flex-shrink: 0;
    /* 按钮不参与拖拽 */
    -webkit-app-region: no-drag;
  }

  .action-btn:hover {
    background: var(--bg-hover);
    color: var(--text-secondary);
  }

  .action-btn:active {
    background: var(--bg-active);
  }

  .action-btn svg {
    width: 14px;
    height: 14px;
    /* SVG 不响应鼠标事件 */
    pointer-events: none;
  }
</style>
