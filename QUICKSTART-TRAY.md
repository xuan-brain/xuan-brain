# 🚀 快速开始 - 系统托盘功能测试

## 问题已修复 ✅

**原错误**: `no matching package named tauri-plugin-tray found`  
**解决方案**: 使用 Tauri 2.x 内置的托盘功能，不需要独立插件

## 立即测试 ⚡

### 步骤 1: 清理旧的构建

```cmd
cd d:\Rust\xuan-brain\src-tauri
cargo clean
```

### 步骤 2: 构建项目

```cmd
cargo build
```

如果构建成功，你会看到类似这样的输出：

```
Compiling xuan-brain v0.1.0 (d:\Rust\xuan-brain\src-tauri)
Finished dev [unoptimized + debuginfo] target(s) in X.XXs
```

### 步骤 3: 运行应用

```cmd
cd ..
yarn tauri dev
```

### 步骤 4: 测试功能

1. **查看托盘**: 应用启动后，检查系统托盘（Windows 任务栏右下角），应该能看到 xuan-brain 图标
2. **关闭窗口**: 点击窗口的 X 按钮，窗口应该消失但托盘图标仍在
3. **显示窗口**: 左键单击托盘图标，窗口重新显示
4. **托盘菜单**: 右键单击托盘图标，应该看到"退出"菜单
5. **退出应用**: 通过托盘菜单点击"退出"，应用完全关闭

## 如果遇到编译错误 ⚠️

### 错误: 找不到 tauri 的某个 feature

```
error: failed to select a version for `tauri`
```

**解决方案**: 确保 Cargo.toml 中的 tauri 配置正确：

```toml
tauri = { version = "2", features = ["tray-icon", "image-png", "image-ico"] }
```

### 错误: 图标加载失败

```
Error: Failed to load icon
```

**解决方案**: 确保图标文件存在：

```cmd
dir src-tauri\icons\icon.png
```

### 错误: 菜单 API 不存在

```
error: no variant named `MenuItem` found
```

**解决方案**: 检查导入路径，应该是：

```rust
use tauri::menu::{Menu, MenuItem};
```

## 成功标志 ✅

如果你看到以下内容，说明功能正常：

- ✅ 应用编译成功，无错误
- ✅ 应用启动时托盘出现图标
- ✅ 关闭窗口后托盘图标仍在
- ✅ 可以通过托盘图标重新打开窗口（左键单击）
- ✅ 托盘菜单显示正常（右键单击，只显示"退出"选项）
- ✅ "退出"功能正常工作

## 需要帮助？ 🆘

查看详细文档：

- `docs/system-tray-implementation.md` - 完整实现说明
- `docs/system-tray-fix.md` - 问题排查指南
- `docs/system-tray-testing.md` - 详细测试步骤

## 配置文件清单 📋

确认以下文件已正确配置：

### ✅ Cargo.toml

```toml
tauri = { version = "2", features = ["tray-icon", "image-png", "image-ico"] }
```

⚠️ **不应该有** `tauri-plugin-tray`

### ✅ lib.rs

导入应该是：

```rust
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
```

### ✅ tauri.conf.json

**不需要**添加 `trayIcon` 配置。

⚠️ **注意**：如果添加了 `trayIcon` 配置，会导致出现两个托盘图标！

托盘图标完全在 Rust 代码中创建和管理。

---

**准备好了吗？现在就运行 `cargo build` 开始测试！** 🎉
