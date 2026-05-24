import { invoke } from '@tauri-apps/api/core';
import type { SearchItem } from '$lib/search/fuzzy';
import type { ExecutionResult } from './index';

export interface ExecutableItem extends SearchItem {
  type: 'app' | 'url' | 'path' | 'command' | 'setting' | 'plugin';
  target: string;
  args?: string[];
  hideWindow?: boolean;
}

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
    try {
      await invoke('open_app', { app: item.target });
      return { success: true, message: `已执行命令: ${item.name}` };
    } catch (error) {
      console.error('命令执行失败:', item.target, error);
      return { success: false, message: `命令执行失败: ${error}` };
    }
  }
}

export function createExecutable(
  item: SearchItem,
  type: ExecutableItem['type'],
  target: string,
  options?: { args?: string[]; hideWindow?: boolean }
): ExecutableItem {
  return { ...item, type, target, args: options?.args, hideWindow: options?.hideWindow };
}

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