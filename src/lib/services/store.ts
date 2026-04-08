import { invoke } from '@tauri-apps/api/core';

/**
 * 存储服务接口
 * 提供键值对的数据持久化能力
 */
export interface StoreService {
  /** 保存数据 */
  save(key: string, value: any): Promise<void>;
  /** 加载数据 */
  load(key: string): Promise<any>;
  /** 删除数据 */
  delete(key: string): Promise<void>;
}

/**
 * Tauri 存储服务实现
 * 基于 tauri-plugin-store 实现数据持久化
 */
class TauriStoreService implements StoreService {
  /**
   * 保存数据到存储
   * @param key - 存储键名
   * @param value - 要存储的值
   */
  async save(key: string, value: any): Promise<void> {
    await invoke('save_to_store', { key, value });
  }

  /**
   * 从存储加载数据
   * @param key - 存储键名
   * @returns 存储的值
   */
  async load(key: string): Promise<any> {
    const result = await invoke<any>('load_from_store', { key });
    return result;
  }

  /**
   * 删除存储的数据
   * @param key - 存储键名
   */
  async delete(key: string): Promise<void> {
    await invoke('delete_from_store', { key });
  }
}

/** 存储服务单例 */
export const storeService = new TauriStoreService();
