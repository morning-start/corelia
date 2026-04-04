import { invoke } from '@tauri-apps/api/core';

export interface StoreService {
  save(key: string, value: any): Promise<void>;
  load(key: string): Promise<any>;
  delete(key: string): Promise<void>;
}

class TauriStoreService implements StoreService {
  async save(key: string, value: any): Promise<void> {
    await invoke('save_to_store', { key, value });
  }

  async load(key: string): Promise<any> {
    const result = await invoke<any>('load_from_store', { key });
    return result;
  }

  async delete(key: string): Promise<void> {
    await invoke('delete_from_store', { key });
  }
}

export const storeService = new TauriStoreService();
