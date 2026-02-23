# Rust 后端 (src-tauri)

**职责:** Tauri IPC 命令、数据库、PDF 处理、AI 集成

## 结构

```
src-tauri/src/
├── lib.rs              # 入口，注册所有命令
├── command/            # IPC 命令层
│   ├── paper/          # 文献命令（query, mutation, import, attachment）
│   ├── category_command.rs
│   ├── label_command.rs
│   ├── search_command.rs
│   └── config_command.rs
├── repository/         # 数据访问层
├── papers/             # 文献处理
│   └── importer/       # DOI, arXiv, PMID, PDF, GROBID, HTML
├── surreal/            # 数据库
│   ├── connection.rs
│   └── models/
├── axum/               # REST API 服务器
│   └── handlers/
├── llm/                # AI 集成
├── service/            # 业务服务
└── sys/                # 系统（config, dirs, error, log）
```

## 添加新命令

```rust
// 1. 在 command/paper/query.rs 创建
#[tauri::command]
pub async fn my_command(db: State<'_, Arc<SurrealClient>>) -> Result<String> {
    // 业务逻辑
}

// 2. 在 command/mod.rs 导出
// 3. 在 lib.rs invoke_handler! 注册
```

## 关键文件

| 文件                         | 用途                 |
| ---------------------------- | -------------------- |
| `lib.rs`                     | 入口点，所有命令注册 |
| `sys/error.rs`               | 统一错误类型         |
| `surreal/connection.rs`      | SurrealDB 连接       |
| `repository/*_repository.rs` | 数据访问封装         |

## 数据库

- SurrealDB 3.0 + RocksDB
- 模型在 `surreal/models/`
- Repository 模式封装查询

## 代码规范

- `#[instrument(skip(db))]` 追踪日志
- 禁止 `#[allow(...)]` 抑制警告
- 错误通过 `sys::error::Result` 返回
