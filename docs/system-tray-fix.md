# 系统托盘功能修复说明

## 问题描述
在首次实现时，错误地使用了 `tauri-plugin-tray` 包，但在 Tauri 2.x 中，系统托盘功能是内置在 `tauri` 核心包中的。

## 解决方案

### 1. Cargo.toml 配置
系统托盘功能通过 `tauri` 包的 `tray-icon` feature 启用：

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
```

**不需要** 添加 `tauri-plugin-tray` 依赖！

### 2. 代码导入
在 Rust 代码中，直接从 `tauri` 模块导入托盘相关类型：

```rust
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
```

### 3. 其他必需的 features
如果遇到其他编译错误，可能需要在 `tauri` 包中添加更多 features：

```toml
tauri = { version = "2", features = ["tray-icon", "image-png"] }
```

常用的 features 包括：
- `tray-icon`: 系统托盘功能
- `image-png`: PNG 图标支持
- `image-ico`: ICO 图标支持（Windows）
- `image-icns`: ICNS 图标支持（macOS）

## 编译和测试

清理并重新编译：
```bash
cd src-tauri
cargo clean
cargo build
```

如果成功编译，运行应用：
```bash
cd ..
yarn tauri dev
```

## 常见编译错误

### 错误 1: 找不到 `tauri::tray` 模块
**解决方案**: 确保 `tauri` 包配置了 `tray-icon` feature

### 错误 2: 图标相关错误
**解决方案**: 添加图像格式支持 features：
```toml
tauri = { version = "2", features = ["tray-icon", "image-png", "image-ico"] }
```

### 错误 3: 菜单相关错误
**解决方案**: Tauri 2.x 的菜单系统已内置，确保代码使用正确的导入路径：
```rust
use tauri::menu::{Menu, MenuItem};
```

## 验证配置

运行以下命令检查依赖：
```bash
cd src-tauri
cargo tree | grep tauri
```

应该看到 `tauri v2.x.x` 及其各种 features，但不应该有 `tauri-plugin-tray`。
