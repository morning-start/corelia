import { windowService } from '$lib/services/window';
import { historyService } from '$lib/services/history';
import { systemExecutor } from './system';
import { settingExecutor } from './setting';
import { pluginExecutor } from './plugin';
import type { ExecutableItem, ExecutionResult } from './types';

export { createExecutable, createSystemItems } from './types';
export type { ExecutableItem, ExecutionResult } from './types';

/**
 * 结果执行器
 *
 * 职责：
 * - 协调各类型执行器
 * - 记录历史记录
 * - 控制窗口显示/隐藏
 *
 * 注意：此服务仅负责协调，不包含具体的执行逻辑
 * 具体执行逻辑由对应的子执行器负责
 */
class ResultExecutor {
  async execute(item: ExecutableItem): Promise<ExecutionResult> {
    try {
      this.recordToHistory(item);

      const result = await this.executeByType(item);
      await this.conditionalHide(item);
      return result;
    } catch (error) {
      console.error('[ResultExecutor] 执行失败:', error);
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
        await windowService.hide();
      } catch (e) {
        console.error('[ResultExecutor] 隐藏窗口失败:', e);
      }
    }
  }

  private recordToHistory(item: ExecutableItem): void {
    historyService.add(item.name);
  }
}

export const resultExecutor = new ResultExecutor();
