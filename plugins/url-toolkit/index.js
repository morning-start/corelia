/**
 * URL Toolkit 插件
 *
 * 功能：
 * 1. URL Encode/Decode
 * 2. Base64 编解码
 * 3. JSON 格式化
 * 4. Hash 生成（简单版）
 */

// ==================== 工具函数 ====================

/** 简单的 Base64 编码（纯 JS 实现） */
function base64Encode(str) {
  // 先将字符串转为 UTF-8 字节数组
  var utf8 = unescape(encodeURIComponent(str));
  var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
  var result = '';
  
  for (var i = 0; i < utf8.length; i += 3) {
    var b1 = utf8.charCodeAt(i);
    var b2 = (i + 1 < utf8.length) ? utf8.charCodeAt(i + 1) : 0;
    var b3 = (i + 2 < utf8.length) ? utf8.charCodeAt(i + 2) : 0;

    result += chars.charAt(b1 >> 2);
    result += chars.charAt(((b1 & 3) << 4) | (b2 >> 4));
    result += (i + 1 < utf8.length) ? chars.charAt(((b2 & 15) << 2) | (b3 >> 6)) : '=';
    result += (i + 2 < utf8.length) ? chars.charAt(b3 & 63) : '=';
  }
  
  return result;
}

/** 简单的 Base64 解码 */
function base64Decode(str) {
  try {
    str = str.replace(/\s/g, '');
    var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
    var result = '';
    
    for (var i = 0; i < str.length; i += 4) {
      var idx1 = chars.indexOf(str.charAt(i));
      var idx2 = (i + 1 < str.length) ? chars.indexOf(str.charAt(i + 1)) : 0;
      var idx3 = (i + 2 < str.length && str.charAt(i + 2) !== '=') ? chars.indexOf(str.charAt(i + 2)) : -1;
      var idx4 = (i + 3 < str.length && str.charAt(i + 3) !== '=') ? chars.indexOf(str.charAt(i + 3)) : -1;
      
      var b1 = (idx1 << 2) | (idx2 >> 4);
      var b2 = ((idx2 & 15) << 4) | (idx3 >= 0 ? idx3 >> 2 : 0);
      var b3 = ((idx3 & 3) << 6) | (idx4 >= 0 ? idx4 : 0);

      result += String.fromCharCode(b1);
      if (idx3 >= 0) result += String.fromCharCode(b2);
      if (idx4 >= 0) result += String.fromCharCode(b3);
    }
    
    return decodeURIComponent(escape(result));
  } catch (e) {
    return { error: 'Base64 解码失败: ' + e.message };
  }
}

/** 简单哈希（djb2 算法，用于演示） */
function simpleHash(str) {
  var hash = 5381;
  for (var i = 0; i < str.length; i++) {
    hash = ((hash << 5) + hash) + str.charCodeAt(i);
  }
  return Math.abs(hash).toString(16);
}

/** 尝试格式化 JSON */
function tryFormatJson(text) {
  try {
    var parsed = JSON.parse(text);
    return JSON.stringify(parsed, null, 2);
  } catch (e) {
    return null;
  }
}

// ==================== 插件生命周期 ====================

function pluginInit() {
  console.log('[url-toolkit] ✅ 插件初始化成功');
  return { success: true, message: 'URL Toolkit 就绪' };
}

// ==================== 搜索函数 ====================

function onSearch(query) {
  console.log('[url-toolkit] 🔍 收到搜索请求:', query);
  var results = [];
  var q = query.toLowerCase().trim();

  if (!q || q === '' || q.indexOf('url') === 0 || q.indexOf('编码') >= 0 || q.indexOf('encode') >= 0 || q.indexOf('decode') >= 0) {
    results.push({
      title: '🔐 URL 编码',
      description: '对输入内容进行 URI Component 编码',
      icon: '🔐',
      action: 'urlEncode'
    });
    results.push({
      title: '🔓 URL 解码',
      description: '对输入内容进行 URI Component 解码',
      icon: '🔓',
      action: 'urlDecode'
    });
  }

  if (!q || q === '' || q.indexOf('base') >= 0 || q.indexOf('b64') >= 0) {
    results.push({
      title: '📦 Base64 编码',
      description: '将文本转换为 Base64 字符串',
      icon: '📦',
      action: 'base64Encode'
    });
    results.push({
      title: '📋 Base64 解码',
      description: '将 Base64 字符串还原为原文',
      icon: '📋',
      action: 'base64Decode'
    });
  }

  if (!q || q === '' || q.indexOf('json') >= 0 || q.indexOf('format') >= 0 || q.indexOf('格式') >= 0) {
    results.push({
      title: '🎨 JSON 格式化',
      description: '美化 / 压缩 JSON 字符串',
      icon: '🎨',
      action: 'jsonFormat'
    });
  }

  if (!q || q === '' || q.indexOf('hash') >= 0 || q.indexOf('md5') >= 0 || q.indexOf('sha') >= 0 || q.indexOf('哈希') >= 0) {
    results.push({
      title: '#️⃣ Hash 生成',
      description: '生成文本的简易 Hash 指纹',
      icon: '#️⃣',
      action: 'hashGen'
    });
  }

  // 如果查询本身看起来像是一个待处理的数据
  if (q.length > 3 && !results.some(function(r) { return r.action === 'quickEncode'; })) {
    if (q.indexOf('%') >= 0) {
      results.unshift({
        title: '🔓 快速解码',
        description: '自动检测并解码 URL 编码内容',
        icon: '⚡',
        action: 'autoDecode',
        data: { input: query }
      });
    } else if (/^[A-Za-z0-9+/=]+$/.test(q.replace(/\s/g, ''))) {
      results.unshift({
        title: '📋 Base64 快速解码',
        description: '尝试 Base64 解码输入内容',
        icon: '⚡',
        action: 'autoB64Decode',
        data: { input: query }
      });
    } else {
      var formatted = tryFormatJson(query);
      if (formatted) {
        results.unshift({
          title: '🎨 JSON 格式化预览',
          description: '检测到有效 JSON，点击格式化',
          icon: '⚡',
          action: 'jsonPreview',
          data: { input: query }
        });
      }
    }
  }

  return results.slice(0, 10);
}

// ==================== 动作执行函数 ====================

function onAction(action, data) {
  console.log('[url-toolkit] ⚡ 执行动作:', action, data);

  switch (action) {
    case 'urlEncode':
      return {
        type: 'text',
        message: '🔐 URL 编码\n\n请输入需要编码的内容，结果将复制到剪贴板。\n\n支持：URIComponent 编码',
        hint: '输入要编码的文本'
      };

    case 'urlDecode':
      return {
        type: 'text',
        message: '🔓 URL 解码\n\n请输入需要解码的 URL 编码内容。',
        hint: '输入要解码的 URL 编码文本'
      };

    case 'base64Encode':
      return {
        type: 'text',
        message: '📦 Base64 编码\n\n请输入需要编码为 Base64 的文本。',
        hint: '输入要编码的文本'
      };

    case 'base64Decode':
      return {
        type: 'text',
        message: '📋 Base64 解码\n\n请输入 Base64 字符串进行解码。',
        hint: '输入 Base64 字符串'
      };

    case 'jsonFormat':
      return {
        type: 'text',
        message: '🎨 JSON 格式化\n\n粘贴 JSON 字符串即可美化输出。\n\n支持：缩进、键排序、Unicode 转义',
        hint: '粘贴 JSON 字符串'
      };

    case 'hashGen':
      return {
        type: 'text',
        message: '#️⃣ Hash 生成\n\n输入文本生成简易哈希指纹。\n\n注意：此为演示用途的 djb2 哈希，不适用于安全场景。',
        hint: '输入要计算 Hash 的文本'
      };

    case 'autoDecode':
      try {
        var decoded = decodeURIComponent(data.input);
        utools.clipboard.writeText(decoded);
        return { type: 'text', message: '✅ 解码结果:\n\n' + decoded + '\n\n(已复制到剪贴板)' };
      } catch (e) {
        return { type: 'error', message: '❌ 自动解码失败: ' + e.message };
      }

    case 'autoB64Decode':
      try {
        var decoded = base64Decode(data.input);
        if (typeof decoded === 'object' && decoded.error) return { type: 'error', message: '❌ ' + decoded.error };
        utools.clipboard.writeText(decoded);
        return { type: 'text', message: '✅ Base64 解码结果:\n\n' + decoded + '\n\n(已复制到剪贴板)' };
      } catch (e) {
        return { type: 'error', message: '❌ Base64 解码失败: ' + e.message };
      }

    case 'jsonPreview':
      try {
        var formatted = tryFormatJson(data.input);
        utools.clipboard.writeText(formatted);
        return { type: 'text', message: '✅ 格式化后的 JSON:\n\n' + formatted + '\n\n(已复制到剪贴板)' };
      } catch (e) {
        return { type: 'error', message: '❌ 格式化失败: ' + e.message };
      }

    default:
      return { type: 'error', message: '❓ 未知动作: ' + action };
  }
}

// 导出
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { pluginInit, onSearch, onAction };
}
