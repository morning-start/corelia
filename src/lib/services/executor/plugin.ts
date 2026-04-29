import { invoke } from '@tauri-apps/api/core';
import { pluginService } from '$lib/plugins/service';
import type { ExecutionResult, ExecutableItem } from './types';

export class PluginExecutor {
  async execute(item: ExecutableItem): Promise<ExecutionResult> {
    if (!item.target) {
      return { success: false, message: '无效的插件项：缺少 target (pluginId)' };
    }

    const pluginId = item.target;
    const action = item.args?.[0] || 'default';
    console.log(`[PluginExecutor] 🚀 执行插件动作: ${pluginId}.${action}`);

    try {
      const actionResult = await pluginService.executeAction(pluginId, action);
      console.log('[PluginExecutor] ✅ 插件执行返回:', actionResult);

      switch (actionResult.type) {
        case 'text': return this.handleTextResult(pluginId, actionResult.message);
        case 'error': return this.handleErrorResult(pluginId, actionResult.message);
        default: return { success: true, message: `✅ 插件 "${pluginId}" 执行完成` };
      }
    } catch (e) {
      console.error('[PluginExecutor] ❌ 插件执行异常:', e);
      return { success: false, message: `❌ 插件执行失败 (${pluginId}): ${e}` };
    }
  }

  private handleTextResult(pluginId: string, message: string | undefined): ExecutionResult {
    if (message) {
      console.log('[PluginExecutor] 📝 插件返回文本:', message);
      invoke('write_clipboard', { text: message }).catch((e) =>
        console.warn('[PluginExecutor] ⚠️ 复制到剪贴板失败:', e)
      );
    }
    return { success: true, message: `✅ 插件 "${pluginId}" 执行成功${message ? ': ' + message.split('\n')[0] : ''}` };
  }

  private handleErrorResult(pluginId: string, message: string | undefined): ExecutionResult {
    console.error('[PluginExecutor] ❌ 插件返回错误:', message);
    return { success: false, message: `❌ 插件错误: ${message || '未知错误'}` };
  }
}

export const pluginExecutor = new PluginExecutor();
