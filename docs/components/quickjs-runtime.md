# QuickJS 运行时

> QuickJS 运行时提供沙箱化的 JavaScript 执行环境，管理 VM 实例池

## 核心结构

### QuickJSConfig

运行时全局配置，包含：
- `max_memory_bytes` - 单个 VM 最大内存限制（默认 50MB）
- `max_execution_time_ms` - 代码执行超时（默认 5秒）
- `max_vm_count` - 最大并发 VM 数（默认 10个）
- `idle_timeout_secs` - 闲置超时回收（默认 5分钟）

### VmInstance

单个 VM 实例，包含：
- QuickJS Runtime 和 Context
- 闲置追踪时间戳
- 关联的插件 ID

### QuickJSRuntime

VM 池管理器，负责：
- VM 创建和销毁
- 闲置回收
- 资源上限控制
- 线程安全访问

## 关键功能

### VM 创建流程

1. 检查 VM 数是否超限
2. 创建新的 QuickJS Runtime
3. 初始化 Context
4. 设置内存限制
5. 返回 VM 实例

### 代码执行

1. 检查执行超时
2. 在 VM Context 中执行 JS 代码
3. 捕获和返回执行结果或错误
4. 更新 VM 闲置时间戳

### 闲置回收

1. 定期检查所有 VM 的闲置时间
2. 回收超过超时时间的 VM
3. 更新 VM 计数

## 线程安全

使用 `Mutex` 保证多线程安全访问：
- `QuickJSRuntime` 使用 Mutex 包裹
- `OnceLock` 用于全局 AppHandle 持有
- 最小化锁持有范围

## 内存管理

- 每个 VM 独立内存空间
- 内存超限自动终止执行
- 支持内存使用监控

## 相关文件

- `src-tauri/src/plugins/quickjs_runtime.rs` - 主实现
