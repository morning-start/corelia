import type { ExecutionResult } from './index';
import type { ExecutableItem } from './system';

export class SettingExecutor {
  async executeSetting(item: ExecutableItem): Promise<ExecutionResult> {
    console.log('打开设置:', item.target);
    window.dispatchEvent(new CustomEvent('open-setting', { detail: { target: item.target } }));
    return { success: true, message: `已打开设置: ${item.name}` };
  }
}