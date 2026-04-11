/**
 * 存储服务
 * @deprecated 请使用 api.store 代替
 */

import { api } from '$lib/api';

export const save = (key: string, value: unknown) => api.store.save(key, value);
export const load = (key: string) => api.store.load(key);
export const deleteKey = (key: string) => api.store.delete(key);
