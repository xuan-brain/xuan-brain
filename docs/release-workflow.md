# GitHub Actions Release Workflow

本文档介绍了 xuan-brain 项目的自动化发布流程。

## 概述

项目使用 GitHub Actions 实现了自动化的跨平台构建和发布流程。当创建新的 Release 时，工作流会自动构建并上传适用于 Windows、Linux 和 macOS 的安装包。

## 工作流文件

位置: `.github/workflows/release.yml`

## 触发条件

工作流在以下情况下自动触发:
- 当推送以 `v` 开头的 tag 时（例如：`v0.1.0`, `v1.2.3`）
- Workflow 会自动创建 GitHub Release 并上传构建产物

## 构建平台

工作流会为以下平台构建安装包:

### Linux
- **架构**: x86_64
- **运行环境**: Ubuntu 22.04
- **输出格式**: `.AppImage`, `.deb`

### macOS
- **Intel 架构**: x86_64
- **Apple Silicon**: aarch64 (M1/M2/M3)
- **运行环境**: macOS latest
- **输出格式**: `.dmg`, `.app` bundle

### Windows
- **架构**: x86_64
- **运行环境**: Windows latest
- **输出格式**: `.msi`, `.exe`

## 构建步骤

每个平台的构建流程包括:

1. **检出代码**: 克隆仓库代码
2. **设置 Node.js**: 安装 Node.js 18
3. **安装 Yarn**: 全局安装 Yarn 包管理器
4. **设置 Rust**: 安装 Rust 工具链和目标平台
5. **Rust 缓存**: 缓存 Rust 依赖以加速构建
6. **安装系统依赖** (仅 Linux):
   - libwebkit2gtk-4.1-dev
   - libappindicator3-dev
   - librsvg2-dev
   - patchelf
   - libsoup-3.0-dev
   - libjavascriptcoregtk-4.1-dev
7. **安装前端依赖**: 运行 `yarn install --frozen-lockfile`
8. **构建 Tauri 应用**: 使用 `tauri-apps/tauri-action` 构建应用
9. **自动创建/更新 Release**: 如果 Release 不存在则创建，否则更新
10. **上传制品**: 自动将所有构建产物上传到 GitHub Release 的附件中

### 自动上传功能

Workflow 使用 `tauri-apps/tauri-action@v0` 的内置上传功能：

- **自动检测构建产物**: Action 会自动找到所有构建生成的安装包
- **批量上传**: 将所有平台的安装包一次性上传到 Release
- **覆盖已有文件**: 如果 Release 中已存在同名文件，会自动覆盖
- **附件命名**: 保持 Tauri 默认的文件命名格式
- **完整性验证**: 所有上传的文件都包含 SHA256 校验和

## 如何使用

### 创建新的 Release

有两种方式创建 release：

#### 方式一：通过 Git Tag 触发（推荐）

```bash
# 1. 确保所有更改已提交
git add .
git commit -m "Prepare for release v0.1.0"

# 2. 创建并推送 tag
git tag v0.1.0
git push origin v0.1.0

# 3. GitHub Actions 会自动：
#    - 构建所有平台的安装包
#    - 创建 GitHub Release
#    - 上传所有构建产物到 Release
```

#### 方式二：通过 GitHub 界面

1. 在 GitHub 仓库页面，点击 "Releases"
2. 点击 "Create a new release"
3. 填写以下信息:
   - **Choose a tag**: 创建新 tag，如 `v0.1.0`
   - **Release title**: 如 "xuan-brain v0.1.0"
   - **Description**: 发布说明（更新内容、新功能等）
4. 点击 "Publish release"
5. 创建 release 会自动创建 tag，触发构建流程

### 监控构建进度

1. 点击 "Actions" 标签
2. 找到对应的 "Release Build" 工作流
3. 查看每个平台的构建状态和日志

### 构建时间

预计构建时间:
- Linux: 约 10-15 分钟
- macOS (两个架构): 各约 15-20 分钟
- Windows: 约 15-20 分钟

总计: 约 1 小时 (所有平台并行构建)

### 下载安装包

构建完成后，安装包会自动上传到对应的 Release 页面：

1. 访问仓库的 Releases 页面
2. 找到对应版本的 release（例如 `v0.1.0`）
3. 在 "Assets" 部分可以看到所有平台的安装包
4. 点击下载对应平台的安装包

所有构建产物都会自动作为 release 附件上传。

## 构建产物

### Linux
- `xuan-brain_0.1.0_amd64.AppImage` - AppImage 格式
- `xuan-brain_0.1.0_amd64.deb` - Debian 包

### macOS
- `xuan-brain_0.1.0_x64.dmg` - Intel Mac 磁盘映像
- `xuan-brain_0.1.0_aarch64.dmg` - Apple Silicon 磁盘映像

### Windows
- `xuan-brain_0.1.0_x64_en-US.msi` - MSI 安装包
- `xuan-brain_0.1.0_x64-setup.exe` - 可执行安装程序

## 故障排查

### 构建失败

如果构建失败，请检查:

1. **Cargo.toml 版本**: 确保与 Release tag 版本一致
2. **package.json 版本**: 确保与 Release tag 版本一致
3. **tauri.conf.json 版本**: 确保与 Release tag 版本一致
4. **依赖项**: 确保所有依赖项都可以正常安装
5. **构建日志**: 查看 Actions 日志了解具体错误信息

### 常见问题

**Q: 为什么需要同时构建 macOS 的两个架构?**

A: Apple Silicon (M1/M2/M3) 芯片使用 ARM 架构，而旧的 Intel Mac 使用 x86_64 架构。为了支持所有 Mac 用户，我们需要构建两个版本。

**Q: 可以手动触发构建吗?**

A: 目前工作流仅在发布 Release 时触发。如需手动构建，可以在本地运行 `yarn tauri build`。

**Q: 如何更新工作流配置?**

A: 编辑 `.github/workflows/release.yml` 文件并提交更改。更改会在下次 Release 时生效。

## 技术细节

### 使用的 GitHub Actions

- `actions/checkout@v4`: 检出代码
- `actions/setup-node@v4`: 设置 Node.js 环境
- `dtolnay/rust-toolchain@stable`: 设置 Rust 工具链
- `swatinem/rust-cache@v2`: 缓存 Rust 依赖
- `tauri-apps/tauri-action@v0`: Tauri 官方构建和发布 Action

### 环境变量

- `GITHUB_TOKEN`: 自动提供，用于上传构建产物到 Release

### 缓存策略

- Rust 依赖缓存: 加速后续构建
- Node.js 依赖: 通过 `--frozen-lockfile` 确保依赖一致性

## 版本管理

在发布新版本前，请确保更新以下文件中的版本号:

1. `src-tauri/Cargo.toml` - `version = "x.x.x"`
2. `package.json` - `"version": "x.x.x"`
3. `src-tauri/tauri.conf.json` - `"version": "x.x.x"`

推荐使用版本管理工具自动更新这些文件。

## 参考资料

- [Tauri Actions 文档](https://github.com/tauri-apps/tauri-action)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Tauri 构建指南](https://tauri.app/v1/guides/building/)
