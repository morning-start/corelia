/**
 * Clipboard 剪贴板增强插件
 *
 * 功能：
 * 1. 剪贴板历史记录
 * 2. 格式转换（大小写、Base64、URL 编码等）
 * 3. 剪贴板内容预览
 * 4. JSON 格式化
 * 5. 字符/行数统计
 * 6. 去除空白
 * 7. UUID 生成
 * 8. 时间戳
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

// 工具函数：URL 编码
function urlEncode(text) {
  try {
    return encodeURIComponent(text);
  } catch (e) {
    return '错误: ' + e.message;
  }
}

// 工具函数：URL 解码
function urlDecode(text) {
  try {
    return decodeURIComponent(text);
  } catch (e) {
    return '错误: ' + e.message;
  }
}

// 工具函数：JSON 格式化
function formatJson(text) {
  try {
    var obj = JSON.parse(text);
    return JSON.stringify(obj, null, 2);
  } catch (e) {
    return '错误: 无效的 JSON - ' + e.message;
  }
}

// 工具函数：去除多余空白
function trimWhitespace(text) {
  return text.replace(/\s+/g, ' ').trim();
}

// 工具函数：去除所有空白
function removeAllWhitespace(text) {
  return text.replace(/\s/g, '');
}

// 工具函数：首字母大写
function capitalizeFirst(text) {
  if (!text) return text;
  return text.charAt(0).toUpperCase() + text.slice(1);
}

// 工具函数：标题格式（每个单词首字母大写）
function toTitleCase(text) {
  return text.replace(/\w\S*/g, function(word) {
    return word.charAt(0).toUpperCase() + word.substr(1).toLowerCase();
  });
}

// 工具函数：反转字符串
function reverseString(text) {
  return text.split('').reverse().join('');
}

// 工具函数：生成 UUID v4
function generateUUID() {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
    var r = Math.random() * 16 | 0;
    var v = c === 'x' ? r : (r & 0x3 | 0x8);
    return v.toString(16);
  });
}

// 工具函数：获取时间戳
function getTimestamp() {
  return Date.now().toString();
}

// 工具函数：统计字符/行数/字数
function getStats(text) {
  var chars = text.length;
  var lines = (text.match(/\n/g) || []).length + 1;
  var words = text.trim() ? text.trim().split(/\s+/).length : 0;
  return '字符: ' + chars + '\n行数: ' + lines + '\n字数: ' + words;
}

// 工具函数：Markdown 转纯文本
function markdownToText(text) {
  return text
    .replace(/[#*_`~]/g, '') // 移除 Markdown 符号
    .replace(/\[([^\]]*)\]\([^)]*\)/g, '$1') // 移除链接
    .replace(/!\[([^\]]*)\]\([^)]*\)/g, '$1') // 移除图片
    .replace(/>\s*/g, ''); // 移除引用符号
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
    // 添加历史记录条目，然后添加工具
    if (currentText && currentText.trim() !== '') {
      // 基础转换
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
        title: '🔤 首字母大写',
        description: '将剪贴板内容首字母大写',
        icon: '🔤',
        action: 'capitalizeFirst',
        data: { text: currentText }
      });

      results.push({
        title: '📝 标题格式',
        description: '每个单词首字母大写',
        icon: '📝',
        action: 'toTitleCase',
        data: { text: currentText }
      });

      // 编码转换
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

      results.push({
        title: '🔗 URL 编码',
        description: 'URL 编码当前剪贴板内容',
        icon: '🔗',
        action: 'urlEncode',
        data: { text: currentText }
      });

      results.push({
        title: '🔓 URL 解码',
        description: 'URL 解码当前剪贴板内容',
        icon: '🔓',
        action: 'urlDecode',
        data: { text: currentText }
      });

      // 格式化
      results.push({
        title: '📄 JSON 格式化',
        description: '格式化 JSON 字符串',
        icon: '📄',
        action: 'formatJson',
        data: { text: currentText }
      });

      // 空白处理
      results.push({
        title: '✨ 去除多余空白',
        description: '将连续空白替换为单个空格',
        icon: '✨',
        action: 'trimWhitespace',
        data: { text: currentText }
      });

      results.push({
        title: '🚫 去除所有空白',
        description: '移除所有空白字符',
        icon: '🚫',
        action: 'removeAllWhitespace',
        data: { text: currentText }
      });

      // 其他工具
      results.push({
        title: '🔄 反转字符串',
        description: '反转当前剪贴板内容',
        icon: '🔄',
        action: 'reverseString',
        data: { text: currentText }
      });

      results.push({
        title: '📊 统计信息',
        description: '显示字符数、行数、字数',
        icon: '📊',
        action: 'getStats',
        data: { text: currentText }
      });

      results.push({
        title: '📝 Markdown 转纯文本',
        description: '移除 Markdown 格式符号',
        icon: '📝',
        action: 'markdownToText',
        data: { text: currentText }
      });

      // 将当前内容添加到历史的选项
      results.push({
        title: '💾 保存到历史',
        description: '将当前剪贴板内容保存到历史',
        icon: '💾',
        action: 'saveToHistory',
        data: { text: currentText }
      });
    }

    // 独立工具（不需要剪贴板内容）
    results.push({
      title: '🆔 生成 UUID',
      description: '生成并复制一个 UUID v4',
      icon: '🆔',
      action: 'generateUUID'
    });

    results.push({
      title: '⏱️ 获取时间戳',
      description: '获取当前时间戳（毫秒）',
      icon: '⏱️',
      action: 'getTimestamp'
    });

    // 历史管理
    results.push({
      title: '📋 查看完整历史',
      description: '查看所有剪贴板历史记录',
      icon: '📋',
      action: 'showFullHistory'
    });

    results.push({
      title: '🗑️ 清空历史记录',
      description: '删除所有剪贴板历史',
      icon: '🗑️',
      action: 'clearHistory'
    });

    results.push({
      title: '🧹 清空剪贴板',
      description: '清空当前剪贴板内容',
      icon: '🧹',
      action: 'clearClipboard'
    });
  }

  return results.slice(0, 30);
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

    // 基础转换
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

    case 'capitalizeFirst':
      try {
        var capitalized = capitalizeFirst(data.text);
        if (typeof utools !== 'undefined' && utools.clipboard) {
          utools.clipboard.writeText(capitalized);
          addToHistory(capitalized);
        }
        return { type: 'text', message: '✅ 首字母大写:\n\n' + capitalized };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

    case 'toTitleCase':
      try {
        var title = toTitleCase(data.text);
        if (typeof utools !== 'undefined' && utools.clipboard) {
          utools.clipboard.writeText(title);
          addToHistory(title);
        }
        return { type: 'text', message: '✅ 标题格式:\n\n' + title };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

    // 编码转换
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

    case 'urlEncode':
      try {
        var encodedUrl = urlEncode(data.text);
        if (typeof utools !== 'undefined' && utools.clipboard) {
          utools.clipboard.writeText(encodedUrl);
          addToHistory(encodedUrl);
        }
        return { type: 'text', message: '✅ URL 编码:\n\n' + encodedUrl };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

    case 'urlDecode':
      try {
        var decodedUrl = urlDecode(data.text);
        if (typeof utools !== 'undefined' && utools.clipboard) {
          utools.clipboard.writeText(decodedUrl);
          addToHistory(decodedUrl);
        }
        return { type: 'text', message: '✅ URL 解码:\n\n' + decodedUrl };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

    // 格式化
    case 'formatJson':
      try {
        var formatted = formatJson(data.text);
        if (typeof utools !== 'undefined' && utools.clipboard) {
          utools.clipboard.writeText(formatted);
        }
        return { type: 'text', message: '✅ JSON 格式化:\n\n' + formatted };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

    // 空白处理
    case 'trimWhitespace':
      try {
        var trimmed = trimWhitespace(data.text);
        if (typeof utools !== 'undefined' && utools.clipboard) {
          utools.clipboard.writeText(trimmed);
          addToHistory(trimmed);
        }
        return { type: 'text', message: '✅ 去除多余空白:\n\n' + trimmed };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

    case 'removeAllWhitespace':
      try {
        var noWhitespace = removeAllWhitespace(data.text);
        if (typeof utools !== 'undefined' && utools.clipboard) {
          utools.clipboard.writeText(noWhitespace);
          addToHistory(noWhitespace);
        }
        return { type: 'text', message: '✅ 去除所有空白:\n\n' + noWhitespace };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

    // 其他工具
    case 'reverseString':
      try {
        var reversed = reverseString(data.text);
        if (typeof utools !== 'undefined' && utools.clipboard) {
          utools.clipboard.writeText(reversed);
          addToHistory(reversed);
        }
        return { type: 'text', message: '✅ 反转字符串:\n\n' + reversed };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

    case 'getStats':
      try {
        var stats = getStats(data.text);
        return { type: 'text', message: '📊 统计信息:\n\n' + stats };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

    case 'markdownToText':
      try {
        var plainText = markdownToText(data.text);
        if (typeof utools !== 'undefined' && utools.clipboard) {
          utools.clipboard.writeText(plainText);
          addToHistory(plainText);
        }
        return { type: 'text', message: '✅ Markdown 转纯文本:\n\n' + plainText };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

    case 'saveToHistory':
      try {
        addToHistory(data.text);
        return { type: 'text', message: '✅ 已保存到历史记录' };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

    // 独立工具
    case 'generateUUID':
      try {
        var uuid = generateUUID();
        if (typeof utools !== 'undefined' && utools.clipboard) {
          utools.clipboard.writeText(uuid);
          addToHistory(uuid);
        }
        return { type: 'text', message: '✅ 生成 UUID:\n\n' + uuid };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

    case 'getTimestamp':
      try {
        var timestamp = getTimestamp();
        if (typeof utools !== 'undefined' && utools.clipboard) {
          utools.clipboard.writeText(timestamp);
          addToHistory(timestamp);
        }
        return { type: 'text', message: '✅ 当前时间戳:\n\n' + timestamp + '\n\n日期: ' + new Date(parseInt(timestamp)).toLocaleString() };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

    // 历史管理
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

    case 'clearHistory':
      try {
        saveClipboardHistory([]);
        return { type: 'text', message: '🗑️ 历史记录已清空' };
      } catch (e) {
        return { type: 'error', message: '❌ 失败: ' + e.message };
      }

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
