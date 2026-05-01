/**
 * App Launcher 应用启动器插件
 *
 * 功能：
 * 1. 快速启动常用应用程序
 * 2. 管理自定义应用列表
 * 3. 支持应用分类
 */

// 默认应用列表
var DEFAULT_APPS = [
  { id: '1', name: 'Chrome 浏览器', path: 'chrome', icon: '🌐', category: '网络' },
  { id: '2', name: 'VS Code', path: 'code', icon: '💻', category: '开发' },
  { id: '3', name: 'Terminal', path: 'terminal', icon: '⌨️', category: '系统' },
  { id: '4', name: 'File Explorer', path: 'explorer', icon: '📁', category: '系统' },
  { id: '5', name: 'Settings', path: 'settings', icon: '⚙️', category: '系统' },
  { id: '6', name: 'Calculator', path: 'calc', icon: '🔢', category: '工具' },
  { id: '7', name: 'Notepad', path: 'notepad', icon: '📝', category: '工具' },
  { id: '8', name: 'Paint', path: 'mspaint', icon: '🎨', category: '工具' }
];

// 获取应用列表
function getApps() {
  try {
    var stored = utools.dbStorage.getItem('corelia_apps');
    if (stored && Array.isArray(stored) && stored.length > 0) {
      return stored;
    }
    return DEFAULT_APPS;
  } catch (e) {
    console.error('[app-launcher] 读取失败:', e);
    return DEFAULT_APPS;
  }
}

// 保存应用列表
function saveApps(apps) {
  try {
    utools.dbStorage.setItem('corelia_apps', apps);
    return true;
  } catch (e) {
    console.error('[app-launcher] 保存失败:', e);
    return false;
  }
}

// 获取最近使用记录
function getRecentApps() {
  try {
    var stored = utools.dbStorage.getItem('corelia_apps_recent');
    if (stored && Array.isArray(stored)) {
      return stored;
    }
    return [];
  } catch (e) {
    console.error('[app-launcher] 读取最近使用失败:', e);
    return [];
  }
}

// 保存最近使用记录
function saveRecentApps(recent) {
  try {
    utools.dbStorage.setItem('corelia_apps_recent', recent.slice(0, 10));
    return true;
  } catch (e) {
    console.error('[app-launcher] 保存最近使用失败:', e);
    return false;
  }
}

// 添加到最近使用
function addToRecent(app) {
  var recent = getRecentApps();

  // 移除已存在的
  recent = recent.filter(function(item) { return item.id !== app.id; });

  // 添加到开头
  recent.unshift({
    id: app.id,
    name: app.name,
    timestamp: Date.now()
  });

  // 限制数量
  if (recent.length > 10) {
    recent = recent.slice(0, 10);
  }

  saveRecentApps(recent);
}

// 插件初始化
function pluginInit() {
  console.log('[app-launcher] ✅ 应用启动器就绪');

  // 初始化默认应用
  try {
    if (!utools.dbStorage.getItem('corelia_apps')) {
      saveApps(DEFAULT_APPS);
      console.log('[app-launcher] ✅ 初始化默认应用列表');
    }
  } catch (e) {
    console.error('[app-launcher] 初始化失败:', e);
  }

  return { success: true, message: 'App Launcher ready!' };
}

// 搜索函数
function onSearch(query) {
  console.log('[app-launcher] 🔍 收到搜索请求:', query);
  var results = [];
  var q = query.toLowerCase().trim();

  var apps = getApps();
  var recent = getRecentApps();

  // 显示最近使用（优先）
  if (recent.length > 0 && (!q || q.indexOf('app') === 0 || q.indexOf('应用') >= 0)) {
    for (var i = 0; i < Math.min(recent.length, 3); i++) {
      var recentItem = recent[i];
      var app = apps.find(function(a) { return a.id === recentItem.id; });
      if (app) {
        results.push({
          title: '⏰ ' + app.icon + ' ' + app.name,
          description: '最近使用 | ' + (app.category || '未分类'),
          icon: app.icon,
          action: 'launchApp',
          data: { app: app }
        });
      }
    }
  }

  // 显示所有匹配的应用
  for (var j = 0; j < apps.length; j++) {
    var app = apps[j];
    var match = !q || q === '' ||
      app.name.toLowerCase().indexOf(q) !== -1 ||
      app.path.toLowerCase().indexOf(q) !== -1 ||
      (app.category && app.category.toLowerCase().indexOf(q) !== -1);

    if (match) {
      results.push({
        title: app.icon + ' ' + app.name,
        description: (app.category ? '[' + app.category + '] ' : '') + app.path,
        icon: app.icon,
        action: 'launchApp',
        data: { app: app }
      });
    }
  }

  // 添加管理功能
  if (!q || q.indexOf('app') === 0 || q.indexOf('应用') >= 0 || q.indexOf('启动') >= 0) {
    results.push({
      title: '➕ 添加新应用',
      description: '添加自定义应用程序到列表',
      icon: '➕',
      action: 'addApp'
    });

    results.push({
      title: '📋 管理应用',
      description: '查看和管理所有应用程序',
      icon: '📋',
      action: 'manageApps'
    });
  }

  return results.slice(0, 20);
}

// 执行动作函数
function onAction(action, data) {
  console.log('[app-launcher] ⚡ 执行动作:', action, data);

  switch (action) {
    case 'launchApp':
      try {
        var app = data.app;
        addToRecent(app);

        return {
          type: 'text',
          message: '🚀 启动: ' + app.name + '\n\n应用路径: ' + app.path + '\n\n⚠️ 提示: 完整执行需要后端 Shell API 支持',
          timestamp: new Date().toISOString()
        };
      } catch (e) {
        return { type: 'error', message: '❌ 启动失败: ' + e.message };
      }

    case 'addApp':
      return {
        type: 'text',
        message: '➕ 添加新应用\n\n格式:\n• 名称: 显示名称\n• 路径: 执行路径/命令\n• 图标: Emoji 图标\n• 分类: 可选分类\n\n当前应用: ' + getApps().map(function(a) {
          return '\n  ' + a.icon + ' ' + a.name + ' - ' + a.path;
        }).join('')
      };

    case 'manageApps':
      var allApps = getApps();
      var list = allApps.map(function(a) {
        return '• ' + a.icon + ' ' + a.name + ' (' + a.path + ')' + (a.category ? ' [' + a.category + ']' : '');
      }).join('\n');
      return {
        type: 'text',
        message: '📋 应用管理\n\n共 ' + allApps.length + ' 个应用:\n\n' + list
      };

    default:
      return { type: 'error', message: '❓ 未知动作: ' + action };
  }
}

// 导出
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { pluginInit, onSearch, onAction };
}
