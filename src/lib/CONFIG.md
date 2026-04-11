# 全局常量配置

## 概述

`src/lib/config.ts` 提供了应用的全局常量配置，确保整个应用的配置一致性。

## 使用方法

```typescript
import { WINDOW_CONFIG } from '$lib/config';

// 使用窗口配置
const width = WINDOW_CONFIG.WIDTH;  // 680
const height = WINDOW_CONFIG.HEIGHT; // 520
```

## 配置项

### WINDOW_CONFIG

窗口相关配置：

```typescript
{
  WIDTH: 600,    // 窗口宽度（逻辑像素，受 DPI 缩放影响）
  HEIGHT: 500,   // 窗口高度（逻辑像素，受 DPI 缩放影响）
  MIN_WIDTH: 200,  // 最小宽度
  MIN_HEIGHT: 300, // 最小高度
  MAX_WIDTH: 1200, // 最大宽度
  MAX_HEIGHT: 900, // 最大高度
}
```

**使用场景**：
- Svelte 中设置窗口尺寸
- Rust 后端窗口初始化（如果需要）
- 响应式布局计算

### SEARCH_CONFIG

搜索相关配置：

```typescript
{
  MAX_HISTORY_ITEMS: 5,     // 搜索历史最大条目数
  DEBOUNCE_DELAY: 150,      // 搜索防抖延迟（毫秒）
}
```

### PERFORMANCE_CONFIG

性能相关配置：

```typescript
{
  DELAY_UNPIN_MS: 100,      // 延迟取消置顶时间（毫秒）
}
```

## 最佳实践

### ✅ 推荐

1. **统一使用全局常量**
   ```typescript
   // ✅ 好
   import { WINDOW_CONFIG } from '$lib/config';
   appWindow.setSize(new LogicalSize(WINDOW_CONFIG.WIDTH, WINDOW_CONFIG.HEIGHT));
   
   // ❌ 不好
   appWindow.setSize(new LogicalSize(680, 520));
   ```

2. **集中管理相关配置**
   ```typescript
   // config.ts
   export const WINDOW_CONFIG = { ... };
   export const SEARCH_CONFIG = { ... };
   ```

3. **使用 `as const` 确保类型安全**
   ```typescript
   export const CONFIG = {
     VALUE: 123,
   } as const;
   
   // 类型为字面量类型，不可修改
   ```

### ❌ 避免

1. **硬编码数值**
   ```typescript
   // ❌ 避免
   const width = 680;
   
   // ✅ 推荐
   const width = WINDOW_CONFIG.WIDTH;
   ```

2. **分散配置**
   ```typescript
   // ❌ 避免 - 多处定义相同配置
   const WIDTH = 680;  // file1.ts
   const WIDTH = 680;  // file2.ts
   
   // ✅ 推荐 - 统一从 config 导入
   import { WINDOW_CONFIG } from '$lib/config';
   ```

## 扩展配置

添加新配置时，遵循以下模式：

```typescript
/**
 * 功能模块配置
 */
export const FEATURE_CONFIG = {
  /** 配置项说明 */
  KEY: value,
} as const;
```

## 文件位置

```
src/
├── lib/
│   ├── config.ts          # 全局常量配置
│   ├── components/        # 组件
│   ├── services/          # 服务
│   └── stores/            # 状态管理
```

## 版本历史

- **v1.0.0** (2026-04-04): 初始版本
  - WINDOW_CONFIG
  - SEARCH_CONFIG
  - PERFORMANCE_CONFIG
