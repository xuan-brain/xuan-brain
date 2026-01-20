# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

**璇玑 (xuan-brain)** 是一个基于 **Tauri 2.x + SvelteKit 5 + TypeScript** 构建的 AI 驱动科研文献管理桌面应用。本设计借鉴 Zotero 的核心理念，通过插件机制提供功能强大且易于使用的文献管理平台。

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
- **SvelteKit 5 + Svelte 5**: 编译时框架、运行时开销小、runes 响应式系统，提供现代开发体验
- **Skeleton 4.x**: 基于 Tailwind CSS 的自适应设计系统，提供统一的 UI 组件和主题系统
  - **依赖要求**: Svelte 5 + Vite 6 + Tailwind CSS 4
  - **特点**: 丰富的 UI 组件、可定制主题、暗色模式支持、无障碍访问
  - **注意**: Skeleton 4.x 不支持 Svelte 4，必须使用 Svelte 5
- **Tailwind CSS 4**: 最新版本的实用优先 CSS 框架，性能优化和配置简化
- **Rust 后端**: 高性能 PDF 处理、文件 I/O、AI 模型推理
- **SQLite**: 嵌入式数据库、零配置、单文件备份

### 设计理念
- **模块化组织**: 每个功能模块独立开发和测试
- **性能保障**: Rust 的出色性能和并发能力
- **内存安全**: 降低内存相关漏洞风险
- **代码可维护性**: 清晰的模块划分

## 核心架构

### 前端 (SvelteKit)
- **框架**: SvelteKit 5.x 使用 Svelte 5 的新 runes 语法 (`$state`, `$derived` 等)
- **渲染模式**: SPA 模式 (禁用 SSR)，因为 Tauri 没有 Node.js 服务器
- **适配器**: `@sveltejs/adapter-static` with fallback to `index.html"
- **路由**: 文件系统路由位于 `src/routes/`
- **开发服务器**: 运行在 `http://localhost:1420`
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
# 单次检查
yarn check

# 监听模式
yarn check:watch
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

### Svelte 5 Runes 语法
- 使用 `$state()` 代替 `let` 声明响应式变量
- 使用 `$derived()` 计算派生值
- 使用 `$effect()` 处理副作用
- 不要使用 Svelte 4 的 `export let` props 语法(在 `.svelte` 文件中)

### 端口配置
- Vite 开发服务器必须运行在端口 **1420**
- `tauri.conf.json` 中的 `devUrl` 必须匹配
- 端口被占用会导致启动失败

### SSR 禁用
- 所有页面布局必须设置 `export const ssr = false;`
- 这已在 `src/routes/+layout.ts` 中配置

### Skeleton 4.x 配置
- **主题系统**: Skeleton 4.x 提供了完整的主题系统，包括:
  - **颜色主题**: 21 个预设主题 (cerberus, catppuccin, modern 等)
  - **暗色模式**: 通过 `data-mode` 属性控制
  - **主题切换**: 使用 `data-theme` 属性切换颜色主题
- **全局样式**: 在 `src/lib/css/app.css` 中导入所有主题:
  ```css
  @import "tailwindcss";
  /* 导入所有预设主题 */
  @import "@skeletonlabs/skeleton/themes/catppuccin";
  @import "@skeletonlabs/skeleton/themes/cerberus";
  /* ... 其他 19 个主题 */
  
  /* 定义 dark mode 变体 */
  @custom-variant dark (&:where([data-mode="dark"], [data-mode="dark"] *));
  @custom-variant light (&:where([data-mode="light"], [data-mode="light"] *));
  ```
- **HTML 属性**:
  - `data-theme="cerberus"` - 设置颜色主题
  - `data-mode="dark"` - 设置暗色/明亮模式
- **注意**: Skeleton 4.x 要求 Svelte 5，不支持 Svelte 4
- **主题切换组件**: 使用 `src/lib/components/ThemeSwitcher.svelte` 提供主题切换功能
  - 颜色主题选择器（下拉菜单）
  - Dark Mode 开关
  - 自动持久化到 localStorage
  - 防止主题闪烁（在 hydration 前设置）

### Tailwind CSS 4 配置
- **Vite 插件**: 在 `vite.config.js` 中使用 `@tailwindcss/vite` 插件
- **PostCSS**: 使用 `@tailwindcss/postcss` 而非传统 PostCSS 配置
- **配置文件**: `tailwind.config.js` 使用新格式
- **优先级**: Tailwind 插件必须在 Vite 配置中的 Svelte 插件之前

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
src/                         # SvelteKit 前端
├── routes/                 # 页面路由
│   ├── +layout.ts         # 根布局(SSR disabled)
│   ├── +page.svelte       # 主页
│   ├── library/           # 文献库页面
│   ├── reader/            # PDF 阅读器
│   └── settings/          # 设置页面
├── lib/                   # 前端工具库
│   ├── components/        # 自定义组件和 Skeleton 组件封装
│   ├── css/               # 全局样式文件
│   │   └── app.css       # Tailwind + Skeleton 样式导入
│   ├── stores/            # Svelte stores (状态管理)
│   │   └── theme.ts      # 主题切换 store
│   └── types.ts          # TypeScript 类型定义
├── app.html               # HTML 模板
vite.config.js             # Vite 配置
svelte.config.js           # Svelte 配置
tailwind.config.js         # Tailwind CSS 配置
postcss.config.js          # PostCSS 配置
package.json               # Node.js 依赖
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
│   ├── db/               # 数据库层
│   │   ├── mod.rs
│   │   ├── models.rs
│   │   ├── schema.rs
│   │   └── queries.rs
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
```

**覆盖范围**:
- 各个 Rust 后端模块
- 解析器、数据库、AI 算法
- 使用 `cargo test` 和 `assert!` 宏

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
- 使用 WebDriver 协议(Selenium)
- 模拟真实用户操作
- 跨平台测试(Windows/macOS/Linux)
- 测试场景: 导入文献 → 搜索 → 添加笔记 → 验证保存

### CI/CD 流程

**GitHub Actions 工作流**:
1. **编译检查**: Linux、Windows、macOS 三平台编译
2. **运行测试**: 单元测试 + 集成测试
3. **代码质量**: `clippy` + `rustfmt`
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
  - Svelte for VS Code
  - Tauri
  - rust-analyzer
  - Even Better TOML(Cargo.toml 支持)
  - Error Lens(内联错误显示)

- **调试工具**:
  - CrabNebula DevTools (Tauri 应用调试)
  - Chrome DevTools (前端调试)

## 参考资源

### 官方文档
- [Tauri 官方文档](https://tauri.app/)
- [SvelteKit 文档](https://kit.svelte.dev/)
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
4. 确保通过所有检查(`cargo test`, `cargo clippy`, `yarn check`)
5. 提交 Pull Request

**代码规范**:
- Rust: 遵循 `rustfmt` 格式化，通过 `clippy` 检查
- TypeScript: 遵循 ESLint 规则
- 提交前运行 `yarn check` 和 `cargo test`
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

**最后更新**: 2025-01-20
