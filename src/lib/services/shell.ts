/**
 * Shell 服务
 * @deprecated 请使用 api.shell 代替
 */

import { api } from '$lib/api';

export const openUrl = (url: string) => api.shell.openUrl(url);
export const openPath = (path: string) => api.shell.openPath(path);
export const openApp = (app: string) => api.shell.openApp(app);
