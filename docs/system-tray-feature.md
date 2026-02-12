# 系统托盘功能实现文档

## 概述
本文档描述了为 xuan-brain 应用添加的系统托盘功能的实现细节。

## 功能特性

### 1. 系统托盘图标
- 应用程序在启动时会在系统托盘显示图标
- 使用应用默认图标作为托盘图标

### 2. 窗口关闭行为
- 点击窗口关闭按钮时，窗口不会真正退出应用，而是隐藏到系统托盘
- 只有通过托盘菜单选择"退出"才会真正关闭应用

### 3. 托盘交互
- **单击左键**: 切换窗口显示/隐藏状态
  - 如果窗口可见，点击后隐藏窗口
  - 如果窗口隐藏，点击后显示窗口并获得焦点
  
- **右键菜单**（Windows）/ **左键菜单**（macOS）:
  - **显示窗口**: 显示主窗口并获得焦点
  - **退出**: 完全退出应用程序

## 技术实现

### 依赖项
在 `src-tauri/Cargo.toml` 中，系统托盘功能通过 `tauri` 核心包的 `tray-icon` feature 启用：
```toml
tauri = { version = "2", features = ["tray-icon"] }
```

注意：在 Tauri 2.x 中，托盘功能是内置的，不需要单独的插件包。

### 配置文件
在 `src-tauri/tauri.conf.json` 中添加托盘配置：
```json
"trayIcon": {
  "id": "main",
  "iconPath": "icons/icon.png",
  "iconAsTemplate": false,
  "menuOnLeftClick": false,
  "title": "xuan-brain",
  "tooltip": "xuan-brain"
}
```

### 代码实现

#### 1. 导入必要的类型
```rust
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
```

#### 2. 创建托盘菜单
在 `setup` 闭包中创建托盘菜单项：
```rust
let show_i = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
let menu = Menu::with_items(app, &[&show_i, &quit_i])?;
```

#### 3. 构建托盘图标
使用 `TrayIconBuilder` 创建托盘图标并设置事件处理：
```rust
let _tray = TrayIconBuilder::new()
    .icon(app.default_window_icon().unwrap().clone())
    .menu(&menu)
    .menu_on_left_click(false)
    .on_menu_event(|app, event| match event.id.as_ref() {
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    })
    .on_tray_icon_event(|tray, event| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            let app = tray.app_handle();
            if let Some(window) = app.get_webview_window("main") {
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }
    })
    .build(app)?;
```

#### 4. 处理窗口关闭事件
使用 `on_window_event` 拦截窗口关闭请求：
```rust
.on_window_event(|window, event| {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        // Prevent window from closing and hide it instead
        window.hide().unwrap();
        api.prevent_close();
    }
})
```

## 用户体验

### Windows
- 托盘图标显示在任务栏右下角的系统托盘区域
- 左键单击图标可快速切换窗口显示状态
- 右键单击显示菜单，可选择"显示窗口"或"退出"

### macOS
- 托盘图标显示在菜单栏右侧
- 点击图标显示菜单，可选择"显示窗口"或"退出"
- 单击行为与 macOS 系统习惯一致

### Linux
- 托盘图标显示在系统托盘区域（具体位置取决于桌面环境）
- 交互行为与 Windows 类似

## 注意事项

1. **图标资源**: 托盘图标使用应用的默认窗口图标，确保 `icons/icon.png` 文件存在
2. **平台差异**: 不同操作系统的托盘行为略有不同，已按照 Tauri 2.x 的推荐方式实现
3. **窗口状态**: 应用使用 `tauri-plugin-window-state` 插件，窗口的位置和大小状态会被保存
4. **性能**: 窗口隐藏时应用继续在后台运行，不会释放资源

## 未来改进建议

1. 添加更多托盘菜单选项（如快速操作、通知设置等）
2. 托盘图标动态更新（显示通知数量等）
3. 添加托盘消息提示
4. 提供设置选项让用户选择关闭窗口时的行为（最小化到托盘或直接退出）
