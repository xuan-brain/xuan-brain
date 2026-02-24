# 轩脑 (xuan-brain)

> AI 驱动的科研文献管理工具

<div align="center">

**让文献管理更智能,让科研工作更高效**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/your-org/xuan-brain)
[![Tauri](https://img.shields.io/badge/Tauri-2.x-orange.svg)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3-42b883.svg)](https://vuejs.org/)

</div>

## ✨ 特性

### 📚 智能文献管理

- **元数据导入**: 通过 DOI 和 arXiv ID 自动导入文献
- **自动解析**: 提取标题、作者、摘要、出版信息等元数据
- **文献管理**: 高效管理文献库，支持添加、编辑、删除
- **作者关联**: 自动关联作者信息，支持多作者论文

### 🌐 网页剪藏

- **一键剪藏**: 快速保存网页内容到本地知识库
- **Markdown 支持**: 自动将网页内容转换为 Markdown 格式
- **图片下载**: 自动下载并本地化网页中的图片
- **标签管理**: 为剪藏内容添加标签，方便分类检索
- **评论功能**: 为剪藏内容添加评论和笔记
- **阅读状态**: 跟踪剪藏内容的阅读状态

### 🗂️ 高效组织与检索

- **分类管理**: 树形分类结构，支持无限层级嵌套
- **标签系统**: 多维度标签分类，支持自定义颜色
- **拖拽操作**: 直观的拖拽操作，方便组织文献和分类
- **三栏布局**: 左侧导航、中间列表、右侧详情，高效浏览

### 🎨 现代化界面

- **暗色/亮色主题**: 自由切换主题，支持动态强调色
- **响应式布局**: 自适应窗口大小，可拖拽调整各栏宽度
- **国际化支持**: 内置中英文界面，一键切换
- **流畅体验**: 基于 Vue 3 与 Tauri 2，性能卓越

### 🔒 安全与隐私

- **本地优先**: 所有数据存储在本地 SurrealDB 数据库
- **开源透明**: 代码开源，可自行审计
- **跨平台**: 支持 Windows、macOS、Linux

### 🛠 开发中功能

- **PDF 阅读器**: 即将支持内置 PDF 阅读和标注
- **全文搜索**: 基于关键词的快速文献检索
- **本地文件导入**: 支持导入 PDF、DOCX 等本地文件
- **引用管理**: 自动生成多种引用格式

## 🏗️ 技术架构

### 技术栈

- **前端**: [Vue 3](https://vuejs.org/) + [TypeScript](https://www.typescriptlang.org/)
  - UI 框架: [Vuetify 3](https://vuetifyjs.com/)
  - 状态管理: [Pinia](https://pinia.vuejs.org/)
  - 路由: [Vue Router 4](https://router.vuejs.org/)
  - 树形组件: Vuetify v-treeview
  - 样式: [Tailwind CSS](https://tailwindcss.com/)（如需）
- **后端**: [Rust](https://www.rust-lang.org/) + [Tauri 2](https://tauri.app/)
  - ORM: [SeaORM 2.0](https://www.sea-ql.org/SeaORM/)
  - 日志: [tracing](https://docs.rs/tracing/)
  - 异步运行时: [Tokio](https://tokio.rs/)
- **数据库**: [SurrealDB](https://surrealdb.com/)
  - 嵌入式模式，支持图关系查询

### 为什么选择 Rust + Tauri?

- **轻量高效**: 相比 Electron 体积小 80%,内存占用更低
- **性能卓越**: Rust 的零成本抽象和内存安全
- **跨平台**: 一套代码,支持 Windows、macOS、Linux
- **安全可靠**: Rust 的类型安全和内存安全保证

## 🚀 快速开始

### 环境要求

- Node.js 24+
- yarn 1.22+
- Rust 1.70+
- 系统 WebView2 (Windows) / WebKit (macOS/Linux)

### 安装

```bash
# 克隆仓库
git clone https://github.com/your-org/xuan-brain.git
cd xuan-brain

# 安装前端依赖
yarn install

# 安装 Rust 工具链（如果尚未安装）
# 访问 https://www.rust-lang.org/tools/install
```

### 开发

```bash
# 启动开发模式
yarn tauri dev
```

### 构建

```bash
# 构建生产版本
yarn tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`

**跨平台支持**:

- Windows: `.msi` 安装包
- macOS: `.dmg` 磁盘映像 (支持 Intel 和 Apple Silicon)
- Linux: `.AppImage` / `.deb` / `.rpm`

### 自动化构建

项目集成了 GitHub Actions 自动化构建流程：

#### Release Build（正式版本）
当创建新的 Release 时，会自动构建并上传所有平台的安装包。

#### Nightly Build（每日构建）
每天自动构建最新开发版本，并自动创建/更新 "nightly" 预发布版本。可直接在 [Releases 页面](https://github.com/xuan-brain/xuan-brain/releases) 下载标记为 "Pre-release" 的 nightly 版本。

详细信息请参阅 [发布工作流文档](docs/release-workflow.md)。

## 📖 使用文档

详细的使用文档和开发指南,请参阅:

- [项目介绍](docs/introduction.md) - 完整的架构设计文档
- [用户手册](docs/user-guide.md) - 安装、配置和使用教程（待创建）
- [开发者文档](docs/developer-guide.md) - API 文档、开发指南（待创建）
- [插件开发](docs/plugin-development.md) - 插件开发教程（待创建）

## 🤝 贡献

欢迎社区贡献!请查看 [贡献指南](CONTRIBUTING.md)。

### 开发流程

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

### 代码规范

- **Rust**:
  ```bash
  cd src-tauri
  cargo fmt          # 格式化代码
  cargo clippy       # 代码检查
  cargo test         # 运行测试
  cargo audit        # 安全扫描
  ```
- **TypeScript**:
  ```bash
  yarn tsc --noEmit  # 类型检查
  yarn lint          # ESLint 检查
  yarn format        # Prettier 格式化
  ```
- **日志规范**: 所有日志消息使用英文

## 🛣️ 路线图

### v0.1.0 (当前版本)

- [x] 基础文献管理（通过 DOI 和 arXiv ID 导入）
- [x] 分类树管理（树形结构、拖拽操作）
- [x] 标签系统（自定义颜色、多维度分类）
- [x] 三栏布局（可调整宽度）
- [x] 主题切换（暗色/亮色模式）
- [x] 国际化支持（中英文）
- [x] 本地数据存储（SurrealDB）
- [x] 网页剪藏功能（Markdown 转换、图片下载、评论）

### v0.2.0 (计划中)

- [ ] PDF 阅读器实现
- [ ] 本地文件导入（PDF、DOCX 等）
- [ ] 全文搜索功能
- [ ] 作者管理
- [ ] 关键词管理
- [ ] 附件管理
- [ ] 设置页面

### v0.3.0 (未来版本)

- [ ] AI 智能推荐和分类
- [ ] 关键词自动提取
- [ ] 笔记和标注功能
- [ ] 引用格式生成
- [ ] 云同步服务
- [ ] WebDAV 支持
- [ ] 插件系统
- [ ] RESTful API

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## 🙏 致谢

- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [Vue](https://vuejs.org/) - Vue 框架
- [Vuetify](https://vuetifyjs.com/) - Material Design 3 UI 框架
- [Pinia](https://pinia.vuejs.org/) - 轻量状态管理
- [Vue Router](https://router.vuejs.org/) - 官方路由器
- [Tailwind CSS](https://tailwindcss.com/) - 实用类样式库
- [Rust](https://www.rust-lang.org/) - 系统级语言
- [SurrealDB](https://surrealdb.com/) - 嵌入式图数据库

---

<div align="center">
 Made with ❤️ by the xuan-brain team
</div>
