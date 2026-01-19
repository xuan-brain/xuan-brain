# 日志系统使用指南

本文档说明如何在 xuan-brain 项目中使用 `tracing` 和 `tracing-subscriber` 进行日志记录。

## 概述

xuan-brain 使用 Rust 生态中标准的 `tracing` 框架进行日志记录，具有以下特点：

- **结构化日志**：支持结构化数据，便于查询和分析
- **异步感知**：完美配合 tokio 异步运行时
- **可配置性**：通过环境变量灵活控制日志级别
- **上下文追踪**：支持 span 和 context，便于追踪请求链路
- **高性能**：零开销设计，生产环境友好

## 日志级别

`tracing` 支持以下日志级别（从低到高）：

| 级别 | 说明 | 使用场景 |
|-------|------|----------|
| `trace` | 最详细的日志，包含所有函数调用 | 深度调试、性能分析 |
| `debug` | 调试信息，适合开发环境 | 开发调试、问题排查 |
| `info` | 一般信息，生产环境默认 | 应用状态、重要事件 |
| `warn` | 警告信息 | 潜在问题、降级处理 |
| `error` | 仅错误信息 | 错误、异常情况 |
| `off` | 禁用日志 | 不记录日志 |

## 配置方法

### 1. 使用环境变量

通过 `RUST_LOG` 环境变量配置日志级别：

```bash
# 开发环境（详细日志）
export RUST_LOG=debug

# 生产环境（只显示警告和错误）
export RUST_LOG=warn

# 针对特定模块
export RUST_LOG=xuan_brain=debug,tauri=info
```

### 2. 使用 .env 文件

创建 `.env` 文件（参考 `.env.example`）：

```env
# 基础配置
RUST_LOG=xuan_brain=debug,tauri=info

# 完整配置示例
RUST_LOG=info,xuan_brain::db=debug,xuan_brain::api=trace
```

### 3. 代码中的默认配置

在 `src-tauri/src/lib.rs` 中已经配置了默认日志级别：

```rust
tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new("xuan_brain=debug,tauri=debug")
        }),
    )
    .init();
```

## 日志宏使用

### 基础日志

```rust
use tracing::{info, debug, warn, error, trace};

// 信息日志
info!("应用启动中");
info!("用户登录: {}", username);

// 调试日志
debug!("处理请求: {:?}", request);
debug!("数据库查询耗时: {}ms", duration);

// 警告日志
warn!("缓存未命中，从数据库加载");
warn!("磁盘空间不足: {}%", usage);

// 错误日志
error!("数据库连接失败: {}", err);
error!("文件读取失败: {}", path.display());

// 追踪日志（最详细）
trace!("函数参数: x={}, y={}", x, y);
trace!("进入函数: {}", std::any::type_name::<Self>());
```

### 带字段的日志

```rust
use tracing::{info, instrument};

#[instrument]
async fn process_document(doc_id: i32) {
    info!("开始处理文档", doc_id = doc_id);
    // 自动记录函数参数和返回值
}

// 手动添加字段
info!(
    "文献导入完成",
    count = total_count,
    success = success_count,
    failed = failed_count,
    duration_ms = elapsed.as_millis()
);
```

### 错误处理

```rust
use tracing::error;

match result {
    Ok(data) => info!("操作成功: {:?}", data),
    Err(e) => error!(
        error = %e,  // 使用 Display trait
        error_chain = ?e,  // 使用 Debug trait（完整错误链）
        "操作失败"
    ),
}
```

## Span（跨度）

Span 用于追踪代码执行路径和上下文：

```rust
use tracing::{span, Level, info};

let span = span!(Level::INFO, "process_request", request_id = id);
let _enter = span.enter();

info!("开始处理");
// ... 代码 ...
info!("处理完成"); // 继承 span 的上下文
```

### 使用宏自动创建 span

```rust
use tracing::{info, instrument, error};

#[instrument(skip(secret_data))]  // 跳过敏感字段
async fn process_payment(
    user_id: i32,
    amount: f64,
    secret_data: &str
) -> Result<(), PaymentError> {
    info!("处理支付");
    // 自动记录函数参数和返回值
    
    match payment_gateway.charge(amount).await {
        Ok(_) => {
            info!("支付成功");
            Ok(())
        }
        Err(e) => {
            error!(error = %e, "支付失败");
            Err(e)
        }
    }
}
```

## 最佳实践

### 1. 选择合适的日志级别

- **trace**: 仅用于性能关键路径或深度调试
- **debug**: 开发期间的调试信息
- **info**: 重要的应用状态和业务事件
- **warn**: 潜在问题、降级处理、重试
- **error**: 实际错误、异常情况

### 2. 添加结构化字段

```rust
// 好的做法
info!(
    "用户操作",
    user_id = user.id,
    action = "import_document",
    document_count = documents.len(),
    duration_ms = elapsed.as_millis()
);

// 避免的做法
info!("用户 {} 导入了 {} 个文档，耗时 {}ms", 
    user.id, documents.len(), elapsed.as_millis());
```

### 3. 使用 `instrument` 宏

为异步函数自动添加 span：

```rust
#[instrument(skip_all)]  // 跳过所有参数
async fn background_task() {
    // 自动记录函数进入、返回和错误
}
```

### 4. 处理敏感数据

```rust
#[instrument(skip(password))]  // 不记录密码
async fn login(username: &str, password: &str) -> Result<bool> {
    // ...
}
```

### 5. 错误日志包含上下文

```rust
error!(
    error = %err,
    user_id = user.id,
    document_id = doc.id,
    retry_count = retries,
    "导入文献失败"
);
```

## Tauri 特定配置

### 开发环境

```bash
# 使用 RUST_LOG 环境变量
RUST_LOG=debug yarn tauri dev

# 或在系统环境变量中设置
export RUST_LOG=xuan_brain=debug
```

### 生产环境

```bash
# 最小日志，只显示错误
RUST_LOG=error yarn tauri build
```

### Windows PowerShell

```powershell
$env:RUST_LOG="xuan_brain=debug"
yarn tauri dev
```

### Windows CMD

```cmd
set RUST_LOG=xuan_brain=debug
yarn tauri dev
```

## 日志输出示例

### 控制台输出（开发模式）

```
2024-01-18T14:30:00.123456Z  INFO xuan_brain: 应用启动中...
2024-01-18T14:30:00.234567Z  INFO xuan_brain: 正在初始化应用数据目录...
2024-01-18T14:30:00.345678Z  INFO xuan_brain: 应用数据目录: "C:\\Users\\xxx\\AppData\\Roaming\\org.xuan-brain"
2024-01-18T14:30:00.456789Z  INFO xuan_brain: 创建 配置文件 目录: "C:\\Users\\xxx\\AppData\\Roaming\\org.xuan-brain\\config"
2024-01-18T14:30:00.567890Z  INFO xuan_brain: 配置文件 目录创建成功: "C:\\Users\\xxx\\AppData\\Roaming\\org.xuan-brain\\config"
```

### 带时间戳和文件位置的详细日志

```
2024-01-18T14:30:00.123456Z DEBUG xuan_brain::lib: greet 命令被调用，参数: "Alice"
                                           ^^^^^^^^^^^^^^^^^^^^^
                                           模块路径
```

## 故障排查

### 日志未显示

1. 检查环境变量是否正确设置：
   ```bash
   echo $RUST_LOG
   ```

2. 确认代码中使用了正确的日志宏：
   ```rust
   use tracing::info;
   info!("消息");  // 不是 println!
   ```

3. 查看日志级别是否匹配：
   ```bash
   RUST_LOG=debug  # 显示 debug 及以上级别
   ```

### 性能问题

如果日志影响性能，调整日志级别：

```bash
# 生产环境只记录错误
RUST_LOG=error
```

或禁用特定模块：

```bash
RUST_LOG=xuan_brain=debug,tokio=off
```

## 相关资源

- [tracing 官方文档](https://docs.rs/tracing/)
- [tracing-subscriber 文档](https://docs.rs/tracing-subscriber/)
- [EnvFilter 配置](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/struct.EnvFilter.html)