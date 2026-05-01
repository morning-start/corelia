/**
 * Screenshot 截图增强插件
 *
 * 功能：
 * 1. 快速截图（全屏、区域）
 * 2. 截图历史管理
 * 3. 简单编辑（演示）
 */

// 获取截图历史
function getScreenshotHistory() {
  try {
    var stored = utools.dbStorage.getItem('corelia_screenshots');
    if (stored && Array.isArray(stored)) {
      return stored;
    }
    return [];
  } catch (e) {
    console.error('[screenshot] 读取历史失败:', e);
    return [];
  }
}

// 保存截图历史
function saveScreenshotHistory(history) {
  try {
    utools.dbStorage.setItem('corelia_screenshots', history.slice(0, 20));
    return true;
  } catch (e) {
    console.error('[screenshot] 保存历史失败:', e);
    return false;
  }
}

// 添加到历史
function addToHistory(filename, type) {
  var history = getScreenshotHistory();
  history.unshift({
    filename: filename,
    type: type,
    timestamp: Date.now(),
    id: 'ss_' + Date.now()
  });

  if (history.length > 20) {
    history = history.slice(0, 20);
  }

  saveScreenshotHistory(history);
}

// 插件初始化
function pluginInit() {
  console.log('[screenshot] ✅ 截图插件就绪');

  return { success: true, message: 'Screenshot plugin ready!' };
}

// 搜索函数
function onSearch(query) {
  console.log('[screenshot] 🔍 收到搜索请求:', query);
  var results = [];
  var q = query.toLowerCase().trim();

  var history = getScreenshotHistory();

  // 显示截图功能
  if (!q || q.indexOf('ss') === 0 || q.indexOf('截图') >= 0 || q.indexOf('screenshot') >= 0) {
    results.push({
      title: '📷 区域截图',
      description: '选择屏幕区域截图',
      icon: '📷',
      action: 'areaScreenshot'
    });

    results.push({
      title: '🖥️ 全屏截图',
      description: '截取整个屏幕',
      icon: '🖥️',
      action: 'fullScreenshot'
    });

    results.push({
      title: '📋 截图历史',
      description: '查看最近的截图记录',
      icon: '📋',
      action: 'showHistory'
    });
  }

  // 显示最近的截图
  if (history.length > 0) {
    for (var i = 0; i < Math.min(history.length, 3); i++) {
      var item = history[i];
      results.push({
        title: '⏰ ' + item.filename,
        description: new Date(item.timestamp).toLocaleString() + ' | ' + item.type,
        icon: '📷',
        action: 'openScreenshot',
        data: { item: item }
      });
    }
  }

  return results.slice(0, 20);
}

// 执行动作函数
function onAction(action, data) {
  console.log('[screenshot] ⚡ 执行动作:', action, data);

  switch (action) {
    case 'areaScreenshot':
      var filename = 'area_screenshot_' + new Date().toISOString().replace(/[:.]/g, '-') + '.png';
      addToHistory(filename, 'Area');

      return {
        type: 'text',
        message: '📷 区域截图\n\n⚠️ 提示: 实际截图功能需要后端 API 支持\n\n已记录到历史: ' + filename
      };

    case 'fullScreenshot':
      var filename = 'full_screenshot_' + new Date().toISOString().replace(/[:.]/g, '-') + '.png';
      addToHistory(filename, 'Full');

      return {
        type: 'text',
        message: '🖥️ 全屏截图\n\n⚠️ 提示: 实际截图功能需要后端 API 支持\n\n已记录到历史: ' + filename
      };

    case 'showHistory':
      var history = getScreenshotHistory();
      var list = history.map(function(item, index) {
        return (index + 1) + '. ' + item.filename + ' (' + item.type + ')';
      }).join('\n');

      return {
        type: 'text',
        message: '📋 截图历史 (共 ' + history.length + ' 条)\n\n' + (history.length > 0 ? list : '暂无截图记录')
      };

    case 'openScreenshot':
      try {
        var item = data.item;
        return {
          type: 'text',
          message: '📷 打开截图\n\n文件名: ' + item.filename + '\n时间: ' + new Date(item.timestamp).toLocaleString() + '\n\n⚠️ 提示: 实际打开功能需要文件系统支持'
        };
      } catch (e) {
        return { type: 'error', message: '❌ 打开失败: ' + e.message };
      }

    default:
      return { type: 'error', message: '❓ 未知动作: ' + action };
  }
}

// 导出
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { pluginInit, onSearch, onAction };
}
