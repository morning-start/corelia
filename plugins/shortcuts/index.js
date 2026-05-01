/**
 * Shortcuts 快捷命令插件
 *
 * 功能：
 * 1. 保存自定义快捷命令
 * 2. 快速执行常用命令
 * 3. 支持命令分类
 */

// 默认命令
var DEFAULT_SHORTCUTS = [
  { id: '1', name: '打开终端', cmd: 'terminal', icon: '💻', category: '系统' },
  { id: '2', name: '打开文件浏览器', cmd: 'explorer', icon: '📁', category: '系统' },
  { id: '3', name: '打开任务管理器', cmd: 'taskmgr', icon: '📊', category: '系统' }
];

// 获取保存的命令
function getShortcuts() {
  try {
    var stored = utools.dbStorage.getItem('corelia_shortcuts');
    if (stored && stored.length > 0) {
      return stored;
    }
    return DEFAULT_SHORTCUTS;
  } catch (e) {
    console.error('[shortcuts] 读取失败:', e);
    return DEFAULT_SHORTCUTS;
  }
}

// 保存命令
function saveShortcuts(shortcuts) {
  try {
    utools.dbStorage.setItem('corelia_shortcuts', shortcuts);
    return true;
  } catch (e) {
    console.error('[shortcuts] 保存失败:', e);
    return false;
  }
}

// 插件初始化
function pluginInit() {
  console.log('[shortcuts] ✅ 快捷命令插件就绪');

  // 初始化默认命令
  try {
    if (!utools.dbStorage.getItem('corelia_shortcuts')) {
      saveShortcuts(DEFAULT_SHORTCUTS);
      console.log('[shortcuts] ✅ 初始化默认快捷命令');
    }
  } catch (e) {
    console.error('[shortcuts] 初始化失败:', e);
  }

  return { success: true, message: 'Shortcuts plugin ready!' };
}

// 搜索函数
function onSearch(query) {
  console.log('[shortcuts] 🔍 收到搜索请求:', query);
  var results = [];
  var q = query.toLowerCase().trim();

  var shortcuts = getShortcuts();

  // 显示所有快捷命令
  for (var i = 0; i < shortcuts.length; i++) {
    var sc = shortcuts[i];
    var match = !q || q === '' ||
      sc.name.toLowerCase().indexOf(q) !== -1 ||
      sc.cmd.toLowerCase().indexOf(q) !== -1 ||
      (sc.category && sc.category.toLowerCase().indexOf(q) !== -1);

    if (match) {
      results.push({
        title: sc.icon + ' ' + sc.name,
        description: sc.cmd + (sc.category ? ' [' + sc.category + ']' : ''),
        icon: sc.icon || '⌨️',
        action: 'runShortcut',
        data: { shortcut: sc }
      });
    }
  }

  // 添加管理命令
  if (!q || q.indexOf('sc') === 0 || q.indexOf('快捷') >= 0 || q.indexOf('命令') >= 0) {
    results.push({
      title: '➕ 添加新命令',
      description: '保存一个新的快捷命令',
      icon: '✨',
      action: 'addCmd'
    });

    results.push({
      title: '📋 管理命令',
      description: '查看和编辑所有快捷命令',
      icon: '⚙️',
      action: 'manageCmds'
    });
  }

  return results.slice(0, 20);
}

// 执行动作函数
function onAction(action, data) {
  console.log('[shortcuts] ⚡ 执行动作:', action, data);

  switch (action) {
    case 'runShortcut':
      try {
        var shortcut = data.shortcut;
        // 尝试执行命令（在实际环境中会有 shell API）
        return {
          type: 'text',
          message: '✅ 执行: ' + shortcut.name + '\n\n命令: ' + shortcut.cmd + '\n\n⚠️ 提示: 完整执行需要后端 Shell API 支持',
          timestamp: new Date().toISOString()
        };
      } catch (e) {
        return { type: 'error', message: '❌ 执行失败: ' + e.message };
      }

    case 'addCmd':
      return {
        type: 'text',
        message: '➕ 添加新命令\n\n格式:\n• 名称: 显示名称\n• 命令: 实际执行命令\n• 图标: Emoji 图标\n\n当前命令列表: ' + getShortcuts().map(function(s) {
          return '\n  ' + s.icon + ' ' + s.name + ' - ' + s.cmd;
        }).join('')
      };

    case 'manageCmds':
      var all = getShortcuts();
      var list = all.map(function(s) {
        return '• ' + s.icon + ' ' + s.name + ' (' + s.cmd + ')';
      }).join('\n');
      return {
        type: 'text',
        message: '📋 快捷命令管理\n\n共 ' + all.length + ' 个命令:\n\n' + list
      };

    default:
      return { type: 'error', message: '❓ 未知动作: ' + action };
  }
}

// 导出
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { pluginInit, onSearch, onAction };
}
