import { invoke } from '@tauri-apps/api/core';

export class ShortcutService {
  /**
   * 注册自定义快捷键
   */
  async register(shortcut: string): Promise<void> {
    await invoke('register_custom_shortcut', { shortcut });
  }

  /**
   * 注销所有快捷键
   */
  async unregisterAll(): Promise<void> {
    await invoke('unregister_all_shortcuts');
  }

  /**
   * 获取当前快捷键
   */
  async getCurrent(): Promise<string | null> {
    return await invoke<string | null>('get_current_shortcut');
  }

  /**
   * 注册默认快捷键
   */
  async registerDefault(): Promise<void> {
    await invoke('register_shortcut_cmd');
  }
}

export const shortcutService = new ShortcutService();
