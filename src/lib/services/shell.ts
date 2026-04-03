import { invoke } from '@tauri-apps/api/core';

export async function openUrl(url: string): Promise<void> {
  await invoke('open_url', { url });
}

export async function openPath(path: string): Promise<void> {
  await invoke('open_path', { path });
}

export async function openApp(app: string): Promise<void> {
  await invoke('open_app', { app });
}
