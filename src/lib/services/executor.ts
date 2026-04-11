import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { searchHistory } from '$lib/stores/history';
import { pluginService } from '$lib/plugins/service';
import type { SearchItem } from '$lib/search/fuzzy';

/**
 * 可执行的搜索项接口
 * 扩展基础 SearchItem，添加执行能力
 */
export interface ExecutableItem extends SearchItem {
  /** 执行类型 */
  type: 'app' | 'url' | 'path' | 'command' | 'setting' | 'plugin';
  /** 执行目标（应用名、URL、路径等） */
  target: string;
  /** 执行参数（可选） */
  args?: string[];
  /** 执行后是否隐藏窗口 */
  hideWindow?: boolean;
}

/**
 * 执行结果
 */
export interface ExecutionResult {
  success: boolean;
  message?: string;
}

/**
 * 结果执行器服务
 * 处理不同类型搜索结果的执行逻辑
 */
class ResultExecutor {
  private appWindow = getCurrentWindow();

  /**
   * 执行搜索项
   * @param item - 要执行的搜索项
   * @returns 执行结果
   */
  async execute(item: ExecutableItem): Promise<ExecutionResult> {
    try {
      // 记录到搜索历史
      await this.recordToHistory(item);

      // 根据类型执行不同操作
      switch (item.type) {
        case 'app':
          return await this.executeApp(item);
        case 'url':
          return await this.executeUrl(item);
        case 'path':
          return await this.executePath(item);
        case 'command':
          return await this.executeCommand(item);
        case 'setting':
          return await this.executeSetting(item);
        case 'plugin':
          return await this.executePlugin(item);
        default:
          return { success: false, message: `未知的执行类型: ${item.type}` };
      }
    } catch (error) {
      console.error('执行失败:', error);
      return { success: false, message: String(error) };
    }
  }

  /**
   * 执行应用启动
   */
  private async executeApp(item: ExecutableItem): Promise<ExecutionResult> {
    await invoke('open_app', { app: item.target });

    if (item.hideWindow !== false) {
      await this.hideWindow();
    }

    return { success: true, message: `已启动应用: ${item.name}` };
  }

  /**
   * 执行 URL 打开
   */
  private async executeUrl(item: ExecutableItem): Promise<ExecutionResult> {
    let url = item.target;

    // 自动添加协议前缀
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      url = 'https://' + url;
    }

    await invoke('open_url', { url });

    if (item.hideWindow !== false) {
      await this.hideWindow();
    }

    return { success: true, message: `已打开链接: ${url}` };
  }

  /**
   * 执行路径打开
   */
  private async executePath(item: ExecutableItem): Promise<ExecutionResult> {
    await invoke('open_path', { path: item.target });

    if (item.hideWindow !== false) {
      await this.hideWindow();
    }

    return { success: true, message: `已打开路径: ${item.target}` };
  }

  /**
   * 执行命令
   */
  private async executeCommand(item: ExecutableItem): Promise<ExecutionResult> {
    // TODO: 实现命令执行逻辑，可能需要调用后端执行系统命令
    console.log('执行命令:', item.target, item.args);

    if (item.hideWindow !== false) {
      await this.hideWindow();
    }

    return { success: true, message: `已执行命令: ${item.name}` };
  }

  /**
   * 执行设置项
   */
  private async executeSetting(item: ExecutableItem): Promise<ExecutionResult> {
    // 设置项通常不隐藏窗口，而是打开设置面板
    console.log('打开设置:', item.target);

    // 可以发送事件通知设置面板打开特定设置项
    window.dispatchEvent(new CustomEvent('open-setting', {
      detail: { target: item.target }
    }));

    return { success: true, message: `已打开设置: ${item.name}` };
  }

  /**
   * 执行插件功能
   *
   * 执行流程：
   * 1. 从 item.target 获取插件 ID
   * 2. 从 item.args[0] 获取动作名称（action）
   * 3. 调用 pluginService.executeAction() 执行插件动作
   * 4. 根据返回结果类型进行相应处理：
   *    - type: 'text' → 显示文本消息（可复制到剪贴板或显示通知）
   *    - type: 'error' → 显示错误信息
   *    - 其他类型 → 记录日志
   *
   * @param item - 插件类型的搜索项（必须包含 target 和 args）
   * @returns 执行结果
   */
  private async executePlugin(item: ExecutableItem): Promise<ExecutionResult> {
    if (!item.target) {
      return { success: false, message: '无效的插件项：缺少 target (pluginId)' };
    }

    const pluginId = item.target;
    const action = item.args?.[0] || 'default';

    console.log(`[Executor] 🚀 执行插件动作: ${pluginId}.${action}`);

    try {
      // 调用插件的动作执行方法
      const actionResult = await pluginService.executeAction(pluginId, action);

      console.log('[Executor] ✅ 插件执行返回:', actionResult);

      // 处理不同类型的返回值
      switch (actionResult.type) {
        case 'text':
          // 文本类型：显示消息并复制到剪贴板
          if (actionResult.message) {
            console.log('[Executor] 📝 插件返回文本:', actionResult.message);

            // 尝试复制到剪贴板
            try {
              await invoke('write_clipboard', { text: actionResult.message });
              console.log('[Executor] 📋 已复制到剪贴板');
            } catch (clipboardError) {
              console.warn('[Executor] ⚠️ 复制到剪贴板失败:', clipboardError);
            }
          }

          if (item.hideWindow !== false) {
            await this.hideWindow();
          }

          return {
            success: true,
            message: `✅ 插件 "${pluginId}" 执行成功${actionResult.message ? ': ' + actionResult.message.split('\n')[0] : ''}`
          };

        case 'error':
          // 错误类型：记录错误并显示提示
          console.error('[Executor] ❌ 插件返回错误:', actionResult.message);

          if (item.hideWindow !== false) {
            await this.hideWindow();
          }

          return {
            success: false,
            message: `❌ 插件错误: ${actionResult.message || '未知错误'}`
          };

        default:
          // 其他类型：记录日志
          console.log('[Executor] ℹ️ 插件返回未知类型:', actionResult);

          if (item.hideWindow !== false) {
            await this.hideWindow();
          }

          return {
            success: true,
            message: `✅ 插件 "${pluginId}" 执行完成`
          };
      }

    } catch (e) {
      console.error('[Executor] ❌ 插件执行异常:', e);

      if (item.hideWindow !== false) {
        await this.hideWindow();
      }

      return {
        success: false,
        message: `❌ 插件执行失败 (${pluginId}): ${e}`
      };
    }
  }

  /**
   * 隐藏主窗口
   */
  private async hideWindow(): Promise<void> {
    try {
      await this.appWindow.hide();
    } catch (e) {
      console.error('隐藏窗口失败:', e);
    }
  }

  /**
   * 记录到搜索历史
   */
  private async recordToHistory(item: ExecutableItem): Promise<void> {
    // 使用 item.name 作为历史记录的关键词
    searchHistory.add(item.name);
  }

  /**
   * 将基础 SearchItem 转换为 ExecutableItem
   * @param item - 基础搜索项
   * @param type - 执行类型
   * @param target - 执行目标
   * @returns 可执行的搜索项
   */
  createExecutable(
    item: SearchItem,
    type: ExecutableItem['type'],
    target: string,
    options?: { args?: string[]; hideWindow?: boolean }
  ): ExecutableItem {
    return {
      ...item,
      type,
      target,
      args: options?.args,
      hideWindow: options?.hideWindow
    };
  }
}

/**
 * 创建系统内置的可执行项
 */
export function createSystemItems(): ExecutableItem[] {
  return [
    {
      id: 'system-settings',
      name: '设置',
      description: '打开系统设置',
      category: '系统',
      type: 'setting',
      target: 'general',
      hideWindow: false
    },
    {
      id: 'system-calculator',
      name: '计算器',
      description: '打开计算器应用',
      category: '系统',
      type: 'app',
      target: 'calc',
      hideWindow: true
    },
    {
      id: 'system-notepad',
      name: '记事本',
      description: '打开记事本',
      category: '系统',
      type: 'app',
      target: 'notepad',
      hideWindow: true
    },
    {
      id: 'system-explorer',
      name: '文件资源管理器',
      description: '打开文件资源管理器',
      category: '系统',
      type: 'app',
      target: 'explorer',
      hideWindow: true
    },
    {
      id: 'system-cmd',
      name: '命令提示符',
      description: '打开命令提示符',
      category: '系统',
      type: 'app',
      target: 'cmd',
      hideWindow: true
    },
    {
      id: 'folder-documents',
      name: '文档',
      description: '打开文档文件夹',
      category: '系统',
      type: 'path',
      target: '%USERPROFILE%\\Documents',
      hideWindow: true
    },
    {
      id: 'folder-downloads',
      name: '下载',
      description: '打开下载文件夹',
      category: '系统',
      type: 'path',
      target: '%USERPROFILE%\\Downloads',
      hideWindow: true
    },
    {
      id: 'folder-desktop',
      name: '桌面',
      description: '打开桌面文件夹',
      category: '系统',
      type: 'path',
      target: '%USERPROFILE%\\Desktop',
      hideWindow: true
    },
    {
      id: 'web-google',
      name: 'Google',
      description: '在浏览器中打开 Google',
      category: '插件',
      type: 'url',
      target: 'https://www.google.com',
      hideWindow: true
    },
    {
      id: 'web-github',
      name: 'GitHub',
      description: '在浏览器中打开 GitHub',
      category: '插件',
      type: 'url',
      target: 'https://github.com',
      hideWindow: true
    }
  ];
}

// 导出单例
export const resultExecutor = new ResultExecutor();
