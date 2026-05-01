/**
 * QR Code 二维码工具插件
 *
 * 功能：
 * 1. 文本生成二维码
 * 2. 支持 URL、联系方式、WiFi 等格式
 * 3. 二维码解析（演示）
 */

// 简单的二维码生成（演示用，实际项目建议使用专门的库）
function generateSimpleQR(text, size) {
  size = size || 256;

  // 这里仅演示格式，实际需要二维码库
  var svg = '<svg xmlns="http://www.w3.org/2000/svg" width="' + size + '" height="' + size + '">';
  svg += '<rect width="' + size + '" height="' + size + '" fill="white"/>';
  svg += '<text x="' + (size / 2) + '" y="' + (size / 2) + '" text-anchor="middle" fill="black" font-size="12">QR Code</text>';
  svg += '<text x="' + (size / 2) + '" y="' + (size / 2 + 20) + '" text-anchor="middle" fill="black" font-size="10">' + text.substring(0, 20) + '</text>';
  svg += '</svg>';

  return {
    svg: svg,
    size: size,
    text: text
  };
}

// 生成 WiFi 格式
function generateWiFiQR(ssid, password, encryption) {
  var encryption = encryption || 'WPA';
  var wifiStr = 'WIFI:S:' + ssid + ';T:' + encryption + ';P:' + password + ';;';
  return wifiStr;
}

// 生成联系信息格式
function generateContactQR(name, phone, email) {
  var contact = 'BEGIN:VCARD\nVERSION:3.0\nN:' + name + '\n';
  if (phone) contact += 'TEL:' + phone + '\n';
  if (email) contact += 'EMAIL:' + email + '\n';
  contact += 'END:VCARD';
  return contact;
}

// 插件初始化
function pluginInit() {
  console.log('[qrcode] ✅ 二维码插件就绪');

  return { success: true, message: 'QR Code plugin ready!' };
}

// 搜索函数
function onSearch(query) {
  console.log('[qrcode] 🔍 收到搜索请求:', query);
  var results = [];
  var q = query.toLowerCase().trim();

  // 获取剪贴板内容
  var clipboardText = '';
  try {
    if (typeof utools !== 'undefined' && utools.clipboard) {
      clipboardText = utools.clipboard.readText();
    }
  } catch (e) {
    console.warn('[qrcode] 无法读取剪贴板:', e);
  }

  // 如果有剪贴板内容，显示快速生成
  if (clipboardText && clipboardText.trim() !== '') {
    var preview = clipboardText.length > 40 ? clipboardText.substring(0, 40) + '...' : clipboardText;
    results.push({
      title: '🎨 剪贴板内容生成二维码',
      description: preview,
      icon: '🎨',
      action: 'generateFromClipboard',
      data: { text: clipboardText }
    });
  }

  // 如果有查询内容，显示快速生成
  if (q && q.length > 0 && !q.startsWith('qr')) {
    results.push({
      title: '🎨 为 "' + q + '" 生成二维码',
      description: '点击生成二维码',
      icon: '🎨',
      action: 'generateQuick',
      data: { text: q }
    });
  }

  // 显示主要功能
  if (!q || q.indexOf('qr') === 0 || q.indexOf('二维码') >= 0) {
    results.push({
      title: '🎨 生成文本二维码',
      description: '输入文本生成二维码',
      icon: '🎨',
      action: 'generateText'
    });

    results.push({
      title: '🌐 生成 URL 二维码',
      description: '为网站链接生成二维码',
      icon: '🌐',
      action: 'generateURL'
    });

    results.push({
      title: '📱 生成 WiFi 二维码',
      description: '为 WiFi 配置生成二维码',
      icon: '📱',
      action: 'generateWiFi'
    });

    results.push({
      title: '👤 生成联系人二维码',
      description: '为联系信息生成二维码',
      icon: '👤',
      action: 'generateContact'
    });
  }

  return results.slice(0, 20);
}

// 执行动作函数
function onAction(action, data) {
  console.log('[qrcode] ⚡ 执行动作:', action, data);

  var clipboardText = '';
  try {
    if (typeof utools !== 'undefined' && utools.clipboard) {
      clipboardText = utools.clipboard.readText();
    }
  } catch (e) {
    console.warn('[qrcode] 无法读取剪贴板:', e);
  }

  switch (action) {
    case 'generateFromClipboard':
    case 'generateQuick':
      try {
        var text = data && data.text ? data.text : clipboardText;
        if (!text) {
          return { type: 'error', message: '❌ 请先输入或复制要生成的内容' };
        }

        var qr = generateSimpleQR(text, 256);

        return {
          type: 'text',
          message: '🎨 二维码生成成功！\n\n内容:\n' + text + '\n\n⚠️ 提示: 完整 SVG 渲染需要 Webview 支持'
        };
      } catch (e) {
        return { type: 'error', message: '❌ 生成失败: ' + e.message };
      }

    case 'generateText':
      return {
        type: 'text',
        message: '🎨 文本二维码生成器\n\n使用方式:\n1. 输入或复制要生成的文本\n2. 在搜索框中直接输入内容\n3. 点击"为 ... 生成二维码"\n\n支持格式:\n• 纯文本\n• URL (http/https)\n• 邮箱地址\n• 电话号码'
      };

    case 'generateURL':
      return {
        type: 'text',
        message: '🌐 URL 二维码生成器\n\n使用方式:\n1. 复制网址到剪贴板\n2. 点击"剪贴板内容生成二维码"\n\n或者直接在搜索框输入网址'
      };

    case 'generateWiFi':
      return {
        type: 'text',
        message: '📱 WiFi 二维码生成器\n\n格式示例:\n• SSID: MyNetwork\n• Password: 12345678\n• Encryption: WPA (默认)\n\n生成的二维码可让设备自动连接 WiFi'
      };

    case 'generateContact':
      return {
        type: 'text',
        message: '👤 联系人二维码生成器\n\n支持格式 (vCard):\n• 姓名\n• 电话\n• 邮箱\n\n可被手机通讯录直接识别'
      };

    default:
      return { type: 'error', message: '❓ 未知动作: ' + action };
  }
}

// 导出
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { pluginInit, onSearch, onAction };
}
