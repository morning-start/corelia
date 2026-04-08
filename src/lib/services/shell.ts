import { invoke } from '@tauri-apps/api/core';

/**
 * 在浏览器中打开 URL
 * @param url - 目标 URL
 */
export async function openUrl(url: string): Promise<void> {
  await invoke('open_url', { url });
}

/**
 * 打开指定路径（文件或文件夹）
 * @param path - 目标路径
 */
export async function openPath(path: string): Promise<void> {
  await invoke('open_path', { path });
}

/**
 * 启动指定应用
 * @param app - 应用名称或路径
 */
export async function openApp(app: string): Promise<void> {
  await invoke('open_app', { app });
}
