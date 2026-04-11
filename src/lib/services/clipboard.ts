/**
 * 剪贴板服务
 * @deprecated 请使用 api.clipboard 代替
 */

import { api } from '$lib/api';

export const readClipboard = () => api.clipboard.read();
export const writeClipboard = (text: string) => api.clipboard.write(text);
