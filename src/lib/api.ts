/**
 * Tauri API 统一代理层
 *
 * 提供类型安全的 invoke 调用封装
 */

import { invoke } from '@tauri-apps/api/core';

export const api = {
  clipboard: {
    read: () => invoke<string>('read_clipboard'),
    write: (text: string) => invoke('write_clipboard', { text }),
  },

  shell: {
    openUrl: (url: string) => invoke('open_url', { url }),
    openPath: (path: string) => invoke('open_path', { path }),
    openApp: (app: string) => invoke('open_app', { app }),
  },

  store: {
    save: (key: string, value: unknown) => invoke('save_to_store', { key, value }),
    load: (key: string) => invoke('load_from_store', { key }),
    delete: (key: string) => invoke('delete_from_store', { key }),
  },

  shortcut: {
    register: (shortcut: string) => invoke('register_custom_shortcut', { shortcut }),
    unregisterAll: () => invoke('unregister_all_shortcuts'),
    getCurrent: () => invoke<string | null>('get_current_shortcut'),
    registerDefault: () => invoke('register_shortcut_cmd'),
  },
};
