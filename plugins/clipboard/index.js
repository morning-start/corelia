/**
 * Clipboard 剪贴板增强插件
 *
 * 功能：
 * 1. 剪贴板历史记录
 * 2. 格式转换（大小写、Base64、URL 编码等）
 * 3. 剪贴板内容预览
 */

// 剪贴板历史记录限制
var MAX_HISTORY = 50;

// 获取剪贴板历史
function getClipboardHistory() {
  try {
    var stored = utools.dbStorage.getItem('corelia_clipboard_history');
    if (stored && Array.isArray(stored)) {
      return stored;
    }
    return [];
  } catch (e) {
    console.error('[clipboard] 读取历史失败:', e);
    return [];
  }
}

// 保存剪贴板历史
function saveClipboardHistory(history) {
  try {
    utools.dbStorage.setItem('corelia_clipboard_history', history.slice(0, MAX_HISTORY));
    return true;
  } catch (e) {
    console.error('[clipboard] 保存历史失败:', e);
    return false;
  }
}

// 添加到剪贴板历史
function addToHistory(text) {
  if (!text || text.trim() === '') return;

  var history = getClipboardHistory();

  // 检查是否已存在（去重）
  var index = history.findIndex(function(item) { return item.text === text; });
  if (index !== -1) {
    history.splice(index, 1);
  }

  // 添加到开头
  history.unshift({
    text: text,
    timestamp: Date.now(),
    length: text.length
  });

  // 限制数量
  if (history.length > MAX_HISTORY) {
    history = history.slice(0, MAX_HISTORY);
  }

  saveClipboardHistory(history);
}

// 工具函数：Base64 编码
function base64Encode(text) {
  try {
    return btoa(unescape(encodeURIComponent(text)));
  } catch (e) {
    return '错误: ' + e.message;
  }
}

// 工具函数：Base64 解码
function base64Decode(text) {
  try {
    return decodeURIComponent(escape(atob(text)));
  } catch (e) {
    return '错误: ' + e.message;
  }
}

// 插件初始化
function pluginInit() {
  console.log('[clipboard] ✅ 剪贴板插件就绪');

  return { success: true, message: 'Clipboard plugin ready!' };
}

// 搜索函数
function onSearch(query) {
  console.log('[clipboard] 🔍 收到搜索请求:', query);
  var results = [];
  var q = query.toLowerCase().trim();

  // 获取当前剪贴板内容
  var currentText = '';
  try {
    if (typeof utools !== 'undefined' && utools.clipboard) {
      currentText = utools.clipboard.readText();
    }
  } catch (e) {
    console.warn('[clipboard] 无法读取剪贴板:', e);
  }

  // 显示剪贴板内容
  if (currentText && currentText.trim() !== '') {
    var preview = currentText.length > 50 ? currentText.substring(0, 50) + '...' : currentText;
    results.push({
      title: '📋 当前剪贴板: ' + preview,
      description: '长度: ' + currentText.length + ' 字符',
      icon: '📋',
      action: 'copyCurrent',
      data: { text: currentText }
    });
  }

  // 显示历史记录
  var history = getClipboardHistory();
  for (var i = 0; i < Math.min(history.length, 5); i++) {
    var item = history[i];
    var preview = item.text.length > 40 ? item.text.substring(0, 40) + '...' : item.text;
    var match = !q || q === '' || item.text.toLowerCase().indexOf(q) !== -1;

    if (match) {
      results.push({
        title: '⏰ ' + preview,
        description: new Date(item.timestamp).toLocaleString(),
        icon: '📋',
        action: 'restoreHistory',
        data: { index: i, item: item }
      });
    }
  }

  // 显示工具菜单
  if (!q || q.indexOf('cb') === 0 || q.indexOf('剪贴') >= 0 || q.indexOf('clip') >= 0) {
    if (currentText && currentText.trim() !== '') {
      results.push({
        title: '🔠 转为大写',
        description: '将剪贴板内容转为大写',
        icon: '🔠',
        action: 'toUpperCase',
        data: { text: currentText }
      });

      results.push({
        title: '🔡 转为小写',
        description: '将剪贴板内容转为小写',
        icon: '🔡',
        action: 'toLowerCase',
        data: { text: currentText }
      });

      results.push({
        title: '🔐 Base64 编码',
        description: '将剪贴板内容编码为 Base64',
        icon: '🔐',
        action: 'toBase64',
        data: { text: currentText }
      });

      results.push({
        title: '🔓 Base64 解码',
        description: '将剪贴板内容从 Base64 解码',
        icon: '🔓',
        action: 'fromBase64',
        data: { text: currentText }
      });
    }

    results.push({
      title: '📋 查看完整历史',
      description: '查看所有剪贴板历史记录',
      icon: '📋',
      action: 'showFullHistory'
    });

    results.push({
      title: '🧹 清空剪贴板',
      description: '清空当前剪贴板内容',
      icon: '🧹',
      action: 'clearClipboard'
    });
  }

  return results.slice(0, 20);
}

// 执行动作函数
function onAction(action, data) {
  console.log('[clipboard] ⚡ 执行动作:', action, data);

  var currentText = '';
  try {
    if (typeof utools !== 'undefined' && utools.clipboard) {
      currentText = utools.clipboard.readText();
    }
  } catch (e) {
    console.warn('[clipboard] 无法读取剪贴板:', e);
  }

  switch (action) {
    case 'copyCurrent':
      try {
        if (data.text) {
          addToHistory(data.text);
        }
        return {
          type: 'text',
          message: '✅ 剪贴板内容:\n\n' + data.text
        };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

    case 'restoreHistory':
      try {
        var historyItem = data.item;
        if (typeof utools !== 'undefined' && utools.clipboard) {
          utools.clipboard.writeText(historyItem.text);
        }
        return {
          type: 'text',
          message: '✅ 已恢复到剪贴板:\n\n' + historyItem.text
        };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

    case 'toUpperCase':
      try {
        var upper = data.text.toUpperCase();
        if (typeof utools !== 'undefined' && utools.clipboard) {
          utools.clipboard.writeText(upper);
          addToHistory(upper);
        }
        return { type: 'text', message: '✅ 转为大写:\n\n' + upper };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

    case 'toLowerCase':
      try {
        var lower = data.text.toLowerCase();
        if (typeof utools !== 'undefined' && utools.clipboard) {
          utools.clipboard.writeText(lower);
          addToHistory(lower);
        }
        return { type: 'text', message: '✅ 转为小写:\n\n' + lower };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

    case 'toBase64':
      try {
        var encoded = base64Encode(data.text);
        if (typeof utools !== 'undefined' && utools.clipboard) {
          utools.clipboard.writeText(encoded);
          addToHistory(encoded);
        }
        return { type: 'text', message: '✅ Base64 编码:\n\n' + encoded };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

    case 'fromBase64':
      try {
        var decoded = base64Decode(data.text);
        if (typeof utools !== 'undefined' && utools.clipboard) {
          utools.clipboard.writeText(decoded);
          addToHistory(decoded);
        }
        return { type: 'text', message: '✅ Base64 解码:\n\n' + decoded };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

    case 'showFullHistory':
      var fullHistory = getClipboardHistory();
      var list = fullHistory.map(function(item, index) {
        var preview = item.text.length > 30 ? item.text.substring(0, 30) + '...' : item.text;
        return (index + 1) + '. [' + new Date(item.timestamp).toLocaleTimeString() + '] ' + preview;
      }).join('\n');
      return {
        type: 'text',
        message: '📋 剪贴板历史 (共 ' + fullHistory.length + ' 条)\n\n' + list
      };

    case 'clearClipboard':
      try {
        if (typeof utools !== 'undefined' && utools.clipboard) {
          utools.clipboard.writeText('');
        }
        return { type: 'text', message: '🧹 剪贴板已清空' };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

    default:
      return { type: 'error', message: '❓ 未知动作: ' + action };
  }
}

// 导出
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { pluginInit, onSearch, onAction };
}
