/**
 * 窗口管理服务
 *
 * 职责：
 * - 窗口显示/隐藏/切换
 * - 窗口置顶
 * - 窗口位置管理
 *
 * 注意：此服务仅负责窗口控制逻辑，不包含 UI 状态管理
 */

import { getCurrentWindow } from '@tauri-apps/api/window';

class WindowService {
  private appWindow = getCurrentWindow();

  /**
   * 显示窗口并置顶
   */
  async show(): Promise<void> {
    try {
      await this.appWindow.show();
      await this.appWindow.setFocus();
      console.log('[WindowService] 窗口已显示');
    } catch (e) {
      console.error('[WindowService] 显示窗口失败:', e);
      throw e;
    }
  }

  /**
   * 隐藏窗口
   */
  async hide(): Promise<void> {
    try {
      await this.appWindow.hide();
      console.log('[WindowService] 窗口已隐藏');
    } catch (e) {
      console.error('[WindowService] 隐藏窗口失败:', e);
      throw e;
    }
  }

  /**
   * 切换窗口显示/隐藏状态
   */
  async toggle(): Promise<void> {
    try {
      const isVisible = await this.appWindow.isVisible();
      if (isVisible) {
        await this.hide();
      } else {
        await this.show();
      }
    } catch (e) {
      console.error('[WindowService] 切换窗口状态失败:', e);
      throw e;
    }
  }

  /**
   * 获取窗口是否可见
   */
  async isVisible(): Promise<boolean> {
    return await this.appWindow.isVisible();
  }

  /**
   * 设置窗口置顶
   */
  async setAlwaysOnTop(alwaysOnTop: boolean): Promise<void> {
    try {
      await this.appWindow.setAlwaysOnTop(alwaysOnTop);
      console.log(`[WindowService] 窗口置顶: ${alwaysOnTop}`);
    } catch (e) {
      console.error('[WindowService] 设置窗口置顶失败:', e);
      throw e;
    }
  }

  /**
   * 居中显示窗口
   */
  async center(): Promise<void> {
    try {
      await this.appWindow.center();
      console.log('[WindowService] 窗口已居中');
    } catch (e) {
      console.error('[WindowService] 居中窗口失败:', e);
      throw e;
    }
  }
}

/** 窗口管理服务单例 */
export const windowService = new WindowService();
export default windowService;
