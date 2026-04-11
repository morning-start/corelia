/**
 * 插件数据存储服务
 * 为每个插件提供隔离的数据存储空间
 *
 * 功能：
 * - 获取插件数据目录路径
 * - 读取/写入/删除插件私有数据
 * - 清除插件所有数据
 * - 查询插件数据大小
 *
 * 数据隔离：每个插件使用独立的 data/{plugin_id}/ 目录
 * 配额限制：单个插件默认 10MB 上限（后端强制执行）
 */

import { invoke } from '@tauri-apps/api/core';

class PluginStoreService {
    /**
     * 获取插件的数据目录路径
     * @param pluginId 插件 ID
     * @returns 绝对路径字符串（目录已自动创建）
     */
    async getDataPath(pluginId: string): Promise<string> {
        return await invoke<string>('get_plugin_data_path', { pluginId });
    }

    /**
     * 读取插件数据
     * @param pluginId 插件 ID
     * @param key 数据键名
     * @returns 数据值（任意 JSON 类型）
     * @throws 当 key 不存在时抛出错误
     */
    async readData<T = unknown>(pluginId: string, key: string): Promise<T> {
        return await invoke<T>('read_plugin_data', { pluginId, key });
    }

    /**
     * 写入插件数据
     * @param pluginId 插件 ID
     * @param key 数据键名
     * @param value 数据值（支持任意 JSON 可序列化值）
     * @throws 当超出配额限制时抛出错误
     */
    async writeData(pluginId: string, key: string, value: unknown): Promise<void> {
        await invoke('write_plugin_data', { pluginId, key, value });
    }

    /**
     * 删除插件数据的某个 key
     * @param pluginId 插件 ID
     * @param key 要删除的数据键名
     */
    async deleteData(pluginId: string, key: string): Promise<void> {
        await invoke('delete_plugin_data', { pluginId, key });
    }

    /**
     * 清除插件的所有数据（危险操作）
     * 会删除整个 data/{plugin_id}/ 目录及其内容
     * @param pluginId 插件 ID
     */
    async clearData(pluginId: string): Promise<void> {
        await invoke('clear_plugin_data', { pluginId });
    }

    /**
     * 获取插件数据大小（字节）
     * @param pluginId 插件 ID
     * @returns 数据目录的总大小（字节），无数据时返回 0
     */
    async getDataSize(pluginId: string): Promise<number> {
        return await invoke<number>('get_plugin_data_size', { pluginId });
    }
}

/** 插件存储服务单例实例 */
export const pluginStore = new PluginStoreService();
export default pluginStore;
