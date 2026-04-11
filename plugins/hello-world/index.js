/**
 * Hello World 探针插件
 *
 * 功能：
 * 1. 验证 window.utools API 可用性
 * 2. 测试 dbStorage 读写
 * 3. 返回简单的搜索结果
 */

// 插件初始化函数（由 Corelia 在 load_plugin 时调用）
function pluginInit() {
  console.log('[hello-world] ✅ 插件初始化成功');

  // 测试 dbStorage API 是否可用
  try {
    if (typeof utools !== 'undefined' && utools.dbStorage) {
      const testKey = 'hello_world_test';
      const testValue = {
        time: new Date().toISOString(),
        message: 'from hello-world plugin',
        version: '1.0.0'
      };

      // 写入测试数据
      utools.dbStorage.setItem(testKey, testValue);

      // 读回验证
      const stored = utools.dbStorage.getItem(testKey);
      console.log('[hello-world] ✅ dbStorage 测试通过:', stored);
    } else {
      console.warn('[hello-world] ⚠️ utools.dbStorage 不可用');
    }
  } catch (e) {
    console.error('[hello-world] ❌ dbStorage 测试失败:', e);
  }

  return { success: true, message: 'Hello World 插件就绪 🎉' };
}

// 搜索函数（用户输入时调用）
// 返回搜索结果数组
function onSearch(query) {
  console.log('[hello-world] 🔍 收到搜索请求:', query);

  const results = [];

  // 如果查询包含 hello、hw 或问候相关关键词
  if (query.toLowerCase().includes('hello') ||
      query.toLowerCase().includes('hw') ||
      query.toLowerCase().includes('你好') ||
      query === '') {
    results.push({
      title: '👋 说 Hello',
      description: '输出 Hello 消息，测试基础功能',
      icon: '✨',
      action: 'sayHello'
    });
  }

  // 如果查询包含 world 或世界
  if (query.toLowerCase().includes('world') ||
      query.toLowerCase().includes('世界')) {
    results.push({
      title: '🌍 说 World',
      description: '输出 World 消息，测试多语言支持',
      icon: '🌏',
      action: 'sayWorld'
    });
  }

  // 如果查询包含 test、存储或 storage
  if (query.toLowerCase().includes('test') ||
      query.toLowerCase().includes('存储') ||
      query.toLowerCase().includes('storage')) {
    results.push({
      title: '🧪 测试存储',
      description: '测试 dbStorage 读写功能是否正常',
      icon: '💾',
      action: 'testStorage'
    });
  }

  // 返回结果数组（空数组表示无匹配）
  return results;
}

// 执行动作函数（用户选择某个结果后调用）
// 返回执行结果对象
function onAction(action) {
  console.log('[hello-world] ⚡ 执行动作:', action);

  switch (action) {
    case 'sayHello':
      return {
        type: 'text',
        message: '🎉 Hello from Corelia Plugin System!',
        timestamp: new Date().toISOString()
      };

    case 'sayWorld':
      return {
        type: 'text',
        message: '🌍 World, welcome to Corelia! 你好世界！',
        timestamp: new Date().toISOString()
      };

    case 'testStorage':
      try {
        const data = utools.dbStorage.getItem('hello_world_test');
        return {
          type: 'text',
          message: `💾 存储测试成功！\n\n读取到的数据:\n${JSON.stringify(data, null, 2)}`,
          data: data,
          timestamp: new Date().toISOString()
        };
      } catch (e) {
        return {
          type: 'error',
          message: `❌ 存储测试失败: ${e.message || e}`
        };
      }

    default:
      return {
        type: 'error',
        message: `❓ 未知动作: ${action}`
      };
  }
}

// 导出函数供 Corelia 调用（CommonJS 模块格式）
if (typeof module !== 'undefined' && module.exports) {
  module.exports = {
    pluginInit,
    onSearch,
    onAction
  };
}
