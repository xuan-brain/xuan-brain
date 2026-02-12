# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

**璇玑 (xuan-brain)** 是一个基于 **Tauri 2.x + Vue 3 + TypeScript** 构建的 AI 驱动科研文献管理桌面应用。本设计借鉴 Zotero 的核心理念，通过插件机制提供功能强大且易于使用的文献管理平台。

### 核心功能

- **文献导入与管理**: 支持 PDF、DOCX、HTML、EPUB 等多格式文献导入，自动提取元数据
- **AI 智能分析**:
  - 智能推荐相关论文(基于内容相似度和引用关系)
  - 自动分类和标签提取
  - 关键词自动提取(使用 NLP 技术)
  - 文献摘要生成(集成 LLM)
- **知识图谱**: 构建文献间引用关系、主题关联可视化
- **智能检索**: 全文检索、语义搜索、相关文献推荐
- **笔记与标注**: PDF 高亮、批注、笔记管理
- **引用管理**: 自动生成多种引用格式、参考文献管理
- **云同步**:
  - 官方云端同步服务(端到端加密)
  - WebDAV 协议支持(自建 NAS/云盘)
- **插件系统**: 支持 Rust 和 JavaScript 插件扩展
- **RESTful API**: 与第三方应用集成
- **本地优先**: 所有数据存储在本地，保护隐私

### 技术选型理由

- **Tauri 2.x**: 相比 Electron 更轻量(体积小 80%)、更安全(Rust 内存安全)、性能更好
- **Vue 3**: 现代化的组合式 API，良好的性能和开发体验
  - **组件模型**: 组合式 API (Composition API) + 单文件组件(SFC)
  - **状态管理**: Pinia（轻量且类型友好）
  - **路由**: Vue Router 4（官方路由解决方案）
- **Vuetify 3**: Material Design 3 的 Vue UI 组件库
  - **丰富的组件**: 覆盖桌面应用常见 UI 需求
  - **主题系统**: 内置暗色模式与主题定制能力
  - **生态系统**: 图标、数据表格、树形组件等
- **Tailwind CSS**: 实用优先 CSS 框架，用于样式增强
- **Rust 后端**: 高性能 PDF 处理、文件 I/O、AI 模型推理
- **SeaORM**: 异步 Rust ORM，提供类型安全的数据库操作
- **SQLite**: 嵌入式数据库、零配置、单文件备份

### 设计理念

- **模块化组织**: 每个功能模块独立开发和测试
- **性能保障**: Rust 的出色性能和并发能力
- **内存安全**: 降低内存相关漏洞风险
- **代码可维护性**: 清晰的模块划分

## 核心架构

### 前端 (Vue)

- **框架**: Vue 3 (Composition API + TypeScript)
- **构建工具**: Vite 6.x (快速的开发服务器和构建工具)
- **渲染模式**: SPA (单页应用)，无需 SSR
- **路由**: Vue Router 4，位于 `src/router/`
- **开发服务器**: 运行在 `http://localhost:1420`
- **状态管理**:
  - **客户端状态**: Pinia (轻量级全局状态)
  - **持久化**: pinia-plugin-persistedstate
- **UI 组件库**: Vuetify 3
  - 树形组件: `v-treeview`
  - 数据表格: `v-data-table`
  - 图标: `@mdi/font`
- **样式**: Tailwind CSS + Vuetify 主题系统
- **职责**:
  - 文献列表展示和 UI 交互
  - 搜索和筛选界面
  - 笔记和标签管理
  - 分类树管理
  - 通过 Tauri IPC 调用后端功能

### 后端 (Rust/Tauri)

- **入口点**: `src-tauri/src/main.rs` → 调用 `xuan_brain_lib::run()`
- **核心逻辑**: `src-tauri/src/lib.rs` - 包含 Tauri commands 和应用初始化
- **Commands**: 使用 `#[tauri::command]` 宏定义可从前端调用的函数
- **前端调用**: 使用 `invoke()` from `@tauri-apps/api/core`
- **职责**:
  - 文献导入、解析、索引、搜索等繁重任务
  - PDF 解析(pdfium/Poppler/MuPDF 绑定)
  - OCR 功能(Tesseract 集成)
  - AI 引擎(推荐、分类、关键词提取)
  - 数据库操作(SQLite + sqlx)
  - 云同步和 WebDAV 客户端
  - API 服务器

## 核心模块设计

### 1. 文献处理模块 (Papers)

```
src-tauri/src/papers/
├── mod.rs           # 文献处理模块入口
└── importer/        # 文献导入器
    ├── mod.rs       # 导入器入口
    ├── doi.rs       # DOI 导入器
    └── arxiv.rs     # arXiv 导入器
```

**核心功能**:

- **文献导入**:
  - 通过 DOI 从 Crossref 获取元数据
  - 通过 arXiv ID 从 arXiv API 获取元数据
- **元数据提取**: 自动提取标题、作者、摘要、出版信息等
- **文献管理**: 文献与作者、关键词、标签的关联
- **文献更新**: 支持更新文献详情、笔记、阅读状态等

**DOI 导入器** (`doi.rs`):

- 使用 Crossref REST API 获取文献元数据
- 解析响应并提取: 标题、作者、摘要、出版年份、期刊名称、URL
- 错误处理: 无效 DOI 格式、未找到文献、网络请求失败
- 验证: 检查文献是否已存在(通过 DOI)

**arXiv 导入器** (`arxiv.rs`):

- 使用 arXiv API 获取论文元数据
- 解析响应并提取: 标题、作者、摘要、发布日期、PDF URL、期刊引用
- 错误处理: 无效 arXiv ID 格式、未找到论文、网络请求失败
- 验证: 检查文献是否已存在(通过 DOI 或 URL)

### 2. 数据库层 (Database)

```
src-tauri/src/database/
├── mod.rs           # 数据库入口
├── entities/        # SeaORM 实体定义
│   ├── mod.rs
│   ├── papers.rs           # 文献主表
│   ├── authors.rs          # 作者表
│   ├── keywords.rs         # 关键词表
│   ├── labels.rs           # 标签表
│   ├── category.rs         # 分类表
│   ├── attachments.rs      # 附件表
│   ├── paper_authors.rs    # 文献-作者关联表
│   ├── paper_keywords.rs   # 文献-关键词关联表
│   ├── paper_labels.rs     # 文献-标签关联表
│   └── prelude.rs          # 实体导出
src-tauri/migration/
└── src/
    └── lib.rs              # 数据库迁移脚本
```

├── entities/ # SeaORM 实体定义
│ ├── mod.rs
│ ├── paper.rs
│ ├── category.rs
│ └── label.rs
src-tauri/migration/
├── Cargo.toml # 迁移项目配置
└── src/
└── lib.rs # 数据库迁移脚本

```

**数据存储**:
- 文献元数据(标题、作者、出版年份)
- 摘要、标签、关键词
- 用户笔记和标注
- 分类和标签管理
- 插件相关数据

**技术栈**:
- SQLite(嵌入式数据库)
- SeaORM 2.0 (异步 ORM 框架)
- SeaORM Migration (数据库迁移)
- 全文检索支持

**优势**:
- 轻量级、零配置
- 单文件备份和迁移
- 类型安全的数据库操作
- 支持异步操作

### 3. 命令层 (Command Layer)

```

src-tauri/src/command/
├── mod.rs # 命令入口
├── paper_command.rs # 文献相关命令
│ ├── get_all_papers # 获取所有文献
│ ├── get_paper # 获取单个文献
│ ├── import_paper_by_doi # 通过 DOI 导入
│ ├── import_paper_by_arxiv_id # 通过 arXiv ID 导入
│ ├── add_paper_label # 添加文献标签
│ ├── remove_paper_label # 移除文献标签
│ └── update_paper_details # 更新文献详情
├── category_command.rs # 分类相关命令
│ ├── load_categories # 加载分类树
│ ├── create_category # 创建分类
│ ├── delete_category # 删除分类
│ ├── update_category # 更新分类
│ └── move_category # 移动分类
└── label_command.rs # 标签相关命令
├── get_all_labels # 获取所有标签
├── create_label # 创建标签
├── delete_label # 删除标签
└── update_label # 更新标签

```

**核心命令**:
- 文献管理: 导入、查询、更新、标签关联
- 分类管理: 树形结构加载、增删改、拖拽移动
- 标签管理: 标签的增删改查

### 4. 服务层 (Service Layer)

```

src-tauri/src/service/
└── category_service.rs # 分类服务

```

**职责**:
- 封装复杂业务逻辑
- 处理数据库事务
- 提供可复用的服务方法

### 5. 系统模块 (System Module)
### 5. 系统模块

```

src-tauri/src/sys/
├── mod.rs # 系统模块入口
├── config.rs # 配置管理
├── consts.rs # 常量定义
├── dirs.rs # 目录管理
├── error.rs # 错误处理
└── log.rs # 日志配置

````

**功能**:
- 应用数据目录初始化(使用 `dirs` crate)
- 统一错误类型定义(`AppError` enum，支持多种错误类型)
- 日志系统配置(使用 `tracing` 和 `tracing-appender`)
  - 控制台输出：彩色、可读性强
  - 文件输出：详细格式，按周轮转
  - 支持通过 `RUST_LOG` 环境变量控制日志级别

**错误类型**:
- `DocumentParseError` - 文档解析错误
- `FileSystemError` - 文件系统错误
- `SeaOrmError` - 数据库错误
- `AIError` - AI 引擎错误
- `SyncError` - 同步错误
- `PluginError` - 插件错误
- `ConfigError` - 配置错误
- `AuthenticationError` - 认证错误
- `NetworkError` - 网络错误
- `ValidationError` - 验证错误
- `PermissionError` - 权限错误
- `NotFound` - 资源未找到
- `InvalidInput` - 无效输入
- `OCRError` - OCR 错误
- `PDFError` - PDF 处理错误
- `IoError` - IO 错误
- `Generic` - 通用错误

**日志配置**:
- 使用 `tauri-plugin-tracing` 集成
- 日志文件按周轮转，命名格式: `xuan-brain.YYYY-Www.log`
- 默认日志级别: DEBUG
- 控制台层: 彩色输出，显示文件位置和行号
- 文件层: 详细格式，包含线程信息和 span 事件

## 常用命令

### 开发模式
```bash
yarn dev
````

启动 Vite 开发服务器(端口 1420)。配合 Tauri 使用:

```bash
yarn tauri dev
```

### 构建

```bash
# 构建前端
yarn build

# 构建 Tauri 应用
yarn tauri build
```

**跨平台构建产物**:

- Windows: `.msi` / `.exe` (Inno Setup)
- macOS: `.dmg` / `.app` bundle
- Linux: `.AppImage` / `.deb` / `.rpm`

### Rust 后端命令

```bash
# 运行测试
cd src-tauri && cargo test

# 代码检查
cd src-tauri && cargo clippy

# 格式化代码
cd src-tauri && cargo fmt

# 安全扫描(检查依赖漏洞)
cd src-tauri && cargo audit
```

## Tauri Commands 开发

### 定义新 Command

在 `src-tauri/src/command/` 中对应的命令文件中定义:

```rust
use tauri::State;
use sea_orm::DatabaseConnection;
use crate::sys::error::Result;

#[tauri::command]
pub async fn my_command(
    param: String,
    db: State<'_, DatabaseConnection>
) -> Result<String> {
    // 使用 tracing 记录日志
    tracing::info!("Executing my_command with param: {}", param);

    // 业务逻辑处理...

    Ok(format!("Received: {}", param))
}
```

### 注册 Command

1. 在 `src-tauri/src/command/mod.rs` 中导出命令模块
2. 在 `src-tauri/src/lib.rs` 中导入并注册:

```rust
use crate::command::my_module::my_command;

// 在 run() 函数中
.invoke_handler(tauri::generate_handler![
    // ... 其他命令
    my_command
])
```

### 前端调用

```typescript
// Helper function for lazy loading Tauri API
async function invokeCommand<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd, args);
}

// 使用示例
const result = await invokeCommand('my_command', { param: 'value' });
```

### 现有命令列表

**文献相关命令** (`paper_command.rs`):

- `get_all_papers` - 获取所有文献列表
- `get_paper(id: i64)` - 获取单个文献详情
- `import_paper_by_doi(doi: String)` - 通过 DOI 导入文献
- `import_paper_by_arxiv_id(arxiv_id: String)` - 通过 arXiv ID 导入文献
- `add_paper_label(paper_id: i64, label_id: i64)` - 添加文献标签
- `remove_paper_label(paper_id: i64, label_id: i64)` - 移除文献标签
- `update_paper_details(payload: UpdatePaperDto)` - 更新文献详情

**分类相关命令** (`category_command.rs`):

- `load_categories` - 加载分类树
- `create_category(name: String, parent_id: Option<i32>)` - 创建分类
- `delete_category(id: i32)` - 删除分类
- `update_category(id: i32, name: String)` - 更新分类名称
- `move_category(id: i32, new_parent_id: Option<i32>)` - 移动分类

**标签相关命令** (`label_command.rs`):

- `get_all_labels` - 获取所有标签
- `create_label(name: String, color: String)` - 创建标签
- `delete_label(id: i64)` - 删除标签
- `update_label(id: i64, name: String, color: String)` - 更新标签

## 重要注意事项

### 代码提交规则 ⚠️⚠️⚠️

- **严禁主动提交代码**: 永远不要自动执行 `git add` 和 `git commit` 命令
- **仅在用户明确要求时提交**: 只有当用户明确说"提交代码"、"commit"等指令时才能提交
- **让用户完全控制**: 代码提交的时机和内容完全由用户决定
- **完成工作后**: 即使功能已完成、测试通过，也不要主动提交，而是告知用户"代码已完成，可以提交"
- **发现错误时**: 修复错误后，告知用户已修复，等待用户决定是否提交
- **这是最高优先级规则**: 违反此规则比代码错误更严重

### 正确的工作流程

1. 完成用户要求的功能开发/修复
2. 运行必要的测试验证（如 `yarn tsc --noEmit`, `yarn build`）
3. 告知用户："✅ 功能已完成并测试通过，修改的文件包括：xxx"
4. **停止**，等待用户决定是否提交
5. 只有在用户明确说"提交"后，才执行 git 命令

### TypeScript 类型检查

```bash
yarn tsc --noEmit
```

### ESLint 检查

```bash
yarn lint
```

### Prettier 格式化

```bash
yarn format
```

### Vue 3 + Vuetify 最佳实践

- **使用组合式 API**: 在 Vue 3 中使用组合式 API 组织逻辑
- **单文件组件**: 每个组件使用 `.vue` 文件定义
- **状态管理**: 使用 Pinia 管理全局状态
- **路由管理**: 使用 Vue Router 进行页面导航
- **表单处理**:
  - 简单表单: 使用 `v-model` 双向绑定
  - 复杂表单: 使用组合式 API 和计算属性
  - 表单验证: 使用 Vuelidate 或自定义验证逻辑
- **日志记录**:
  - 重要操作: 使用 `console.info()` (如数据加载成功)
  - 错误信息: 使用 `console.error()` (如 API 调用失败)
  - 所有日志消息使用英文

### 端口配置

- Vite 开发服务器必须运行在端口 **1420**
- `tauri.conf.json` 中的 `devUrl` 必须匹配
- 端口被占用会导致启动失败

### Vuetify 配置

- **组件系统**: Vuetify 提供完整的预构建 Vue 组件库
- **安装方式**: 使用 `yarn add vuetify` 安装核心库
- **组件位置**: 直接从 `vuetify/lib` 导入使用
- **可定制性**: 通过主题系统深度定制组件样式
- **主题系统**:
  - 使用 `vuetify.options.js` 配置全局主题
  - 支持暗色模式切换 (使用 `theme.dark`)
  - 支持自定义颜色、字体、间距等
- **常用组件**:
  - `v-btn`, `v-input`, `v-select`, `v-checkbox`, `v-radio`
  - `v-dialog`, `v-dropdown`, `v-menu`, `v-popover`
  - `v-snackbar`, `v-alert`, `v-card`
  - `v-data-table`, `v-tabs`, `v-tooltip`
- **Tree View**: `v-treeview` 提供高性能树形组件
- **数据表格**: `v-data-table` 提供强大的数据表格功能

**主题切换示例**:

```typescript
// src/theme.ts
import { createVuetify } from 'vuetify';
import 'vuetify/styles';

export default createVuetify({
  theme: {
    defaultTheme: 'light',
    themes: {
      light: {
        colors: {
          primary: '#1976d2',
          secondary: '#424242',
          accent: '#82B1FF',
          error: '#FF5252',
          info: '#2196F3',
          success: '#4CAF50',
          warning: '#FFC107',
        },
      },
      dark: {
        colors: {
          primary: '#90caf9',
          secondary: '#212121',
          accent: '#039be5',
          error: '#e57373',
          info: '#64b5f6',
          success: '#81c784',
          warning: '#ffd54f',
        },
      },
    },
  },
});
```

```tsx
// src/main.ts
import { createApp } from 'vue';
import App from './App.vue';
import vuetify from './plugins/vuetify';
import { createPinia } from 'pinia';
import router from './router';

const app = createApp(App);

app.use(vuetify);
app.use(createPinia());
app.use(router);

app.mount('#app');
```

**Headless Tree 组件示例**:

```tsx
import { createTree, type TreeOptions } from '@headless-tree/core';
import { UncontrolledTreeEnvironment } from '@headless-tree/react';
import type { TreeItemIndex } from '@headless-tree/core';

interface CategoryNode {
  id: string;
  name: string;
  children?: CategoryNode[];
}

export function CategoryTree({ categories }: { categories: CategoryNode[] }) {
  const treeOptions: TreeOptions<CategoryNode> = {
    dataProvider: {
      getTreeItem: (item: TreeItemIndex) => {
        const node = categories.find((c) => c.id === item);
        return node ? { data: node } : undefined;
      },
      getTreeItems: () => categories.map((c) => c.id),
      getChildren: (item: TreeItemIndex) => {
        const node = categories.find((c) => c.id === item);
        return node?.children?.map((c) => c.id) ?? [];
      },
    },
    renderItem: ({ item }) => <div>{item.data.name}</div>,
  };

  const tree = createTree(treeOptions);

  return <UncontrolledTreeEnvironment>{tree.render()}</UncontrolledTreeEnvironment>;
}
```

### Tailwind CSS 4 配置

- **PostCSS**: 使用 `@tailwindcss/postcss` 进行处理
- **配置文件**: `tailwind.config.js` 使用标准格式
- **样式文件**: 在 `src/index.css` 中定义全局样式和 CSS 变量
- **暗色模式**: 结合 Vuetify 的主题系统使用

**配置示例**:

```javascript
// tailwind.config.js
export default {
  content: ['./src/**/*.{html,js,jsx,ts,tsx}'],
  theme: {
    extend: {},
  },
  plugins: [],
};
```

```css
/* src/index.css */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

/* CSS Variables for Light Mode */
:root {
  --ant-color-border: #d9d9d9;
  --ant-color-border-secondary: #f0f0f0;
  --ant-color-bg-layout: #f5f5f5;
  --ant-color-bg-container: #ffffff;
  --ant-color-text: rgba(0, 0, 0, 0.88);
}

/* CSS Variables for Dark Mode */
[data-mode='dark'] {
  --ant-color-border: #424242;
  --ant-color-border-secondary: #303030;
  --ant-color-bg-layout: #141414;
  --ant-color-bg-container: #1f1f1f;
  --ant-color-text: rgba(255, 255, 255, 0.85);
}

#root {
  width: 100%;
  height: 100vh;
}

/* Headless Tree Styles */
.tree-item {
  cursor: pointer;
  transition: background-color 150ms;
}

.tree-item:hover {
  background-color: var(--ant-color-fill-alter);
}

.tree-item[aria-selected='true'] {
  background-color: var(--ant-color-primary-bg);
}
```

### 文件监听

- Vite 配置忽略监听 `src-tauri/**` 目录
- Rust 代码更改会自动触发 Tauri 重新编译

### 安全性考虑

#### 数据加密

- 所有网络通信必须使用 HTTPS + TLS 1.2/1.3
- 敏感数据必须加密存储(AES)
- 云同步数据使用端到端加密
- 使用 `rust-crypto-utils` (AES、RSA)

#### 访问控制

- 插件需要权限声明和沙箱隔离
- API 使用 OAuth2 或令牌认证
- 实施 CORS 策略限制访问

#### 代码安全

- 避免使用 `unsafe` 代码除非绝对必要
- 对所有用户输入进行验证和清理
- 防止注入攻击(SQL 注入、命令注入)
- 定期运行 `cargo audit` 扫描依赖漏洞
- 在 CI 中集成安全扫描
- 对核心模块进行模糊测试和渗透测试

### 跨平台适配

#### Windows 平台

- 提供文件关联功能(双击 PDF 导入)
- 处理特殊路径和权限问题
- 使用 WebView2 控件
- 安装程序: .msi 或 Inno Setup .exe

#### macOS 平台

- 代码签名(避免 Gatekeeper 警告)
- 沙箱权限声明
- 支持 Retina 显示
- 触摸板和输入法支持
- 安装包: .dmg 磁盘映像或 .app bundle

#### Linux 平台

- 支持 AppImage/Snap/.deb/.rpm 格式
- 与 GNOME/KDE 桌面环境集成
- 文件关联注册
- 遵循开源协议，提供源代码

## 项目结构

```
src/                         # React 前端
├── main.tsx                # React 应用入口
├── App.tsx                 # 主应用组件 (路由配置)
├── app.html                # HTML 模板
├── theme.ts                # Ant Design 主题配置
├── index.css               # 全局样式和 Tailwind CSS
├── components/             # 可复用组件
│   ├── layout/             # 布局组件
│   │   ├── Layout.tsx      # 主布局组件 (三栏布局)
│   │   ├── Layout.css      # 布局样式
│   │   └── StatusBar.tsx   # 状态栏组件
│   ├── navigation/         # 导航组件
│   │   ├── Navigation.tsx   # 侧边导航栏（包含分类树和标签）
│   │   ├── CategoryTree.tsx# 文献库分类树组件 (Headless Tree)
│   │   └── TagsSection.tsx # 标签区域组件
│   ├── dialogs/            # 对话框组件
│   │   ├── AddCategoryDialog.tsx  # 添加分类对话框
│   │   ├── EditCategoryDialog.tsx # 编辑分类对话框
│   │   └── AddTagDialog.tsx      # 添加标签对话框
│   └── document/           # 文档相关组件
│       ├── DocumentList.tsx       # 文档列表组件
│       └── DocumentDetails.tsx    # 文档详情组件
├── lib/                    # 工具库
│   └── i18n/              # 国际化
│       ├── index.tsx       # i18n 提供器和 hook
│       ├── en.ts           # 英文翻译
│       └── zh.ts           # 中文翻译
├── stores/                 # Zustand 状态管理
│   └── useAppStore.ts     # 应用全局状态 store
vite.config.js              # Vite 配置
tsconfig.json               # TypeScript 配置
tailwind.config.js          # Tailwind CSS 配置
postcss.config.js           # PostCSS 配置
package.json                # Node.js 依赖
index.html                  # HTML 入口
src-tauri/                  # Rust 后端
├── src/
│   ├── main.rs           # 应用入口
│   ├── lib.rs            # Tauri commands 和应用逻辑
│   ├── command/          # Tauri commands
│   │   ├── mod.rs
│   │   ├── paper_command.rs    # 文献相关命令
│   │   ├── category_command.rs # 分类相关命令
│   │   └── label_command.rs    # 标签相关命令
│   ├── database/         # 数据库层
│   │   ├── mod.rs
│   │   └── entities/     # SeaORM 实体
│   │       ├── mod.rs
│   │       ├── prelude.rs
│   │       ├── papers.rs
│   │       ├── authors.rs
│   │       ├── keywords.rs
│   │       ├── labels.rs
│   │       ├── category.rs
│   │       ├── attachments.rs
│   │       ├── paper_authors.rs
│   │       ├── paper_keywords.rs
│   │       └── paper_labels.rs
│   ├── service/          # 业务逻辑层
│   │   └── category_service.rs # 分类服务
│   ├── papers/           # 文献处理模块
│   │   └── mod.rs
│   └── sys/              # 系统相关功能
│       ├── mod.rs
│       ├── config.rs      # 配置管理
│       ├── consts.rs      # 常量定义
│       ├── dirs.rs        # 目录管理
│       ├── error.rs       # 错误处理
│       └── log.rs         # 日志配置
├── migration/             # 数据库迁移
│   ├── Cargo.toml        # 迁移项目配置
│   └── src/
│       └── lib.rs        # SeaORM 迁移脚本
├── Cargo.toml            # Rust 依赖
└── tauri.conf.json       # Tauri 配置
docs/                       # 项目文档
├── introduction.md       # 项目介绍(架构设计)
├── user-guide.md         # 用户手册(待创建)
├── developer-guide.md    # 开发者指南(待创建)
└── plugin-development.md # 插件开发指南(待创建)
```

## 测试策略

### 单元测试

```bash
# Rust 后端单元测试
cd src-tauri && cargo test

# 运行特定测试
cd src-tauri && cargo test test_init_logger

# 运行特定模块的测试
cd src-tauri && cargo test --package xuan-brain --lib sys
```

**覆盖范围**:

- 系统模块（日志、错误处理、目录管理）
- 数据库操作和迁移
- 文献导入器（DOI、arXiv）
- 使用 `tempfile` 进行临时文件测试

### 集成测试

```bash
# Tauri 集成测试(模拟前端调用)
cd src-tauri && cargo test --test integration
```

**测试场景**:

- 完整文献导入流程（DOI/arXiv）
- 数据库事务和关系操作
- 命令层与数据库层交互

### 端到端测试 (E2E)

- 使用 Playwright 进行 E2E 测试（待实现）
- 模拟真实用户操作
- 跨平台测试(Windows/macOS/Linux)
- 测试场景: 导入文献 → 添加标签 → 搜索文献 → 验证保存

### 代码质量检查

```bash
# Rust 代码检查
cd src-tauri && cargo clippy

# Rust 代码格式化
cd src-tauri && cargo fmt

# TypeScript 类型检查
yarn tsc --noEmit

# Rust 依赖安全扫描
cd src-tauri && cargo audit
```

### CI/CD 流程

**GitHub Actions 工作流**:

1. **编译检查**: Linux、Windows、macOS 三平台编译
2. **运行测试**: 单元测试 + 集成测试
3. **代码质量**: `clippy` + `rustfmt` + ESLint
4. **安全扫描**: `cargo audit` 依赖漏洞扫描
5. **构建发布**: 生成各平台安装包并上传到 GitHub Releases

**构建产物**:

- Linux: .deb / .rpm / AppImage
- Windows: .msi
- macOS: .dmg

## 开发指南

### 项目初始化和数据结构

**数据库实体** (SeaORM):

- `papers` - 文献主表
- `authors` - 作者表
- `keywords` - 关键词表
- `labels` - 标签表
- `category` - 分类表
- `attachments` - 附件表
- `paper_authors` - 文献-作者关联表
- `paper_keywords` - 文献-关键词关联表
- `paper_labels` - 文献-标签关联表

**前端状态管理** (Zustand):

- `useAppStore` - 应用全局状态（主题、accentColor、选中的文档）

**前端页面结构**:

- 三栏布局：左侧导航（分类树）、中间文档列表、右侧文档详情
- 可拖拽调整各栏宽度
- 底部状态栏显示统计信息

**国际化**:

- 自定义 i18n 实现，支持中文和英文
- 翻译文件位于 `src/lib/i18n/`
- 使用 `useI18n()` hook 获取翻译函数 `t()`
- 支持动态切换语言，自动持久化到 localStorage

### 添加新的文献格式支持

### 添加新的文献导入源

1. 在 `src-tauri/src/papers/importer/` 中创建新模块
2. 实现元数据获取逻辑（参考 `doi.rs` 和 `arxiv.rs`）
3. 定义错误类型并实现 `From` 转换到 `AppError`
4. 在 `paper_command.rs` 中添加对应的 Tauri command
5. 在前端添加导入 UI 组件

### 添加新的数据库实体

1. 在 `src-tauri/src/database/entities/` 中创建新实体文件
2. 使用 `#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]` 定义实体
3. 在 `src-tauri/migration/src/lib.rs` 中添加迁移脚本
4. 运行 `sea-orm-cli generate entity` 生成实体（如需要）

### 文献库分类管理

**前端组件**:

- `components/navigation/Navigation.tsx` - 导航组件（包含分类树）
  - 使用 `@headless-tree/core` + `@headless-tree/react` 实现树形结构
  - 支持拖拽功能
  - 支持节点展开/折叠和选中
- `components/dialogs/AddCategoryDialog.tsx` - 添加分类对话框
- `components/dialogs/EditCategoryDialog.tsx` - 编辑分类对话框

**后端接口** (Tauri Commands):

**分类管理**:

```typescript
// 加载分类树
loadCategories(): Promise<CategoryNode[]>

// 创建分类
createCategory(name: string, parentId?: number): Promise<void>

// 更新分类名称
updateCategory(id: number, name: string): Promise<void>

// 删除分类（级联删除子节点）
deleteCategory(id: number): Promise<void>

// 移动分类
moveCategory(id: number, newParentId?: number): Promise<void>
```

**标签管理**:

```typescript
// 获取所有标签
getAllLabels(): Promise<Label[]>

// 创建标签
createLabel(name: string, color: string): Promise<void>

// 删除标签
deleteLabel(id: number): Promise<void>

// 更新标签
updateLabel(id: number, name: string, color: string): Promise<void>
```

**文献管理**:

```typescript
// 获取所有文献
getAllPapers(): Promise<Paper[]>

// 获取单个文献详情
getPaper(id: number): Promise<PaperDetail | null>

// 通过 DOI 导入文献
importPaperByDoi(doi: string): Promise<Paper>

// 通过 arXiv ID 导入文献
importPaperByArxivId(arxivId: string): Promise<Paper>

// 添加文献标签
addPaperLabel(paperId: number, labelId: number): Promise<void>

// 移除文献标签
removePaperLabel(paperId: number, labelId: number): Promise<void>

// 更新文献详情
updatePaperDetails(payload: UpdatePaperDto): Promise<void>
```

**数据结构**:

- 使用 `id` 和 `parent_id` 存储分类层级
- 支持无限层级嵌套

- **开发指南**:
- 分类服务位于 `src-tauri/src/service/category_service.rs`
- Tauri commands 定义在 `src-tauri/src/command/category_command.rs`
- 数据库实体在 `src-tauri/src/database/entities/category.rs`
- 使用 SeaORM 的 `load_many_to_many` 加载关联数据
- 使用 `#[instrument(skip(db))]` 宏为命令添加追踪日志

### React 组件开发

- 使用函数组件 + Hooks
- 使用 TypeScript 进行类型检查
- 使用 Ant Design 组件库
- 使用 Tailwind CSS 进行样式定制
- 使用 React useState 管理表单状态
- 使用 Ant Design 组件处理复杂 UI 场景

**示例组件 - 状态管理 (使用 Zustand)**:

```tsx
import { useAppStore } from '@/stores/useAppStore';

export function ThemeSwitcher() {
  const { isDark, accentColor, toggleTheme } = useAppStore();

  return <Button onClick={toggleTheme}>切换到{isDark ? '亮色' : '暗色'}主题</Button>;
}
```

**示例组件 - 调用 Tauri Commands**:

```tsx
import { useState, useEffect } from 'react';
import { Spin } from 'antd';
import { invoke } from '@tauri-apps/api/core';

interface CategoryNode {
  id: number;
  name: string;
  children?: CategoryNode[];
}

// Helper function for lazy loading Tauri API
async function invokeCommand<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd, args);
}

export function CategoryTree() {
  const [categories, setCategories] = useState<CategoryNode[]>([]);
  const [loading, setLoading] = useState(false);

  const loadCategories = async () => {
    setLoading(true);
    try {
      const data = await invokeCommand<CategoryNode[]>('load_categories');
      setCategories(data);
      console.info('Categories loaded successfully:', data.length);
    } catch (error) {
      console.error('Failed to load categories:', error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadCategories();
  }, []);

  return (
    <div>
      {loading && <Spin size="small" />}
      {/* 渲染分类树 */}
    </div>
  );
}
```

**表单处理 (使用 useState + Ant Design)**:

```tsx
import { useState, useEffect } from 'react';
import { Modal, Input, Button, Typography, Alert } from 'antd';
import { CloseOutlined } from '@ant-design/icons';
import { useI18n } from '../../lib/i18n';

// Helper function for lazy loading Tauri API
async function invokeCommand<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd, args);
}

interface AddCategoryDialogProps {
  open: boolean;
  onClose: () => void;
  onCategoryCreated: () => void;
  parentPath?: string;
  parentName?: string;
}

export default function AddCategoryDialog({
  open,
  onClose,
  onCategoryCreated,
  parentPath,
  parentName,
}: AddCategoryDialogProps) {
  const { t } = useI18n();
  const [name, setName] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  // Reset form when dialog opens
  useEffect(() => {
    if (open) {
      setName('');
      setError('');
    }
  }, [open]);

  const handleClose = () => {
    setName('');
    setError('');
    onClose();
  };

  const handleSubmit = async () => {
    if (!name.trim()) {
      setError(t('dialog.categoryNameRequired'));
      return;
    }

    if (name.length > 50) {
      setError(t('dialog.categoryNameMaxLength'));
      return;
    }

    setLoading(true);
    try {
      await invokeCommand('create_category', {
        name: name.trim(),
        parentPath: parentPath || null,
      });
      console.info('Category created successfully:', name.trim());
      setName('');
      setError('');
      onCategoryCreated();
      handleClose();
    } catch (err) {
      setError(err as string);
    } finally {
      setLoading(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !loading && name.trim() && name.length <= 50) {
      handleSubmit();
    }
  };

  return (
    <Modal
      open={open}
      onCancel={handleClose}
      title={
        <div style={{ position: 'relative', paddingRight: 32 }}>
          <Typography.Text strong>
            {parentPath ? t('dialog.addSubcategory') : t('dialog.addCategory')}
          </Typography.Text>
        </div>
      }
      closeIcon={<CloseOutlined />}
      width={480}
      footer={
        <>
          <Button onClick={handleClose} disabled={loading}>
            {t('dialog.cancel')}
          </Button>
          <Button
            type="primary"
            onClick={handleSubmit}
            loading={loading}
            disabled={!name.trim() || name.length > 50}
          >
            {t('dialog.add')}
          </Button>
        </>
      }
    >
      {error && <Alert message={error} type="error" showIcon style={{ marginBottom: 16 }} />}
      <div style={{ marginBottom: 16 }}>
        <Input
          autoFocus
          placeholder={t('dialog.enterCategoryName')}
          value={name}
          onChange={(e) => {
            setName(e.target.value);
            setError('');
          }}
          onPressEnter={handleKeyPress}
          status={error ? 'error' : ''}
          disabled={loading}
        />
        {error && (
          <Typography.Text type="danger" style={{ fontSize: 12 }}>
            {error}
          </Typography.Text>
        )}
      </div>
      {parentName && (
        <div style={{ marginBottom: 16 }}>
          <div style={{ marginBottom: 4 }}>
            <Typography.Text type="secondary">{t('dialog.parentCategory')}</Typography.Text>
          </div>
          <Input value={parentName} disabled />
        </div>
      )}
      <Typography.Text type="secondary" style={{ fontSize: 12 }}>
        {t('dialog.categoryNameRules')}
      </Typography.Text>
    </Modal>
  );
}
```

### 开发插件

- **Rust 插件**: 实现定义的 trait，编译为动态库(.so/.dll)
  - 使用 `libloading` 加载
  - 实现稳定的 ABI 接口
  - 性能接近原生代码
- **JavaScript 插件**: 使用提供的 API，通过沙箱环境执行
  - 使用 QuickJS 或 V8 引擎
  - 丰富的 JavaScript 生态
  - 快速开发和 UI 扩展
- 详见 `src-tauri/src/plugins/` 和开发者文档

### API 集成

- 所有 API 端点在 `src-tauri/src/api/routes.rs` 中定义
- 遵循 RESTful 规范
- 使用 OAuth2 或令牌认证
- 参考 API 文档: `docs/api.md`(待创建)

**应用场景**:

- 批量导入文献
- 定期查询 ArXiv 最新论文
- 文献引用工具集成
- 写作辅助软件集成
- 文献计量分析

## 推荐开发工具

- **IDE**: VS Code
- **扩展**:
  - ES7+ React/Redux/React-Native snippets
  - Tauri
  - rust-analyzer
  - Even Better TOML(Cargo.toml 支持)
  - Error Lens(内联错误显示)
  - Tailwind CSS IntelliSense

- **调试工具**:
  - CrabNebula DevTools (Tauri 应用调试)
  - React DevTools (React 组件调试)
  - Chrome DevTools (前端调试)

## 参考资源

### 官方文档

- [Tauri 官方文档](https://tauri.app/)
- [React 官方文档](https://react.dev/)
- [React Router 文档](https://reactrouter.com/)
- [Ant Design 文档](https://ant.design/)
- [Headless Tree 文档](https://headless-tree.com/)
- [Rust 官方文档](https://www.rust-lang.org/)
- [SeaORM 文档](https://www.sea-ql.org/SeaORM/)

### 项目文档

- [项目详细介绍](docs/introduction.md) - 完整的架构设计文档
- [用户手册](docs/user-guide.md) - 安装、配置和使用教程(待创建)
- [开发者文档](docs/developer-guide.md) - API 文档、架构说明(待创建)
- [插件开发指南](docs/plugin-development.md) - 插件开发教程(待创建)

### 技术参考

- [SeaORM 文档](https://www.sea-ql.org/SeaORM/)
- [SeaORM Migration 文档](https://www.sea-ql.org/SeaORM/docs/migration)
- [Tailwind CSS 文档](https://tailwindcss.com/)
- [Zustand 文档](https://zustand-demo.pmnd.rs/)
- [Tauri 2.x 文档](https://v2.tauri.app/)
- [Tauri Plugin Tracing 文档](https://github.com/FabianLars/tauri-plugin-tracing)
- [React 19 文档](https://react.dev/)
- [Ant Design 文档](https://ant.design/)
- [Headless Tree 文档](https://headless-tree.com/)
- [tracing 文档](https://docs.rs/tracing/)

## 社区与贡献

### 交流渠道

- **GitHub**: 项目仓库、Issues、Discussions
- **社区平台**: Reddit、Stack Overflow、Discord
- **中文社区**: Gitee、知乎

### 问题跟踪

- **GitHub Issues**: Bug 报告和功能请求
- **标签分类**: bug、enhancement、documentation
- **优先级**: 严重 bug 优先处理

### 贡献指南

欢迎社区贡献！请参考:

1. Fork 项目仓库
2. 创建特性分支
3. 编写测试用例
4. 确保通过所有检查(`cargo test`, `cargo clippy`, `yarn lint`, `yarn test`)
5. 提交 Pull Request

**代码规范**:

- Rust: 遵循 `rustfmt` 格式化，通过 `clippy` 检查
- TypeScript: 遵循 ESLint 规则
- 提交前运行 `yarn lint` 和 `cargo test`
- 所有的日志内容均为英文

**开源许可**: MIT / Apache 2.0(待定)

### 发布节奏

- 规律发布: 新功能或重要修复时发布
- 发布公告: 说明更新内容
- 社区感谢: 认可突出贡献者

## 开发路线图

### v0.1.0 (当前版本)

- [x] 基础文献导入和管理
- [x] 分类树管理
- [x] 标签管理
- [x] 三栏布局（导航、列表、详情）
- [x] 主题切换（亮色/暗色）
- [x] 本地数据存储（SeaORM + SQLite）
- [x] React 19 + Ant Design + Tailwind CSS 4

### v0.2.0 (计划中)

- [ ] 文献导入（PDF、DOCX 等本地文件）
- [ ] 搜索和过滤功能
- [ ] 作者管理
- [ ] 关键词管理
- [ ] 附件管理
- [ ] PDF 阅读器实现
- [ ] 设置页面

### v0.3.0 (未来版本)

- [ ] AI 智能推荐和分类
- [ ] 关键词自动提取
- [ ] 笔记和标注功能
- [ ] 引用格式生成
- [ ] 云同步服务
- [ ] WebDAV 支持
- [ ] 插件系统
- [ ] 知识图谱可视化
- [ ] RESTful API

---

**其他注意事项**

- 不要主动提交代码
- 所有日志内容应为英文
- 遵循 ESLint 和 Prettier 代码规范
- 组件使用函数组件 + Hooks
- 使用 TypeScript 进行类型检查
- 表单使用 React useState + Ant Design 组件管理
- 全局状态使用 Zustand 管理
- 国际化使用自定义 i18n 实现(支持中英文)
- 所有日志内容应为英文
- 使用 `console.info()` 记录重要操作(如数据加载成功)
- 使用 `console.error()` 记录错误信息

**最后更新**: 2025-01-22
