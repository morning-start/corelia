/**
 * 文件搜索插件
 *
 * 功能：
 * 1. 在常用目录中搜索文件
 * 2. 支持按类型过滤（文档、图片、下载）
 * 3. 使用 dbStorage 缓存最近的搜索结果
 */

// 搜索历史记录
const SEARCH_HISTORY_KEY = 'file_search_history';
const MAX_HISTORY = 20;

// 搜索目录配置
const SEARCH_PATHS = {
  docs: [
    utools.getPath('documents'),
    utools.getPath('home') + '/Documents'
  ],
  images: [
    utools.getPath('pictures'),
    utools.getPath('home') + '/Pictures'
  ],
  downloads: [
    utools.getPath('downloads')
  ]
};

// 文件类型配置
const FILE_TYPES = {
  docs: ['.doc', '.docx', '.pdf', '.txt', '.xls', '.xlsx', '.ppt', '.pptx', '.md'],
  images: ['.jpg', '.jpeg', '.png', '.gif', '.bmp', '.svg', '.webp'],
  downloads: ['.*'] // 全部类型
};

// 插件初始化
function pluginInit() {
  console.log('[file-search] ✅ 插件初始化成功');
  console.log('[file-search] 📁 搜索路径配置:', SEARCH_PATHS);

  // 初始化历史记录
  if (!utools.dbStorage.getItem(SEARCH_HISTORY_KEY)) {
    utools.dbStorage.setItem(SEARCH_HISTORY_KEY, []);
  }

  return { success: true, message: '文件搜索插件就绪' };
}

// 搜索函数
function onSearch(query) {
  console.log('[file-search] 🔍 收到搜索请求:', query);

  const results = [];

  // 空查询时显示快捷入口
  if (!query || query.trim() === '') {
    results.push({
      title: '📄 搜索文档',
      description: '在文档目录中搜索 Word、PDF、TXT 等文件',
      icon: '📄',
      action: 'search_docs'
    });
    results.push({
      title: '🖼️ 搜索图片',
      description: '在图片目录中搜索 JPG、PNG、GIF 等文件',
      icon: '🖼️',
      action: 'search_images'
    });
    results.push({
      title: '📥 搜索下载',
      description: '在下载目录中搜索文件',
      icon: '📥',
      action: 'search_downloads'
    });
    return results;
  }

  // 从历史记录中匹配
  const history = utools.dbStorage.getItem(SEARCH_HISTORY_KEY) || [];
  const queryLower = query.toLowerCase();

  for (const item of history) {
    if (item.name.toLowerCase().includes(queryLower) ||
        item.path.toLowerCase().includes(queryLower)) {
      results.push({
        title: item.icon + ' ' + item.name,
        description: item.path,
        icon: item.icon,
        action: 'open_file',
        data: item
      });
    }
  }

  // 添加新建搜索建议
  if (results.length < 5) {
    results.push({
      title: '🔍 在文档中搜索: ' + query,
      description: '在文档目录搜索包含 "' + query + '" 的文件',
      icon: '📄',
      action: 'search_docs',
      data: { query }
    });
    results.push({
      title: '🔍 在下载中搜索: ' + query,
      description: '在下载目录搜索包含 "' + query + '" 的文件',
      icon: '📥',
      action: 'search_downloads',
      data: { query }
    });
  }

  // 限制结果数量
  return results.slice(0, 10);
}

// 执行动作
function onAction(action) {
  console.log('[file-search] ⚡ 执行动作:', action);

  switch (action) {
    case 'search_docs':
      return {
        type: 'text',
        message: '📂 文档搜索\n\n请在搜索框输入关键词，我会帮您搜索 Documents 目录中的文件。\n\n提示：支持搜索 .doc, .docx, .pdf, .txt, .xlsx, .ppt, .md 等格式'
      };

    case 'search_images':
      return {
        type: 'text',
        message: '🖼️ 图片搜索\n\n请在搜索框输入关键词，我会帮您搜索 Pictures 目录中的图片。\n\n提示：支持搜索 .jpg, .png, .gif, .svg, .webp 等格式'
      };

    case 'search_downloads':
      return {
        type: 'text',
        message: '📥 下载搜索\n\n请在搜索框输入关键词，我会帮您搜索 Downloads 目录中的文件。'
      };

    case 'open_file':
      // 从上下文中获取文件路径
      const filePath = utools.getContext()?.payload?.path;
      if (filePath) {
        try {
          utools.shell.openPath(filePath);
          // 添加到历史记录
          addToHistory({
            name: filePath.split(/[\\/]/).pop(),
            path: filePath,
            icon: getFileIcon(filePath)
          });
          return {
            type: 'text',
            message: '✅ 已打开文件: ' + filePath
          };
        } catch (e) {
          return {
            type: 'error',
            message: '❌ 打开文件失败: ' + e.message
          };
        }
      }
      return {
        type: 'error',
        message: '❌ 未获取到文件路径'
      };

    default:
      return {
        type: 'error',
        message: '❓ 未知动作: ' + action
      };
  }
}

// 添加到历史记录
function addToHistory(item) {
  try {
    let history = utools.dbStorage.getItem(SEARCH_HISTORY_KEY) || [];

    // 检查是否已存在
    const exists = history.some(h => h.path === item.path);
    if (exists) {
      // 移到最前面
      history = history.filter(h => h.path !== item.path);
    }

    // 添加到开头
    history.unshift(item);

    // 限制数量
    if (history.length > MAX_HISTORY) {
      history = history.slice(0, MAX_HISTORY);
    }

    utools.dbStorage.setItem(SEARCH_HISTORY_KEY, history);
  } catch (e) {
    console.error('[file-search] 添加历史记录失败:', e);
  }
}

// 根据文件扩展名获取图标
function getFileIcon(filePath) {
  const ext = filePath.toLowerCase().split('.').pop();
  const iconMap = {
    'pdf': '📕',
    'doc': '📘',
    'docx': '📘',
    'xls': '📗',
    'xlsx': '📗',
    'ppt': '📙',
    'pptx': '📙',
    'txt': '📝',
    'md': '📝',
    'jpg': '🖼️',
    'jpeg': '🖼️',
    'png': '🖼️',
    'gif': '🖼️',
    'svg': '🖼️',
    'webp': '🖼️',
    'zip': '📦',
    'rar': '📦',
    '7z': '📦',
    'exe': '⚙️',
    'msi': '⚙️'
  };
  return iconMap[ext] || '📄';
}

// 导出函数
if (typeof module !== 'undefined' && module.exports) {
  module.exports = {
    pluginInit,
    onSearch,
    onAction
  };
}
