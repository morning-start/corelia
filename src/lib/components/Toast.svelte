<script lang="ts">
  type ToastType = 'success' | 'error' | 'warning' | 'info';

  interface Toast {
    id: number;
    message: string;
    type: ToastType;
  }

  let toasts = $state<Toast[]>([]);
  let nextId = 1;

  function show(message: string, type: ToastType = 'info', duration = 3000) {
    const id = nextId++;
    toasts = [...toasts, { id, message, type }];
    
    setTimeout(() => {
      remove(id);
    }, duration);
  }

  function remove(id: number) {
    toasts = toasts.filter(t => t.id !== id);
  }

  function getIcon(type: ToastType): string {
    switch (type) {
      case 'success': return '✓';
      case 'error': return '✕';
      case 'warning': return '⚠';
      case 'info': return 'ℹ';
    }
  }

  function getTypeColor(type: ToastType): string {
    switch (type) {
      case 'success': return 'var(--success-color, #22c55e)';
      case 'error': return 'var(--error-color, #ef4444)';
      case 'warning': return 'var(--warning-color, #f59e0b)';
      case 'info': return 'var(--info-color, #3b82f6)';
    }
  }

  // 导出方法供外部调用
  export const toast = {
    show,
    success: (msg: string) => show(msg, 'success'),
    error: (msg: string) => show(msg, 'error'),
    warning: (msg: string) => show(msg, 'warning'),
    info: (msg: string) => show(msg, 'info'),
  };
</script>

{#if toasts.length > 0}
  <div class="toast-container">
    {#each toasts as toast (toast.id)}
      <div class="toast" style="--toast-color: {getTypeColor(toast.type)}">
        <span class="toast-icon">{getIcon(toast.type)}</span>
        <span class="toast-message">{toast.message}</span>
        <button class="toast-close" onclick={() => remove(toast.id)}>&times;</button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    bottom: 20px;
    right: 20px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    z-index: 10000;
    pointer-events: none;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 16px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-left: 4px solid var(--toast-color);
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    pointer-events: auto;
    animation: slideIn 0.3s ease;
    max-width: 350px;
  }

  @keyframes slideIn {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }

  .toast-icon {
    font-size: 16px;
    font-weight: bold;
    color: var(--toast-color);
    flex-shrink: 0;
  }

  .toast-message {
    font-size: 13px;
    color: var(--text-color);
    line-height: 1.4;
    flex: 1;
    word-wrap: break-word;
  }

  .toast-close {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 16px;
    cursor: pointer;
    padding: 0;
    line-height: 1;
    flex-shrink: 0;
  }

  .toast-close:hover {
    color: var(--text-color);
  }
</style>
