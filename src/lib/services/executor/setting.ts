import type { ExecutionResult, ExecutableItem } from './types';

export class SettingExecutor {
  async execute(item: ExecutableItem): Promise<ExecutionResult> {
    console.log('打开设置:', item.target);
    window.dispatchEvent(new CustomEvent('open-setting', { detail: { target: item.target } }));
    return { success: true, message: `已打开设置: ${item.name}` };
  }
}

export const settingExecutor = new SettingExecutor();
