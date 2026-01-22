# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

**璇玑 (xuan-brain)** 是一个基于 **Tauri 2.x + React 18 + TypeScript** 构建的 AI 驱动科研文献管理桌面应用。本设计借鉴 Zotero 的核心理念，通过插件机制提供功能强大且易于使用的文献管理平台。

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
- **Tauri**: 相比 Electron 更轻量(体积小 80%)、更安全(Rust 内存安全)、性能更好
- **React 18**: 成熟的生态系统、丰富的第三方库、强大的社区支持
  - **组件模型**: 函数组件 + Hooks，声明式编程
  - **虚拟 DOM**: 高效的 diff 算法和渲染优化
  - **并发模式**: 支持后台渲染和可中断渲染
- **Material-UI (MUI)**: 企业级 React UI 组件库
  - **丰富的组件**: 100+ 预构建组件，涵盖所有常见 UI 需求
  - **Material Design**: 遵循 Google Material Design 规范，界面美观一致
  - **高质量**: 优秀的代码质量、可访问性和国际化支持
  - **主题系统**: 强大的主题定制能力，支持暗色模式
  - **Tree View**: 内置功能完善的树形组件 (`@mui/x-tree-view`)
  - **Data Grid**: 高级数据表格组件 (`@mui/x-data-grid`)
  - **生态系统**: 丰富的相关库 (DatePicker, Charts, Toast Notifications)
- **React Router v7**: 事实标准的 React 路由库
- **Zustand**: 轻量级状态管理库
- **React Query (TanStack Query)**: 强大的服务端状态管理
- **Emotion**: CSS-in-JS 库，MUI 默认样式引擎
- **Tailwind CSS 4**: 可选的实用优先 CSS 框架，用于部分样式定制
- **Rust 后端**: 高性能 PDF 处理、文件 I/O、AI 模型推理
- **SQLite**: 嵌入式数据库、零配置、单文件备份

### 设计理念
- **模块化组织**: 每个功能模块独立开发和测试
- **性能保障**: Rust 的出色性能和并发能力
- **内存安全**: 降低内存相关漏洞风险
- **代码可维护性**: 清晰的模块划分

## 核心架构

### 前端 (React)
- **框架**: React 18.x 使用函数组件和 Hooks
- **构建工具**: Vite 6.x (快速的开发服务器和构建工具)
- **渲染模式**: SPA (单页应用)，无需 SSR
- **路由**: React Router v7，位于 `src/routes/` 或 `src/pages/`
- **开发服务器**: 运行在 `http://localhost:1420`
- **状态管理**:
  - **客户端状态**: Zustand (轻量级全局状态)
  - **服务端状态**: React Query (异步数据获取和缓存)
  - **表单状态**: React Hook Form + Zod (类型安全的表单验证)
- **UI 组件库**: Material-UI (MUI)
  - 核心组件: `@mui/material`
  - 树形组件: `@mui/x-tree-view`
  - 数据表格: `@mui/x-data-grid`
  - 日期选择: `@mui/x-date-pickers`
  - 图标: `@mui/icons-material`
- **样式**: Emotion (MUI 默认) + Tailwind CSS 4 (可选)
- **图标**: Material Icons (MUI 内置) 或 Lucide React
- **职责**:
  - 文献列表展示和 UI 交互
  - 搜索和筛选界面
  - 笔记和标签管理
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

### 1. 文献解析器 (Document Parser)

```
src-tauri/src/parser/
├── mod.rs           # 解析器入口
├── pdf.rs           # PDF 解析(pdfium/Poppler)
├── docx.rs          # DOCX 解析
├── html.rs          # HTML/网页解析
├── epub.rs          # EPUB 解析
└── ocr.rs           # OCR 功能(Tesseract FFI)
```

**支持格式**:
- **PDF**: 处理复杂文档结构(章节、分栏、公式)，支持扫描版 OCR
- **DOCX**: 提取 XML 内容，解析元数据
- **HTML**: DOM 解析，支持 arXiv、期刊网站
- **EPUB**: 电子书格式解析
- **其他**: LaTeX 源码、图片 OCR

**解析流程**:
1. 格式识别
2. 内容提取(文本 + 元数据)
3. 数据规范化
4. 存储到数据库
5. 特殊内容处理(注释、图表)

**依赖库**:
- PDF: `pdfium` / Poppler / MuPDF
- DOCX: `docx` 库
- HTML: `select` 库
- EPUB: `epub` 库
- OCR: Tesseract (FFI 调用)

### 2. AI 引擎 (AI Engine)

```
src-tauri/src/ai/
├── mod.rs           # AI 引擎入口
├── recommend.rs     # 智能推荐(基于向量相似度)
├── classify.rs      # 自动分类
├── keywords.rs      # 关键词提取(CRF/NLP)
└── summary.rs       # 文献摘要(LLM 集成)
```

**核心功能**:
- **智能推荐**: 内容相似度计算、向量表示、引用关系分析
- **自动分类**: 机器学习分类模型、主题识别
- **关键词提取**: NLP 技术、条件随机场(CRF)
- **文献摘要**: LLM 集成、论点提炼、问答功能

**技术栈**:
- `ndarray` / `ndarray-stats`: 张量运算和线性代数
- `tch-rs`: PyTorch 绑定(神经网络)
- `linfa`: Rust 机器学习框架
- Python FFI: 调用 PyTorch/TensorFlow 模型

### 3. 数据库层 (Database)

```
src-tauri/src/db/
├── mod.rs           # 数据库入口
├── models.rs        # 数据模型定义
├── schema.rs        # 数据库表结构
└── queries.rs       # 查询操作
```

**数据存储**:
- 文献元数据(标题、作者、出版年份)
- 摘要、标签、关键词
- 用户笔记和标注
- 全文检索索引
- 插件相关数据

**技术栈**:
- SQLite(嵌入式数据库)
- `sqlx`(异步数据库接口)
- 全文检索支持

**优势**:
- 轻量级、零配置
- 单文件备份和迁移
- Rust 生态集成良好

### 4. 同步模块 (Sync)

```
src-tauri/src/sync/
├── mod.rs           # 同步模块入口
├── cloud.rs         # 云端同步服务
├── webdav.rs        # WebDAV 客户端
└── encrypt.rs       # 端到端加密
```

**同步方案**:
- **云端同步**: 官方服务，端到端加密，增量同步，冲突解决
- **WebDAV**: 支持 NAS、坚果云、Dropbox、OwnCloud

**技术栈**:
- `rustls`: TLS 安全传输
- `davcli` / `hyper`: WebDAV 客户端
- `rust-crypto-utils`: AES/RSA 加密
- `tauri-plugin-upload`: 文件上传

**特性**:
- 双向同步
- 差异检测
- 断点续传
- 冲突检测(时间戳 + 文件锁定)

### 5. 插件系统 (Plugin System)

```
src-tauri/src/plugins/
├── mod.rs           # 插件管理器
├── rust_plugin.rs   # Rust 插件加载器(.so/.dll)
├── js_plugin.rs     # JavaScript 插件引擎(QuickJS/V8)
└── api.rs           # 插件 API 接口
```

**插件类型**:
- **Rust 插件**: 高性能、动态库加载(.so/.dll)、原生速度
- **JavaScript 插件**: Web 技术栈、快速开发、UI 注入能力

**技术栈**:
- `libloading`: 动态库加载
- `QuickJS`/`V8`: JavaScript 引擎
- FFI 接口: 暴露必要的 API
- 安全机制: CSP 和权限控制

**插件应用场景**:
- 新的文献导入源
- 自定义引用格式
- 文献数据可视化
- 与外部学术服务集成(arXiv、Google Scholar)

**安全机制**:
- 统一接口
- 权限声明
- 安全校验
- 能力限制(借鉴 Tauri 安全模型)

### 6. API 服务器 (API Server)

```
src-tauri/src/api/
├── mod.rs           # API 服务器入口
├── routes.rs        # RESTful 路由定义
├── auth.rs          # OAuth2/令牌认证
└── middleware.rs    # CORS/安全中间件
```

**API 应用场景**:
- 自动化脚本与工具集成
- 第三方应用集成(文献引用工具、写作辅助软件)
- 插件开发(独立 Web 服务)

**API 设计**:
- 协议: RESTful API
- 格式: JSON 数据
- 方法: GET、POST、PUT、DELETE
- 认证: OAuth2 或令牌认证
- 安全: HTTPS + CORS 策略

## 常用命令

### 开发模式
```bash
yarn dev
```
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

### 类型检查
```bash
# TypeScript 类型检查
yarn tsc --noEmit

# ESLint 检查
yarn lint

# Prettier 格式化
yarn format
```

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
在 `src-tauri/src/lib.rs` 中:

```rust
#[tauri::command]
fn my_command(param: &str) -> String {
    format!("Received: {}", param)
}
```

### 注册 Command
在 `run()` 函数的 `invoke_handler` 中添加:
```rust
.invoke_handler(tauri::generate_handler![greet, my_command])
```

### 前端调用
```typescript
import { invoke } from "@tauri-apps/api/core";

const result = await invoke("my_command", { param: "value" });
```

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

### React Hooks 最佳实践
- **使用函数组件**: 避免类组件，使用函数组件 + Hooks
- **自定义 Hooks**: 提取可复用的逻辑到自定义 Hooks
- **依赖数组**: 正确使用 `useEffect`, `useMemo`, `useCallback` 的依赖数组
- **状态管理**:
  - 局部状态: `useState`, `useReducer`
  - 全局状态: Zustand
  - 服务端状态: React Query
  - 表单状态: React Hook Form
- **性能优化**:
  - 使用 `React.memo` 避免不必要的重渲染
  - 使用 `useMemo` 缓存昂贵的计算
  - 使用 `useCallback` 缓存回调函数
  - 使用 `React.lazy` 和 `Suspense` 进行代码分割

### 端口配置
- Vite 开发服务器必须运行在端口 **1420**
- `tauri.conf.json` 中的 `devUrl` 必须匹配
- 端口被占用会导致启动失败

### Material-UI (MUI) 配置
- **组件系统**: MUI 提供完整的预构建 React 组件库
- **安装方式**: 使用 `yarn add @mui/material @emotion/react @emotion/styled` 安装核心库
- **组件位置**: 直接从 `@mui/material` 导入使用
- **可定制性**: 通过主题系统深度定制组件样式
- **主题系统**:
  - 使用 `ThemeProvider` 包裹应用
  - 支持暗色模式切换 (`createTheme({ palette: { mode: 'dark' } })`)
  - 支持自定义颜色、字体、间距等
  - 使用 `CssBaseline` 统一基础样式
- **常用组件**:
  - `Button`, `TextField`, `FormControl`, `Select`, `Checkbox`
  - `Dialog`, `Menu`, `Popover`
  - `Snackbar`, `Alert`, `Card`
  - `Table`, `Tabs`, `Tooltip`
- **Tree View**: `@mui/x-tree-view` 提供功能完善的树形组件
- **数据网格**: `@mui/x-data-grid` 提供高级数据表格功能

**主题切换示例**:
```typescript
// src/theme.ts
import { createTheme, ThemeOptions } from '@mui/material/styles'

const lightTheme: ThemeOptions = {
  palette: {
    mode: 'light',
    primary: {
      main: '#1976d2',
    },
    secondary: {
      main: '#dc004e',
    },
  },
}

const darkTheme: ThemeOptions = {
  palette: {
    mode: 'dark',
    primary: {
      main: '#90caf9',
    },
    secondary: {
      main: '#f48fb1',
    },
  },
}

export { lightTheme, darkTheme }
```

```tsx
// src/App.tsx
import { ThemeProvider, CssBaseline } from '@mui/material'
import { useState } from 'react'
import { lightTheme, darkTheme } from './theme'

export default function App() {
  const [isDark, setIsDark] = useState(true)

  const theme = createTheme(isDark ? darkTheme : lightTheme)

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      {/* 应用内容 */}
    </ThemeProvider>
  )
}
```

**Tree View 组件示例**:
```tsx
import { TreeView } from '@mui/x-tree-view/TreeView'
import { TreeItem } from '@mui/x-tree-view/TreeItem'
import { ExpandMore, ChevronRight } from '@mui/icons-material'

interface CategoryNode {
  path: string
  name: string
  children?: CategoryNode[]
}

export function CategoryTree({ categories }: { categories: CategoryNode[] }) {
  const renderTree = (node: CategoryNode) => (
    <TreeItem key={node.path} nodeId={node.path} label={node.name}>
      {Array.isArray(node.children)
        ? node.children.map((child) => renderTree(child))
        : null}
    </TreeItem>
  )

  return (
    <TreeView
      defaultCollapseIcon={<ExpandMore />}
      defaultExpandIcon={<ChevronRight />}
    >
      {categories.map((cat) => renderTree(cat))}
    </TreeView>
  )
}
```

### Tailwind CSS 4 配置
- **Vite 插件**: 在 `vite.config.ts` 中使用 `@tailwindcss/vite` 插件
- **PostCSS**: 使用 `@tailwindcss/postcss`
- **配置文件**: `tailwind.config.js` 使用新格式
- **CSS 变量**: 在 `src/index.css` 中定义 Tailwind CSS 变量
- **暗色模式**: 使用 `dark:` 前缀，配合 `data-mode="dark"` 属性

**配置示例**:
```javascript
// vite.config.ts
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [tailwindcss(), react()],
  server: {
    port: 1420,
    strictPort: true,
  },
})
```

```css
/* src/index.css */
@import "tailwindcss";

@theme {
  --color-primary: oklch(0.7 0.15 250);
  --color-secondary: oklch(0.65 0.12 200);
  /* 更多颜色定义 */
}

@custom-variant dark (&:where([data-mode="dark"], [data-mode="dark"] *));
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
├── theme.ts                # MUI 主题配置
├── routes/                 # 页面路由 (或 src/pages/)
│   ├── LibraryPage.tsx     # 文献库主页
│   ├── ReaderPage.tsx      # PDF 阅读器
│   └── SettingsPage.tsx    # 设置页面
├── components/             # 可复用组件
│   ├── Layout.tsx          # 主布局组件
│   ├── Navigation.tsx      # 侧边导航栏（包含分类树）
│   ├── CategoryTree.tsx    # 文献库分类树组件 (MUI TreeView)
│   ├── AddCategoryDialog.tsx  # 添加分类对话框 (MUI Dialog)
│   ├── EditCategoryDialog.tsx # 编辑分类对话框 (MUI Dialog)
│   ├── ThemeSwitcher.tsx   # 主题切换组件
│   ├── StatusBar.tsx       # 状态栏组件
│   ├── TagsSection.tsx     # 标签区域组件
│   └── AddTagDialog.tsx    # 添加标签对话框
├── contexts/               # React Context
│   └── ThemeContext.tsx    # 主题切换 Context (可选，MUI ThemeProvider 已足够)
├── hooks/                  # 自定义 Hooks
│   ├── useTheme.ts
│   ├── useCategories.ts    # 分类管理 Hook
│   └── useTags.ts          # 标签管理 Hook
├── lib/                    # 工具库
│   ├── utils.ts           # 通用工具函数
│   └── types.ts           # TypeScript 类型定义
├── stores/                 # Zustand 状态管理
│   └── themeStore.ts      # 主题状态 store
└── index.css               # 全局样式 (可选，MUI 使用 Emotion)
vite.config.ts              # Vite 配置
tsconfig.json               # TypeScript 配置
package.json                # Node.js 依赖
static/                     # 静态资源
src-tauri/                  # Rust 后端
├── src/
│   ├── main.rs           # 应用入口
│   ├── lib.rs            # Tauri commands 和应用逻辑
│   ├── parser/           # 文献解析器
│   │   ├── mod.rs
│   │   ├── pdf.rs
│   │   ├── docx.rs
│   │   ├── html.rs
│   │   ├── epub.rs
│   │   └── ocr.rs
│   ├── ai/               # AI 引擎
│   │   ├── mod.rs
│   │   ├── recommend.rs
│   │   ├── classify.rs
│   │   ├── keywords.rs
│   │   └── summary.rs
│   ├── command/          # Tauri commands
│   │   ├── mod.rs
│   │   ├── label_command.rs       # 标签相关命令
│   │   └── category_command.rs    # 分类相关命令
│   ├── database/         # 数据库层
│   │   ├── entities/     # SeaORM 实体
│   │   │   ├── mod.rs
│   │   │   ├── label.rs
│   │   │   └── category.rs
│   │   └── mod.rs
│   ├── service/          # 业务逻辑层
│   │   ├── mod.rs
│   │   └── category_service.rs    # 分类服务
│   ├── sync/             # 同步模块
│   │   ├── mod.rs
│   │   ├── cloud.rs
│   │   ├── webdav.rs
│   │   └── encrypt.rs
│   ├── plugins/          # 插件系统
│   │   ├── mod.rs
│   │   ├── rust_plugin.rs
│   │   ├── js_plugin.rs
│   │   └── api.rs
│   └── api/              # API 服务器
│       ├── mod.rs
│       ├── routes.rs
│       ├── auth.rs
│       └── middleware.rs
├── Cargo.toml            # Rust 依赖
├── tauri.conf.json       # Tauri 配置
└── build.rs              # 构建脚本(如果需要)
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
cd src-tauri && cargo test test_parser_pdf

# React 前端单元测试
yarn test
```

**覆盖范围**:
- 各个 Rust 后端模块
- 解析器、数据库、AI 算法
- React 组件和 Hooks
- 使用 Vitest + React Testing Library

### 集成测试
```bash
# Tauri 集成测试(模拟前端调用)
cd src-tauri && cargo test --test integration
```

**测试场景**:
- 完整文献导入流程
- 模块间交互验证
- 使用 Tauri 模拟运行时

### 端到端测试 (E2E)
- 使用 Playwright 进行 E2E 测试
- 模拟真实用户操作
- 跨平台测试(Windows/macOS/Linux)
- 测试场景: 导入文献 → 搜索 → 添加笔记 → 验证保存

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

### 添加新的文献格式支持
1. 在 `src-tauri/src/parser/` 中创建新模块
2. 实现解析逻辑，提取文本和元数据
3. 在 `parser/mod.rs` 中注册新解析器
4. 添加对应的单元测试

### 集成新的 AI 功能
1. 在 `src-tauri/src/ai/` 中创建新模块
2. 实现算法逻辑(或集成 Python 模型)
3. 通过 Tauri command 暴露给前端
4. 在前端添加 UI 和交互逻辑

### 文献库分类管理

**前端组件**:
- `CategoryTree.tsx` - 文献库分类树主组件
  - 使用 `@dnd-kit/core` 实现拖拽功能
  - 支持右键上下文菜单（添加、编辑、删除）
  - 支持节点展开/折叠和选中
- `AddCategoryDialog.tsx` - 添加分类对话框
- `EditCategoryDialog.tsx` - 编辑分类对话框

**后端接口**:
```typescript
// 加载分类树
loadCategories(): Promise<CategoryNode[]>

// 创建分类
createCategory(name: string, parentPath?: string): Promise<void>

// 更新分类名称
updateCategory(path: string, name: string): Promise<void>

// 删除分类（级联删除子节点）
deleteCategory(path: string): Promise<void>

// 移动分类
moveCategory(draggedPath: string, targetPath?: string, position: string): Promise<string>
```

**数据结构**:
- 使用 PostgreSQL `ltree` 扩展存储分类路径
- 路径格式: `"1"`, `"1.2"`, `"1.2.3"` 等
- 支持无限层级嵌套

**开发指南**:
- 分类服务位于 `src-tauri/src/service/category_service.rs`
- Tauri commands 定义在 `src-tauri/src/command/category_command.rs`
- 数据库实体在 `src-tauri/src/database/entities/category.rs`

### React 组件开发
- 使用函数组件 + Hooks
- 使用 TypeScript 进行类型检查
- 使用 Material-UI (MUI) 组件库
- 使用 Emotion 进行样式定制 (MUI 内置)
- 使用 React Hook Form 处理表单
- 使用 Zod 进行表单验证

**示例组件 (使用 MUI)**:
```tsx
import { useState } from 'react'
import { Button, CircularProgress, Box } from '@mui/material'
import { invoke } from '@tauri-apps/api/core'

interface CategoryNode {
  path: string
  name: string
  children?: CategoryNode[]
}

export function CategoryTree() {
  const [categories, setCategories] = useState<CategoryNode[]>([])
  const [loading, setLoading] = useState(false)

  const loadCategories = async () => {
    setLoading(true)
    try {
      const data = await invoke<CategoryNode[]>('load_categories')
      setCategories(data)
    } catch (error) {
      console.error('Failed to load categories:', error)
    } finally {
      setLoading(false)
    }
  }

  return (
    <Box>
      <Button
        variant="contained"
        onClick={loadCategories}
        disabled={loading}
        startIcon={loading ? <CircularProgress size={20} /> : null}
      >
        {loading ? 'Loading...' : 'Refresh'}
      </Button>
      {/* 渲染分类树 */}
    </Box>
  )
}
```

**对话框组件示例 (MUI Dialog)**:
```tsx
import { useState } from 'react'
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Button,
} from '@mui/material'
import { invoke } from '@tauri-apps/api/core'

interface AddCategoryDialogProps {
  open: boolean
  onClose: () => void
  onCategoryCreated: () => void
}

export function AddCategoryDialog({
  open,
  onClose,
  onCategoryCreated,
}: AddCategoryDialogProps) {
  const [name, setName] = useState('')
  const [error, setError] = useState('')

  const handleSubmit = async () => {
    if (!name.trim()) {
      setError('名称不能为空')
      return
    }

    try {
      await invoke('create_category', { name: name.trim() })
      setName('')
      setError('')
      onCategoryCreated()
      onClose()
    } catch (err) {
      setError(err as string)
    }
  }

  return (
    <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
      <DialogTitle>添加分类</DialogTitle>
      <DialogContent>
        <TextField
          autoFocus
          margin="dense"
          label="分类名称"
          fullWidth
          variant="outlined"
          value={name}
          onChange={(e) => setName(e.target.value)}
          error={!!error}
          helperText={error}
          sx={{ mt: 2 }}
        />
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>取消</Button>
        <Button onClick={handleSubmit} variant="contained">
          添加
        </Button>
      </DialogActions>
    </Dialog>
  )
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
- [Material-UI (MUI) 文档](https://mui.com/)
- [MUI X Tree View 文档](https://mui.com/x/react-tree-view/)
- [Rust 官方文档](https://www.rust-lang.org/)

### 项目文档
- [项目详细介绍](docs/introduction.md) - 完整的架构设计文档
- [用户手册](docs/user-guide.md) - 安装、配置和使用教程(待创建)
- [开发者文档](docs/developer-guide.md) - API 文档、架构说明(待创建)
- [插件开发指南](docs/plugin-development.md) - 插件开发教程(待创建)

### 技术参考
- [sqlx 文档](https://docs.rs/sqlx/)
- [ndarray 文档](https://docs.rs/ndarray/)
- [linfa 文档](https://docs.rs/linfa/)
- [TanStack Query 文档](https://tanstack.com/query/latest)
- [Zustand 文档](https://zustand-demo.pmnd.rs/)

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
- [x] PDF 解析和元数据提取
- [x] 基础搜索和过滤
- [x] 本地数据存储
- [x] React 前端重构 (进行中)

### v0.2.0 (计划中)
- [ ] AI 智能推荐和分类
- [ ] 关键词自动提取
- [ ] 笔记和标注功能
- [ ] 引用格式生成

### v0.3.0 (未来版本)
- [ ] 云同步服务
- [ ] WebDAV 支持
- [ ] 插件系统
- [ ] 知识图谱可视化
- [ ] RESTful API

---

**其他注意事项**

- 不要主动提交代码

**最后更新**: 2025-01-22
