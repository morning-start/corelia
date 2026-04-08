import { invoke } from '@tauri-apps/api/core';

/**
 * 开机启动服务接口
 */
export interface StartupService {
  /** 启用开机自启动 */
  enable(): Promise<void>;
  /** 禁用开机自启动 */
  disable(): Promise<void>;
  /** 检查是否已启用 */
  isEnabled(): Promise<boolean>;
}

/**
 * Tauri 开机启动服务实现
 * 通过后端命令控制应用的开机自启动功能
 */
class TauriStartupService implements StartupService {
  /** 启用开机自启动 */
  async enable(): Promise<void> {
    await invoke('enable_autostart');
  }

  /** 禁用开机自启动 */
  async disable(): Promise<void> {
    await invoke('disable_autostart');
  }

  /** 检查是否已启用 */
  async isEnabled(): Promise<boolean> {
    return await invoke<boolean>('is_autostart_enabled');
  }
}

/** 开机启动服务单例 */
export const startupService = new TauriStartupService();