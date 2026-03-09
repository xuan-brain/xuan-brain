# 系统托盘功能 - 修正后的最终配置

## ✅ 问题已解决

原问题：`no matching package named tauri-plugin-tray found`

**根本原因**: Tauri 2.x 的系统托盘功能是内置在核心包中的，不是独立插件。

## 📝 最终配置

### Cargo.toml

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon", "image-png", "image-ico"] }
```

**说明**:

- `tray-icon`: 启用系统托盘功能
- `image-png`: PNG 图标支持（跨平台）
- `image-ico`: ICO 图标支持（Windows）

### lib.rs 导入

```rust
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
```

### tauri.conf.json

```json
{
  "app": {
    "trayIcon": {
      "id": "main",
      "iconPath": "icons/icon.png",
      "iconAsTemplate": false,
      "menuOnLeftClick": false,
      "title": "xuan-brain",
      "tooltip": "xuan-brain"
    }
  }
}
```

## 🚀 立即测试

清理并重新构建：

```cmd
cd src-tauri
cargo clean
cargo build
```

如果构建成功，运行应用：

```cmd
cd ..
yarn tauri dev
```

## ✨ 功能说明

1. **窗口关闭** → 隐藏到托盘（不退出）
2. **左键单击托盘** → 切换窗口显示/隐藏
3. **右键菜单** → "显示窗口" 和 "退出" 选项
4. **完全退出** → 只能通过托盘菜单的"退出"选项

## 📚 相关文档

- `docs/system-tray-feature.md` - 功能详细说明
- `docs/system-tray-testing.md` - 测试指南
- `docs/system-tray-fix.md` - 问题排查指南

## 🔍 验证配置

运行此命令检查依赖树：

```cmd
cd src-tauri
cargo tree | findstr tauri
```

你应该看到 `tauri v2.x.x` 和它的 features，但**不会**有 `tauri-plugin-tray`。
