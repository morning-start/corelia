import { getCurrentWindow } from '@tauri-apps/api/window';
import { searchHistory } from '$lib/stores/history';
import { systemExecutor } from './system';
import { settingExecutor } from './setting';
import { pluginExecutor } from './plugin';
import type { ExecutableItem, ExecutionResult } from './types';

export { createExecutable, createSystemItems } from './types';
export type { ExecutableItem, ExecutionResult } from './types';

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
      case 'app': return systemExecutor.executeApp(item);
      case 'url': return systemExecutor.executeUrl(item);
      case 'path': return systemExecutor.executePath(item);
      case 'command': return systemExecutor.executeCommand(item);
      case 'setting': return settingExecutor.execute(item);
      case 'plugin': return pluginExecutor.execute(item);
      default: return { success: false, message: `未知的执行类型: ${item.type}` };
    }
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

export const resultExecutor = new ResultExecutor();
