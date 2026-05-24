import { getCurrentWindow } from '@tauri-apps/api/window';
import { searchHistory } from '$lib/stores/history';
import { SystemExecutor } from './system';
import { SettingExecutor } from './setting';
import { PluginExecutor } from './plugin';
import type { ExecutableItem } from './system';

export type { ExecutableItem } from './system';
export { createExecutable, createSystemItems, type SystemExecutor } from './system';
export type { SettingExecutor } from './setting';
export type { PluginExecutor } from './plugin';

export interface ExecutionResult {
  success: boolean;
  message?: string;
}

class ResultExecutor {
  system = new SystemExecutor();
  setting = new SettingExecutor();
  plugin = new PluginExecutor();
  private appWindow = getCurrentWindow();
  private executing = false;

  async execute(item: ExecutableItem): Promise<ExecutionResult> {
    if (this.executing) return { success: false, message: '正在执行中...' };

    try {
      this.executing = true;
      await this.recordToHistory(item);

      const result = await this.executeByType(item);
      await this.conditionalHide(item);
      return result;
    } catch (error) {
      console.error('执行失败:', error);
      return { success: false, message: String(error) };
    } finally {
      this.executing = false;
    }
  }

  private async executeByType(item: ExecutableItem): Promise<ExecutionResult> {
    switch (item.type) {
      case 'app': return this.system.executeApp(item);
      case 'url': return this.system.executeUrl(item);
      case 'path': return this.system.executePath(item);
      case 'command': return this.system.executeCommand(item);
      case 'setting': return this.setting.executeSetting(item);
      case 'plugin': return this.plugin.executePlugin(item);
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