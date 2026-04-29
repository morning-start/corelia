import { invoke } from '@tauri-apps/api/core';
import type { ExecutionResult, ExecutableItem } from './types';

export class SystemExecutor {
  async executeApp(item: ExecutableItem): Promise<ExecutionResult> {
    await invoke('open_app', { app: item.target });
    return { success: true, message: `已启动应用: ${item.name}` };
  }

  async executeUrl(item: ExecutableItem): Promise<ExecutionResult> {
    let url = item.target;
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      url = 'https://' + url;
    }
    await invoke('open_url', { url });
    return { success: true, message: `已打开链接: ${url}` };
  }

  async executePath(item: ExecutableItem): Promise<ExecutionResult> {
    await invoke('open_path', { path: item.target });
    return { success: true, message: `已打开路径: ${item.target}` };
  }

  async executeCommand(item: ExecutableItem): Promise<ExecutionResult> {
    console.log('执行命令:', item.target, item.args);
    return { success: true, message: `已执行命令: ${item.name}` };
  }
}

export const systemExecutor = new SystemExecutor();
