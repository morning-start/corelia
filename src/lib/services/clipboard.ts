import { invoke } from '@tauri-apps/api/core';

export async function readClipboard(): Promise<string> {
  return await invoke<string>('read_clipboard');
}

export async function writeClipboard(text: string): Promise<void> {
  await invoke('write_clipboard', { text });
}
