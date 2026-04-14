import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { searchHistory } from '$lib/stores/history';
import { pluginService } from '$lib/plugins/service';
import type { SearchItem } from '$lib/search/fuzzy';

export interface ExecutableItem extends SearchItem {
  type: 'app' | 'url' | 'path' | 'command' | 'setting' | 'plugin';
  target: string;
  args?: string[];
  hideWindow?: boolean;
}

export interface ExecutionResult {
  success: boolean;
  message?: string;
}

class ResultExecutor {
  private appWindow = getCurrentWindow();

  async execute(item: ExecutableItem): Promise<ExecutionResult> {
    try {
      await this.recordToHistory(item);

      const result = await this.executeByType(item);
      await this.conditionalHide(item);
      return result;
    } catch (error) {
      console.error('执行失败:', error);
      return { success: false, message: String(error) };
    }
  }

  private async executeByType(item: ExecutableItem): Promise<ExecutionResult> {
    switch (item.type) {
      case 'app': return this.executeApp(item);
      case 'url': return this.executeUrl(item);
      case 'path': return this.executePath(item);
      case 'command': return this.executeCommand(item);
      case 'setting': return this.executeSetting(item);
      case 'plugin': return this.executePlugin(item);
      default: return { success: false, message: `未知的执行类型: ${item.type}` };
    }
  }

  private async executeApp(item: ExecutableItem): Promise<ExecutionResult> {
    await invoke('open_app', { app: item.target });
    return { success: true, message: `已启动应用: ${item.name}` };
  }

  private async executeUrl(item: ExecutableItem): Promise<ExecutionResult> {
    let url = item.target;
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      url = 'https://' + url;
    }
    await invoke('open_url', { url });
    return { success: true, message: `已打开链接: ${url}` };
  }

  private async executePath(item: ExecutableItem): Promise<ExecutionResult> {
    await invoke('open_path', { path: item.target });
    return { success: true, message: `已打开路径: ${item.target}` };
  }

  private async executeCommand(item: ExecutableItem): Promise<ExecutionResult> {
    console.log('执行命令:', item.target, item.args);
    return { success: true, message: `已执行命令: ${item.name}` };
  }

  private async executeSetting(item: ExecutableItem): Promise<ExecutionResult> {
    console.log('打开设置:', item.target);
    window.dispatchEvent(new CustomEvent('open-setting', { detail: { target: item.target } }));
    return { success: true, message: `已打开设置: ${item.name}` };
  }

  private async executePlugin(item: ExecutableItem): Promise<ExecutionResult> {
    if (!item.target) {
      return { success: false, message: '无效的插件项：缺少 target (pluginId)' };
    }

    const pluginId = item.target;
    const action = item.args?.[0] || 'default';
    console.log(`[Executor] 🚀 执行插件动作: ${pluginId}.${action}`);

    try {
      const actionResult = await pluginService.executeAction(pluginId, action);
      console.log('[Executor] ✅ 插件执行返回:', actionResult);

      switch (actionResult.type) {
        case 'text': return this.handleTextResult(pluginId, actionResult.message);
        case 'error': return this.handleErrorResult(pluginId, actionResult.message);
        default: return { success: true, message: `✅ 插件 "${pluginId}" 执行完成` };
      }
    } catch (e) {
      console.error('[Executor] ❌ 插件执行异常:', e);
      return { success: false, message: `❌ 插件执行失败 (${pluginId}): ${e}` };
    }
  }

  private handleTextResult(pluginId: string, message: string | undefined): ExecutionResult {
    if (message) {
      console.log('[Executor] 📝 插件返回文本:', message);
      invoke('write_clipboard', { text: message }).catch((e) =>
        console.warn('[Executor] ⚠️ 复制到剪贴板失败:', e)
      );
    }
    return { success: true, message: `✅ 插件 "${pluginId}" 执行成功${message ? ': ' + message.split('\n')[0] : ''}` };
  }

  private handleErrorResult(pluginId: string, message: string | undefined): ExecutionResult {
    console.error('[Executor] ❌ 插件返回错误:', message);
    return { success: false, message: `❌ 插件错误: ${message || '未知错误'}` };
  }

  private async conditionalHide(item: ExecutableItem): Promise<void> {
    if (item.hideWindow !== false) {
      try {
        await this.appWindow.hide();
      } catch (e) {
        console.error('隐藏窗口失败:', e);
      }
    }
  }

  private async recordToHistory(item: ExecutableItem): Promise<void> {
    searchHistory.add(item.name);
  }
}

/** 创建可执行项（供外部/插件使用） */
export function createExecutable(
  item: SearchItem,
  type: ExecutableItem['type'],
  target: string,
  options?: { args?: string[]; hideWindow?: boolean }
): ExecutableItem {
  return { ...item, type, target, args: options?.args, hideWindow: options?.hideWindow };
}

/** 系统内置项工厂 */
export function createSystemItems(): ExecutableItem[] {
  return [
    { id: 'system-settings', name: '设置', description: '打开系统设置', category: '系统', type: 'setting', target: 'general', hideWindow: false },
    { id: 'system-calculator', name: '计算器', description: '打开计算器应用', category: '系统', type: 'app', target: 'calc', hideWindow: true },
    { id: 'system-notepad', name: '记事本', description: '打开记事本', category: '系统', type: 'app', target: 'notepad', hideWindow: true },
    { id: 'system-explorer', name: '文件资源管理器', description: '打开文件资源管理器', category: '系统', type: 'app', target: 'explorer', hideWindow: true },
    { id: 'system-cmd', name: '命令提示符', description: '打开命令提示符', category: '系统', type: 'app', target: 'cmd', hideWindow: true },
    { id: 'folder-documents', name: '文档', description: '打开文档文件夹', category: '系统', type: 'path', target: '%USERPROFILE%\\Documents', hideWindow: true },
    { id: 'folder-downloads', name: '下载', description: '打开下载文件夹', category: '系统', type: 'path', target: '%USERPROFILE%\\Downloads', hideWindow: true },
    { id: 'folder-desktop', name: '桌面', description: '打开桌面文件夹', category: '系统', type: 'path', target: '%USERPROFILE%\\Desktop', hideWindow: true },
    { id: 'web-google', name: 'Google', description: '在浏览器中打开 Google', category: '插件', type: 'url', target: 'https://www.google.com', hideWindow: true },
    { id: 'web-github', name: 'GitHub', description: '在浏览器中打开 GitHub', category: '插件', type: 'url', target: 'https://github.com', hideWindow: true }
  ];
}

export const resultExecutor = new ResultExecutor();
