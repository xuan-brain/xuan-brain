@echo off
chcp 65001 >nul
echo 正在提交系统托盘功能...
echo.

REM 添加所有修改的文件
git add src-tauri/Cargo.toml
git add src-tauri/src/lib.rs
git add src-tauri/tauri.conf.json

REM 添加所有文档文件
git add docs/system-tray-feature.md
git add docs/system-tray-testing.md
git add docs/system-tray-fix.md
git add docs/system-tray-summary.md
git add docs/system-tray-implementation.md
git add docs/system-tray-menu-update.md
git add docs/fix-duplicate-tray-icons.md
git add docs/TRAY-FIX-SUMMARY.md
git add QUICKSTART-TRAY.md

REM 提交
git commit -m "feat: 添加系统托盘功能

- 添加系统托盘图标，支持最小化到托盘
- 左键单击托盘图标切换窗口显示/隐藏
- 右键菜单显示"退出"选项
- 窗口关闭时隐藏到托盘而不退出应用
- 修复重复托盘图标问题（移除配置文件中的trayIcon）
- 使用 Tauri 2.x 内置的 tray-icon feature
- 完善相关文档和测试指南

技术实现：
- 在 Cargo.toml 中启用 tray-icon, image-png, image-ico features
- 在 lib.rs 中使用 TrayIconBuilder 创建托盘
- 添加窗口关闭事件处理，防止应用退出
- 支持中文托盘菜单"

echo.
echo ✅ 提交完成！
echo.
pause
