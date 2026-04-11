/**
 * 快捷键服务
 * @deprecated 请使用 api.shortcut 代替
 */

import { api } from '$lib/api';

export class ShortcutService {
  async register(shortcut: string): Promise<void> {
    await api.shortcut.register(shortcut);
  }

  async unregisterAll(): Promise<void> {
    await api.shortcut.unregisterAll();
  }

  async getCurrent(): Promise<string | null> {
    return await api.shortcut.getCurrent();
  }

  async registerDefault(): Promise<void> {
    await api.shortcut.registerDefault();
  }
}

export const shortcutService = new ShortcutService();
