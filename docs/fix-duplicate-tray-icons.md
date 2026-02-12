# 修复：启动时出现两个托盘图标

## 问题描述
应用启动时，系统托盘中出现了两个 xuan-brain 图标。

## 问题原因
托盘图标被创建了两次：
1. **tauri.conf.json** 中的 `trayIcon` 配置会自动创建一个托盘图标
2. **lib.rs** 代码中使用 `TrayIconBuilder` 手动创建了另一个托盘图标

## 解决方案
从 `tauri.conf.json` 中移除 `trayIcon` 配置，只在代码中创建托盘图标。

### 修改前 (tauri.conf.json)
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

### 修改后 (tauri.conf.json)
```json
{
  "app": {
    "windows": [...],
    "security": {...}
    // 移除了 trayIcon 配置
  }
}
```

## 为什么选择在代码中创建？

在代码中创建托盘图标有以下优势：
1. ✅ **更灵活** - 可以完全控制托盘行为
2. ✅ **动态菜单** - 可以根据应用状态动态修改菜单
3. ✅ **事件处理** - 可以自定义点击事件处理逻辑
4. ✅ **中文支持** - 菜单文本可以直接使用中文

## 测试修复

### 清理并重新编译
```cmd
cd d:\Rust\xuan-brain\src-tauri
cargo clean
cargo build
```

### 运行应用
```cmd
cd ..
yarn tauri dev
```

### 验证
启动应用后，检查系统托盘：
- ✅ 应该只有**一个** xuan-brain 托盘图标
- ✅ 左键单击可以切换窗口显示/隐藏
- ✅ 右键单击显示"退出"菜单

## 技术说明

### Tauri 2.x 的两种托盘创建方式

#### 方式 1: 配置文件（简单但不灵活）
```json
// tauri.conf.json
{
  "app": {
    "trayIcon": {
      "id": "main",
      "iconPath": "icons/icon.png"
    }
  }
}
```
- 优点：配置简单
- 缺点：无法自定义菜单和事件

#### 方式 2: 代码创建（推荐，灵活）
```rust
// lib.rs
let _tray = TrayIconBuilder::new()
    .icon(app.default_window_icon().unwrap().clone())
    .menu(&menu)
    .on_menu_event(|app, event| { /* 自定义处理 */ })
    .on_tray_icon_event(|tray, event| { /* 自定义处理 */ })
    .build(app)?;
```
- 优点：完全可控，支持动态菜单和自定义事件
- 缺点：需要编写代码

### 我们的选择
✅ 使用**方式 2**（代码创建），因为我们需要：
- 自定义的左键单击行为（切换窗口）
- 中文菜单项
- 自定义的窗口关闭行为

## 相关文件

### 已修改
- ✅ `src-tauri/tauri.conf.json` - 移除了 `trayIcon` 配置

### 保持不变
- ✅ `src-tauri/src/lib.rs` - 托盘创建代码保持不变

## 常见问题

### Q: 为什么不能两种方式都用？
A: 因为它们会各自创建一个独立的托盘图标，导致出现重复图标。

### Q: 如果我想要配置文件的简单性怎么办？
A: 如果不需要自定义菜单和事件处理，可以只使用配置文件方式，并删除 `lib.rs` 中的 `TrayIconBuilder` 代码。

### Q: 托盘图标的功能会受影响吗？
A: 不会，所有功能保持不变：
- 左键单击切换窗口
- 右键单击显示"退出"菜单
- 关闭窗口时隐藏到托盘

---

**修复日期**: 2026年2月12日  
**状态**: ✅ 已修复  
**需要重新编译**: 是
