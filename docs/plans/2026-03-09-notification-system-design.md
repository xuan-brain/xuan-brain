# 通知系统设计文档

**日期**: 2026-03-09
**状态**: 设计已完成，等待实施

## 背景

当前 xuan-brain 应用缺乏统一的通知机制。错误处理分散在各个组件中，使用 `v-alert` 内嵌显示，且没有通知历史记录。需要一个系统化的通知解决方案。

## 需求总结

1. **正常通知**：用户操作时的成功/信息类通知，在状态栏显示（支持临时 toast 和固定状态）
2. **错误通知**：使用弹出对话框显示（严重错误用全局对话框，一般错误用对话框内嵌）
3. **系统通知**：应用隐藏或失去焦点时，调用 Tauri 系统通知
4. **通知历史**：保存所有通知，可在设置中查看和管理
5. **四级分类**：成功（绿色）、信息（蓝色）、警告（黄色）、错误（红色）
6. **前后端统一**：前端和后端都能发送通知，使用同一套处理逻辑

## 架构设计

### 数据流

```
┌─────────────────────────────────────────────────────────────┐
│                    统一通知中心                              │
│                  (NotificationService)                      │
└─────────────────────────────────────────────────────────────┘
         ↑                              ↑
         │                              │
    前端直接调用                    后端事件触发
         │                              │
    ┌────┴────┐                  ┌──────┴──────┐
    │ Vue 组件 │                  │ Rust Backend│
    │ useNotification()          │ Tauri Events│
    └─────────┘                  └─────────────┘
              ↓                              ↓
         ┌──────────────────────────────────┴─────────┐
         │       useNotificationStore (状态管理)        │
         └─────────────────────────────────────────────┘
                          ↓
         ┌────────────────┼────────────────┬────────────┐
         ↓                ↓                ↓            ↓
    StatusBar Toast    全局对话框      系统通知      历史记录
```

### 文件结构

```
src/
├── stores/
│   └── useNotificationStore.ts      # 通知状态管理
├── components/
│   ├── notification/
│   │   ├── NotificationToast.vue    # 状态栏临时通知组件
│   │   ├── GlobalErrorDialog.vue    # 全局错误对话框
│   │   └── NotificationHistory.vue  # 通知历史面板
│   └── layout/
│       └── StatusBar.vue            # 修改：集成通知显示区域
├── lib/
│   └── notification.ts              # 通知服务
├── composables/
│   └── useNotification.ts           # 便捷调用函数
└── lib/i18n/
    ├── zh.ts                        # 添加通知相关中文翻译
    └── en.ts                        # 添加通知相关英文翻译

src-tauri/src/
└── notification/                    # 新增模块
    ├── mod.rs
    ├── types.rs                     # 通知类型定义
    └── emitter.rs                   # 事件发送封装
```

## 核心类型定义

```typescript
enum NotificationType {
  Success = 'success', // 绿色
  Info = 'info', // 蓝色
  Warning = 'warning', // 黄色
  Error = 'error', // 红色
}

enum NotificationDisplay {
  Toast = 'toast', // 临时消息，自动消失
  StatusBar = 'status', // 固定状态文本
  Dialog = 'dialog', // 全局错误对话框
  System = 'system', // 系统通知
}

interface Notification {
  id: string;
  type: NotificationType;
  title: string;
  message: string;
  display: NotificationDisplay;
  duration?: number; // Toast 持续时间（毫秒）
  persistent?: boolean; // 是否持久保存到历史
  timestamp: number;
  read?: boolean; // 是否已读
  details?: string; // 错误详情（可选）
}
```

## 组件设计

### NotificationToast.vue

- 在 StatusBar 内显示临时通知
- 支持 4 种类型，带颜色区分
- 自动消失（默认 3 秒）
- 支持多通知堆叠

### GlobalErrorDialog.vue

- 全局模态对话框，显示严重错误
- 可折叠的错误详情
- 操作按钮：确定、复制错误、关闭应用

### StatusBar.vue（修改）

- 左侧新增固定状态文本区域（如"正在导入... 3/10"）
- 临时通知浮在状态栏上方

### NotificationHistory.vue

- 设置页面的通知历史面板
- 支持按类型/状态/日期筛选
- 支持清空历史、标记已读

## 调用方式

### 前端调用

```typescript
// 使用 composable（推荐）
import { useNotification } from '@/composables/useNotification';

const { showSuccess, showError, showWarning, showInfo, setStatus } = useNotification();

// 简单调用
showSuccess('操作成功');

// 带选项
showError('操作失败', {
  display: 'dialog',
  persistent: true,
  details: errorStack,
});

// 进度状态
setStatus('正在导入... 3/10');
// 完成后
clearStatus();
```

### 后端调用（Rust）

```rust
use crate::notification::NotificationEmitter;

// 发送通知
NotificationEmitter::success(&app_handle)
    .title("导入成功")
    .message("文献导入完成")
    .display(NotificationDisplay::Toast)
    .persistent(true)
    .send()?;
```

## Tauri 事件定义

**事件名称**：

- `notification:success`
- `notification:info`
- `notification:warning`
- `notification:error`

**事件 payload**：

```json
{
  "title": "操作成功",
  "message": "文献导入完成",
  "display": "toast",
  "persistent": true,
  "duration": 3000
}
```

## 国际化

新增翻译键（中英文）：

```typescript
notification: {
  success: '成功',
  info: '信息',
  warning: '警告',
  error: '错误',
  operationSuccess: '操作成功',
  operationFailed: '操作失败',
  details: '详情',
  copy: '复制',
  close: '关闭',
  clearHistory: '清空历史',
  notificationHistory: '通知历史',
  // ...
}
```

## 实施计划

### 阶段 1：核心基础设施

1. 创建 `useNotificationStore.ts`
2. 创建 `notification.ts` 服务层
3. 创建 `useNotification.ts` composable
4. 定义 TypeScript 类型

### 阶段 2：UI 组件

5. 创建 `NotificationToast.vue`
6. 创建 `GlobalErrorDialog.vue`
7. 修改 `StatusBar.vue`
8. 创建 `NotificationHistory.vue`

### 阶段 3：Rust 后端

9. 创建 `src-tauri/src/notification/` 模块
10. 实现事件发送逻辑
11. 更新现有 commands 使用新通知系统

### 阶段 4：集成与优化

12. 在 `App.vue` 中注册全局组件和事件监听
13. 更新现有组件使用新通知系统
14. 添加国际化文本
15. 测试与调优

## 测试场景

1. 用户操作成功 → 状态栏显示绿色 toast
2. 网络请求失败 → 显示全局错误对话框
3. 长时间任务：显示状态栏进度 → 完成后显示 toast
4. 应用最小化时收到通知 → 系统通知弹出
5. 打开设置 → 查看通知历史
6. 筛选/删除/清空通知历史

## 关键文件清单

**新增**：

- `src/stores/useNotificationStore.ts`
- `src/lib/notification.ts`
- `src/composables/useNotification.ts`
- `src/components/notification/NotificationToast.vue`
- `src/components/notification/GlobalErrorDialog.vue`
- `src/components/notification/NotificationHistory.vue`
- `src-tauri/src/notification/mod.rs`
- `src-tauri/src/notification/types.rs`
- `src-tauri/src/notification/emitter.rs`

**修改**：

- `src/App.vue`
- `src/components/layout/StatusBar.vue`
- `src/lib/i18n/zh.ts`
- `src/lib/i18n/en.ts`
- `src-tauri/src/lib.rs`
- `src-tauri/src/command/*.rs`（逐步迁移）

## 依赖确认

- ✅ `@tauri-apps/api`: ^2.10.3（已安装）
- ✅ `@tauri-apps/plugin-notification`: ^2.3.3（已安装）
- ✅ Pinia（已使用）
- ✅ Vue 3 Composition API（已使用）

无需新增依赖。
