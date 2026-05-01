# 代码规范

> Corelia 代码风格与规范

---

## Rust 代码规范

### 格式化

```bash
# 格式化代码
cargo fmt
```

### Clippy 检查

```bash
# 运行 Clippy
cargo clippy
```

**要求**: 无 Clippy 警告

### 命名规范

| 类型 | 风格 | 示例 |
|------|------|------|
| 函数 | snake_case | `fn create_vm()` |
| 变量 | snake_case | `let plugin_name` |
| 常量 | SCREAMING_SNAKE_CASE | `const MAX_VM_COUNT` |
| 类型 | PascalCase | `struct PluginLoader` |
| 模块 | snake_case | `mod quickjs_runtime` |

### 文档注释

```rust
/// 创建新的 VM 实例
/// 
/// # Arguments
/// * `name` - VM 实例名称
/// 
/// # Returns
/// VM 实例 ID
pub fn create_vm(name: &str) -> String {
    // ...
}
```

---

## TypeScript 代码规范

### 格式化

使用 Prettier 格式化代码。

### 命名规范

| 类型 | 风格 | 示例 |
|------|------|------|
| 函数 | camelCase | `function loadData()` |
| 变量 | camelCase | `const pluginName` |
| 常量 | SCREAMING_SNAKE_CASE | `const MAX_COUNT` |
| 类型/接口 | PascalCase | `interface PluginConfig` |
| 文件 | kebab-case | `plugin-service.ts` |

### 类型定义

```typescript
// 优先使用 interface
interface PluginConfig {
  name: string;
  version: string;
}

// 复杂类型使用 type
type PluginAction = () => Promise<void>;
```

---

## Svelte 组件规范

### 组件结构

```svelte
<script lang="ts">
  // 1. 导入
  import { onMount } from 'svelte';
  
  // 2. Props
  interface Props {
    title: string;
  }
  let { title }: Props = $props();
  
  // 3. 状态
  let count = $state(0);
  
  // 4. 计算属性
  let doubled = $derived(count * 2);
  
  // 5. 方法
  function increment() {
    count++;
  }
</script>

<!-- 模板 -->
<button onclick={increment}>
  {title}: {count}
</button>

<!-- 样式 -->
<style>
  button {
    padding: 8px 16px;
  }
</style>
```

### Svelte 5 Runes

优先使用 Runes 语法：

| 旧语法 | 新语法 |
|--------|--------|
| `let count = 0` | `let count = $state(0)` |
| `$: doubled = count * 2` | `let doubled = $derived(count * 2)` |
| `writable(0)` | `$state(0)` |

---

## 提交规范

### Commit 消息格式

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Type 类型

| Type | 说明 |
|------|------|
| feat | 新功能 |
| fix | Bug 修复 |
| docs | 文档更新 |
| style | 代码格式 |
| refactor | 重构 |
| test | 测试 |
| chore | 构建/工具 |

### 示例

```
feat(plugin): add quickjs vm pool support

- Implement VM creation and destruction
- Add idle timeout cleanup
- Limit max VM count to 10

Closes #123
```

---

## 文件组织

### 目录结构

```
src/lib/
├── components/     # UI 组件
├── stores/         # 状态管理
├── services/       # 业务逻辑
└── utils/          # 工具函数

src-tauri/src/
├── commands/       # Tauri Commands
├── services/       # 业务服务
└── plugins/        # 插件系统
```

### 文件命名

- 组件: `PascalCase.svelte`
- 服务: `kebab-case.ts`
- 工具: `kebab-case.ts`
- 类型: `types.ts`

---

## 相关文档

- [环境配置](./environment.md)
- [MVP 指南](./mvp-guide.md)
