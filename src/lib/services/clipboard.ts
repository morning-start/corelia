import { invoke } from '@tauri-apps/api/core';

/**
 * 读取剪贴板内容
 * @returns 剪贴板文本内容
 */
export async function readClipboard(): Promise<string> {
  return await invoke<string>('read_clipboard');
}

/**
 * 写入剪贴板内容
 * @param text - 要写入的文本
 */
export async function writeClipboard(text: string): Promise<void> {
  await invoke('write_clipboard', { text });
}
