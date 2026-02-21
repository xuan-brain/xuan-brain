# 璇脑 (XUAN-BRAIN) 知识库

**生成时间:** 2026-02-22
**技术栈:** Tauri 2.x + Vue 3 + SurrealDB 3.0

## 概述

AI 驱动的科研文献管理桌面应用。借鉴 Zotero 理念，支持插件架构。双栈架构：Rust 后端（PDF 处理、数据库、AI）+ Vue 3 前端（UI、状态管理）。

## 目录结构

```
xuan-brain/
├── src/                    # Vue 3 前端
│   ├── components/         # 按领域划分的 UI 组件 (paper, navigation, settings, dialogs)
│   ├── pages/              # 路由级页面视图
│   ├── stores/             # Pinia 状态管理（支持持久化）
│   ├── lib/                # 工具库（API、i18n）
│   └── router/             # Vue Router 4
├── src-tauri/              # Rust 后端
│   └── src/
│       ├── command/        # Tauri IPC 命令（分层：paper/, category, label）
│       ├── repository/     # 数据访问层（SurrealDB）
│       ├── papers/         # 导入器（DOI, arXiv, PMID, PDF, GROBID）
│       ├── surreal/        # 数据库连接 + 模型
│       ├── axum/           # 内部 REST API 服务器
│       ├── llm/            # AI 集成
│       └── sys/            # 配置、目录、错误、日志
└── docs/                   # 架构文档
```

## 快速定位

| 任务            | 位置                             | 备注                              |
| --------------- | -------------------------------- | --------------------------------- |
| 添加 Tauri 命令 | `src-tauri/src/command/`         | 在 `lib.rs` invoke_handler 中注册 |
| 前端调用后端    | `src/lib/api/*.ts`               | 使用 `invokeCommand()` 辅助函数   |
| 文献导入逻辑    | `src-tauri/src/papers/importer/` | DOI, arXiv, PMID, PDF             |
| 数据库查询      | `src-tauri/src/repository/`      | SurrealDB 仓库层                  |
| Vue 组件        | `src/components/{domain}/`       | 按领域组织                        |
| 全局状态        | `src/stores/useAppStore.ts`      | Pinia + persistedstate            |
| 国际化          | `src/lib/i18n/locales/`          | JSON 文件 (en, zh)                |
| 主题/UI 配置    | `src/main.ts`                    | Vuetify + VxeTable 配置           |

## 常用命令

```bash
# 开发
yarn tauri dev              # 完整应用热重载
yarn dev                    # 仅前端 (端口 1420)

# 构建
yarn tauri build            # 生产构建（全平台）

# 代码质量
yarn lint && yarn tsc --noEmit
cd src-tauri && cargo clippy && cargo test
```

## 代码规范

**前端 (Vue 3)**

- 组合式 API + `<script setup>` SFC
- Pinia 状态管理（localStorage 持久化）
- Vuetify 3 组件（v-treeview, v-data-table）
- Tailwind CSS 工具类样式
- Tauri API 惰性加载：`await import('@tauri-apps/api/core')`

**后端 (Rust)**

- 分层架构：Command → Repository → SurrealDB
- `#[tauri::command]` 定义 IPC 处理器
- `#[instrument(skip(db))]` 添加追踪日志
- SurrealDB 3.0 + RocksDB 持久化
- Axum 内部 REST API 服务器

**数据库**

- 使用 SurrealDB（非 SQLite/SeaORM）
- 模型定义在 `src-tauri/src/surreal/models/`
- Repository 层封装数据访问

## 禁止事项

- **禁止自动提交代码** - 必须用户明确要求
- **禁止使用 `as any` / `@ts-ignore`** - 正确修复类型
- **禁止 `#[allow(...)]` 抑制警告** - 除非有充分理由
- **禁止生产环境留 console.log** - 使用 `console.info`/`console.error`
- **禁止模块顶层调用 Tauri API** - 必须惰性加载

## 特色模式

**Tauri API 辅助函数（前端）**

```typescript
async function invokeCommand<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd, args);
}
```

**命令注册（后端）**

```rust
// 1. 在 src-tauri/src/command/paper/query.rs 创建
#[tauri::command]
pub async fn get_all_papers(db: State<'_, Arc<SurrealClient>>) -> Result<Vec<Paper>> { ... }

// 2. 在 mod.rs 导出
// 3. 在 lib.rs invoke_handler! 中注册
```

**Pinia Store 模式**

```typescript
export const useAppStore = defineStore(
  'app',
  () => {
    const isDark = ref(true);
    return { isDark, toggleTheme: () => (isDark.value = !isDark.value) };
  },
  { persist: { storage: localStorage } }
);
```

## 核心模块

| 模块                             | 用途                     |
| -------------------------------- | ------------------------ |
| `command/paper/`                 | 文献增删改查、导入、附件 |
| `command/category_command.rs`    | 分类树操作               |
| `repository/paper_repository.rs` | SurrealDB 查询           |
| `papers/importer/`               | 元数据提取（6 种来源）   |
| `surreal/models/`                | 数据模型定义             |
| `axum/handlers/`                 | REST API 端点            |

## 注意事项

- Vite 开发服务器必须使用端口 **1420**（与 tauri.conf.json 匹配）
- 窗口关闭 → 隐藏（托盘图标），非退出
- 系统托盘：左键点击切换窗口可见性
- 默认暗色主题（store 中 `isDark: true`）
- VxeTable 数据表格（主题与 Vuetify 同步）
