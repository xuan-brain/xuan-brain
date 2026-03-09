# 通知系统实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**目标:** 为 xuan-brain 应用创建统一的通知系统，支持状态栏 toast、固定状态、全局错误对话框、系统通知和通知历史记录。

**架构:** 使用 Pinia store 管理通知状态，通过 Tauri Events 连接前后端，NotificationService 路由通知到正确的显示方式。

**技术栈:** Vue 3 Composition API, Pinia, Vuetify 3, TypeScript, Tauri 2.x, Rust

---

## 前置准备

### Task 0: 创建组件目录结构

**Files:**

- Create: `src/components/notification/` (directory)
- Create: `src/composables/` (if not exists)
- Create: `src-tauri/src/notification/` (directory)

**Step 1: 创建前端通知组件目录**

```bash
mkdir -p src/components/notification
```

**Step 2: 创建 Rust 通知模块目录**

```bash
mkdir -p src-tauri/src/notification
```

**Step 3: 验证目录创建成功**

```bash
ls -la src/components/notification/
ls -la src-tauri/src/notification/
```

Expected: 两个目录都存在且为空

---

## 阶段 1：核心基础设施

### Task 1: 定义 TypeScript 类型

**Files:**

- Create: `src/types/notification.ts`

**Step 1: 创建通知类型定义文件**

```typescript
// src/types/notification.ts
export enum NotificationType {
  Success = 'success',
  Info = 'info',
  Warning = 'warning',
  Error = 'error',
}

export enum NotificationDisplay {
  Toast = 'toast',
  StatusBar = 'status',
  Dialog = 'dialog',
  System = 'system',
}

export interface Notification {
  id: string;
  type: NotificationType;
  title: string;
  message: string;
  display: NotificationDisplay;
  duration?: number;
  persistent?: boolean;
  timestamp: number;
  read?: boolean;
  details?: string;
}

export interface NotificationOptions {
  display?: NotificationDisplay;
  duration?: number;
  persistent?: boolean;
  details?: string;
}
```

**Step 2: 运行 TypeScript 类型检查**

```bash
yarn tsc --noEmit
```

Expected: 无类型错误

**Step 3: Commit**

```bash
git add src/types/notification.ts
git commit -m "feat(notification): add TypeScript type definitions"
```

---

### Task 2: 创建通知 Store

**Files:**

- Create: `src/stores/useNotificationStore.ts`

**Step 1: 创建通知 store 文件**

```typescript
// src/stores/useNotificationStore.ts
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type {
  Notification,
  NotificationOptions,
  NotificationType,
  NotificationDisplay,
} from '@/types/notification';

export const useNotificationStore = defineStore(
  'notification',
  () => {
    // State
    const toasts = ref<Notification[]>([]);
    const statusText = ref<string | null>(null);
    const history = ref<Notification[]>([]);
    const showDialog = ref(false);
    const dialogContent = ref<Notification | null>(null);

    // Getters
    const unreadCount = computed(() => history.value.filter((n) => !n.read).length);

    // Helper function to create notification
    function createNotification(
      type: NotificationType,
      title: string,
      message: string,
      options: NotificationOptions = {}
    ): Notification {
      return {
        id: `notif-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        type,
        title,
        message,
        display: options.display || 'toast',
        duration: options.duration ?? 3000,
        persistent: options.persistent ?? true,
        timestamp: Date.now(),
        read: false,
        details: options.details,
      };
    }

    // Add notification (main entry point)
    function add(notification: Partial<Notification>): void {
      const fullNotification: Notification = {
        id: `notif-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        type: notification.type || 'info',
        title: notification.title || '',
        message: notification.message || '',
        display: notification.display || 'toast',
        duration: notification.duration ?? 3000,
        persistent: notification.persistent ?? true,
        timestamp: Date.now(),
        read: false,
        details: notification.details,
      };

      // Save to history if persistent
      if (fullNotification.persistent) {
        history.value.unshift(fullNotification);
        // Limit history to 1000 items
        if (history.value.length > 1000) {
          history.value = history.value.slice(0, 1000);
        }
      }

      // Route based on display type
      switch (fullNotification.display) {
        case 'status':
          statusText.value = fullNotification.message;
          break;
        case 'dialog':
          dialogContent.value = fullNotification;
          showDialog.value = true;
          break;
        case 'toast':
        default:
          toasts.value.push(fullNotification);
          // Auto-remove after duration
          if (fullNotification.duration && fullNotification.duration > 0) {
            setTimeout(() => {
              removeToast(fullNotification.id);
            }, fullNotification.duration);
          }
          break;
      }
    }

    // Convenience methods
    function success(message: string, options: NotificationOptions = {}): void {
      add({ type: 'success', title: 'Success', message, ...options });
    }

    function info(message: string, options: NotificationOptions = {}): void {
      add({ type: 'info', title: 'Info', message, ...options });
    }

    function warning(message: string, options: NotificationOptions = {}): void {
      add({ type: 'warning', title: 'Warning', message, ...options });
    }

    function error(message: string, options: NotificationOptions = {}): void {
      add({ type: 'error', title: 'Error', message, ...options });
    }

    // Status bar methods
    function setStatus(text: string): void {
      statusText.value = text;
    }

    function clearStatus(): void {
      statusText.value = null;
    }

    // Toast management
    function removeToast(id: string): void {
      const index = toasts.value.findIndex((t) => t.id === id);
      if (index !== -1) {
        toasts.value.splice(index, 1);
      }
    }

    function clearToasts(): void {
      toasts.value = [];
    }

    // Dialog management
    function closeDialog(): void {
      showDialog.value = false;
      dialogContent.value = null;
    }

    // History management
    function markAsRead(id: string): void {
      const notification = history.value.find((n) => n.id === id);
      if (notification) {
        notification.read = true;
      }
    }

    function markAllAsRead(): void {
      history.value.forEach((n) => (n.read = true));
    }

    function deleteNotification(id: string): void {
      const index = history.value.findIndex((n) => n.id === id);
      if (index !== -1) {
        history.value.splice(index, 1);
      }
    }

    function clearHistory(): void {
      history.value = [];
    }

    return {
      // State
      toasts,
      statusText,
      history,
      showDialog,
      dialogContent,
      // Getters
      unreadCount,
      // Methods
      add,
      success,
      info,
      warning,
      error,
      setStatus,
      clearStatus,
      removeToast,
      clearToasts,
      closeDialog,
      markAsRead,
      markAllAsRead,
      deleteNotification,
      clearHistory,
    };
  },
  {
    persist: {
      key: 'xuan-brain-notification-storage',
      storage: localStorage,
      pick: ['history'],
    },
  }
);
```

**Step 2: 运行 TypeScript 类型检查**

```bash
yarn tsc --noEmit
```

Expected: 无类型错误

**Step 3: Commit**

```bash
git add src/stores/useNotificationStore.ts
git commit -m "feat(notification): add notification store with Pinia"
```

---

### Task 3: 创建通知服务层

**Files:**

- Create: `src/lib/notification.ts`

**Step 1: 创建通知服务文件**

```typescript
// src/lib/notification.ts
import { getCurrentWindow } from '@tauri-apps/api/window';
import {
  sendNotification,
  isPermissionGranted,
  requestPermission,
} from '@tauri-apps/plugin-notification';
import { useNotificationStore } from '@/stores/useNotificationStore';
import type { Notification, NotificationOptions } from '@/types/notification';

/**
 * 判断是否应该使用系统通知
 * 应用未聚焦或不可见时返回 true
 */
async function shouldUseSystemNotification(): Promise<boolean> {
  try {
    const currentWindow = getCurrentWindow();
    const [isFocused, isVisible] = await Promise.all([
      currentWindow.isFocused(),
      currentWindow.isVisible(),
    ]);
    return !isFocused || !isVisible;
  } catch (error) {
    console.error('Failed to check window state:', error);
    return false;
  }
}

/**
 * 显示系统通知
 */
async function showSystemNotification(notification: Notification): Promise<void> {
  try {
    let permission = await isPermissionGranted();
    if (!permission) {
      const requested = await requestPermission();
      permission = requested === 'granted';
    }

    if (permission) {
      await sendNotification({
        title: notification.title,
        body: notification.message,
      });
    }
  } catch (error) {
    console.error('Failed to show system notification:', error);
  }
}

/**
 * 发送通知的主入口
 */
export async function sendNotification(
  type: 'success' | 'info' | 'warning' | 'error',
  title: string,
  message: string,
  options: NotificationOptions = {}
): Promise<void> {
  const notificationStore = useNotificationStore();

  // 如果指定了 system 显示方式，检查应用状态
  if (options.display === 'system' || (await shouldUseSystemNotification())) {
    const notification: Notification = {
      id: `notif-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      type,
      title,
      message,
      display: 'system',
      persistent: options.persistent ?? true,
      timestamp: Date.now(),
      read: false,
      details: options.details,
    };

    // 保存到历史
    if (notification.persistent) {
      notificationStore.add(notification);
    }

    // 显示系统通知
    await showSystemNotification(notification);
  } else {
    // 否则使用普通通知
    notificationStore.add({ type, title, message, ...options });
  }
}

/**
 * 便捷方法
 */
export const notificationService = {
  success: (message: string, options?: NotificationOptions) =>
    sendNotification('success', 'Success', message, options),
  info: (message: string, options?: NotificationOptions) =>
    sendNotification('info', 'Info', message, options),
  warning: (message: string, options?: NotificationOptions) =>
    sendNotification('warning', 'Warning', message, options),
  error: (message: string, options?: NotificationOptions) =>
    sendNotification('error', 'Error', message, options),
};
```

**Step 2: 运行 TypeScript 类型检查**

```bash
yarn tsc --noEmit
```

Expected: 无类型错误

**Step 3: Commit**

```bash
git add src/lib/notification.ts
git commit -m "feat(notification): add notification service with system notification support"
```

---

### Task 4: 创建 useNotification Composable

**Files:**

- Create: `src/composables/useNotification.ts`

**Step 1: 创建 composable 文件**

```typescript
// src/composables/useNotification.ts
import { useNotificationStore } from '@/stores/useNotificationStore';
import type { NotificationOptions } from '@/types/notification';

/**
 * 通知 composable，提供便捷的通知调用方法
 */
export function useNotification() {
  const notificationStore = useNotificationStore();

  return {
    // Toast 通知
    success: (message: string, options?: NotificationOptions) =>
      notificationStore.success(message, { display: 'toast', ...options }),
    info: (message: string, options?: NotificationOptions) =>
      notificationStore.info(message, { display: 'toast', ...options }),
    warning: (message: string, options?: NotificationOptions) =>
      notificationStore.warning(message, { display: 'toast', ...options }),
    error: (message: string, options?: NotificationOptions) =>
      notificationStore.error(message, { display: 'toast', ...options }),

    // 对话框通知
    dialogSuccess: (message: string, options?: NotificationOptions) =>
      notificationStore.success(message, { display: 'dialog', ...options }),
    dialogError: (message: string, options?: NotificationOptions) =>
      notificationStore.error(message, { display: 'dialog', ...options }),

    // 状态栏
    setStatus: (text: string) => notificationStore.setStatus(text),
    clearStatus: () => notificationStore.clearStatus(),

    // Store 访问
    toasts: notificationStore.toasts,
    statusText: notificationStore.statusText,
    history: notificationStore.history,
    showDialog: notificationStore.showDialog,
    dialogContent: notificationStore.dialogContent,
    unreadCount: notificationStore.unreadCount,

    // 历史记录管理
    markAsRead: (id: string) => notificationStore.markAsRead(id),
    markAllAsRead: () => notificationStore.markAllAsRead(),
    deleteNotification: (id: string) => notificationStore.deleteNotification(id),
    clearHistory: () => notificationStore.clearHistory(),
    closeDialog: () => notificationStore.closeDialog(),
  };
}
```

**Step 2: 运行 TypeScript 类型检查**

```bash
yarn tsc --noEmit
```

Expected: 无类型错误

**Step 3: Commit**

```bash
git add src/composables/useNotification.ts
git commit -m "feat(notification): add useNotification composable"
```

---

## 阶段 2：UI 组件

### Task 5: 创建 NotificationToast 组件

**Files:**

- Create: `src/components/notification/NotificationToast.vue`

**Step 1: 创建 toast 组件**

```vue
<!-- src/components/notification/NotificationToast.vue -->
<script setup lang="ts">
  import { useNotificationStore } from '@/stores/useNotificationStore';
  import { computed } from 'vue';

  const notificationStore = useNotificationStore();

  const toasts = computed(() => notificationStore.toasts);

  function getIcon(type: string) {
    switch (type) {
      case 'success':
        return 'mdi-check-circle';
      case 'info':
        return 'mdi-information';
      case 'warning':
        return 'mdi-alert';
      case 'error':
        return 'mdi-close-circle';
      default:
        return 'mdi-information';
    }
  }

  function getColor(type: string) {
    switch (type) {
      case 'success':
        return 'success';
      case 'info':
        return 'info';
      case 'warning':
        return 'warning';
      case 'error':
        return 'error';
      default:
        return 'info';
    }
  }

  function removeToast(id: string) {
    notificationStore.removeToast(id);
  }
</script>

<template>
  <div class="notification-toasts">
    <TransitionGroup name="toast">
      <v-card
        v-for="toast in toasts"
        :key="toast.id"
        :color="getColor(toast.type)"
        :class="['notification-toast', `toast-${toast.type}`]"
        elevation="4"
      >
        <div class="toast-content">
          <v-icon :icon="getIcon(toast.type)" class="toast-icon" />
          <div class="toast-text">
            <div class="toast-title">{{ toast.title }}</div>
            <div class="toast-message">{{ toast.message }}</div>
          </div>
          <v-btn
            icon="mdi-close"
            size="small"
            variant="text"
            class="toast-close"
            @click="removeToast(toast.id)"
          />
        </div>
      </v-card>
    </TransitionGroup>
  </div>
</template>

<style scoped>
  .notification-toasts {
    position: fixed;
    bottom: 48px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 2000;
    display: flex;
    flex-direction: column;
    gap: 8px;
    pointer-events: none;
  }

  .notification-toast {
    min-width: 300px;
    max-width: 500px;
    pointer-events: auto;
  }

  .toast-content {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
  }

  .toast-icon {
    flex-shrink: 0;
  }

  .toast-text {
    flex: 1;
    min-width: 0;
  }

  .toast-title {
    font-weight: 500;
    font-size: 14px;
  }

  .toast-message {
    font-size: 13px;
    opacity: 0.9;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .toast-close {
    flex-shrink: 0;
  }

  /* Toast animations */
  .toast-enter-active,
  .toast-leave-active {
    transition: all 0.3s ease;
  }

  .toast-enter-from {
    opacity: 0;
    transform: translateY(20px);
  }

  .toast-leave-to {
    opacity: 0;
    transform: translateX(100%);
  }
</style>
```

**Step 2: 运行 TypeScript 类型检查**

```bash
yarn tsc --noEmit
```

Expected: 无类型错误

**Step 3: Commit**

```bash
git add src/components/notification/NotificationToast.vue
git commit -m "feat(notification): add NotificationToast component"
```

---

### Task 6: 创建 GlobalErrorDialog 组件

**Files:**

- Create: `src/components/notification/GlobalErrorDialog.vue`

**Step 1: 创建全局错误对话框组件**

```vue
<!-- src/components/notification/GlobalErrorDialog.vue -->
<script setup lang="ts">
  import { useNotificationStore } from '@/stores/useNotificationStore';
  import { ref, computed } from 'vue';

  const notificationStore = useNotificationStore();

  const showDialog = computed(() => notificationStore.showDialog);
  const dialogContent = computed(() => notificationStore.dialogContent);

  const showDetails = ref(false);

  function handleClose() {
    showDetails.value = false;
    notificationStore.closeDialog();
  }

  async function copyToClipboard() {
    if (dialogContent.value?.details) {
      try {
        await navigator.clipboard.writeText(dialogContent.value.details);
        console.info('Error details copied to clipboard');
      } catch (error) {
        console.error('Failed to copy to clipboard:', error);
      }
    }
  }

  function getIcon(type: string) {
    switch (type) {
      case 'error':
        return 'mdi-alert-circle';
      case 'warning':
        return 'mdi-alert';
      default:
        return 'mdi-information';
    }
  }

  function getColor(type: string) {
    switch (type) {
      case 'error':
        return 'error';
      case 'warning':
        return 'warning';
      default:
        return 'info';
    }
  }
</script>

<template>
  <v-dialog :model-value="showDialog" @update:model-value="handleClose" max-width="500">
    <v-card v-if="dialogContent">
      <v-card-title class="d-flex align-center">
        <v-icon
          :icon="getIcon(dialogContent.type)"
          :color="getColor(dialogContent.type)"
          class="mr-2"
        />
        <span>{{ dialogContent.title }}</span>
      </v-card-title>

      <v-card-text>
        <div class="error-message">{{ dialogContent.message }}</div>

        <v-expand-transition>
          <div v-if="dialogContent.details" class="error-details-container">
            <v-btn
              variant="text"
              size="small"
              class="toggle-details-btn"
              @click="showDetails = !showDetails"
            >
              {{ showDetails ? 'Hide' : 'Show' }} details
              <v-icon :icon="showDetails ? 'mdi-chevron-up' : 'mdi-chevron-down'" end />
            </v-btn>

            <v-expand-transition>
              <div v-show="showDetails" class="error-details">
                <pre class="error-stack">{{ dialogContent.details }}</pre>
              </div>
            </v-expand-transition>
          </div>
        </v-expand-transition>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn v-if="dialogContent.details" variant="text" @click="copyToClipboard">Copy</v-btn>
        <v-btn :color="getColor(dialogContent.type)" variant="elevated" @click="handleClose">
          OK
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
  .error-message {
    font-size: 15px;
    line-height: 1.5;
    padding: 8px 0;
  }

  .error-details-container {
    margin-top: 16px;
  }

  .toggle-details-btn {
    padding: 0;
  }

  .error-details {
    margin-top: 8px;
    padding: 12px;
    background-color: rgba(0, 0, 0, 0.05);
    border-radius: 4px;
    max-height: 200px;
    overflow-y: auto;
  }

  .error-stack {
    margin: 0;
    font-size: 12px;
    font-family: 'Consolas', 'Monaco', monospace;
    white-space: pre-wrap;
    word-break: break-all;
  }
</style>
```

**Step 2: 运行 TypeScript 类型检查**

```bash
yarn tsc --noEmit
```

Expected: 无类型错误

**Step 3: Commit**

```bash
git add src/components/notification/GlobalErrorDialog.vue
git commit -m "feat(notification): add GlobalErrorDialog component"
```

---

### Task 7: 创建 NotificationHistory 组件

**Files:**

- Create: `src/components/notification/NotificationHistory.vue`

**Step 1: 创建通知历史组件**

```vue
<!-- src/components/notification/NotificationHistory.vue -->
<script setup lang="ts">
  import { useNotification } from '@/composables/useNotification';
  import { useI18n } from '@/lib/i18n';
  import { ref, computed } from 'vue';

  const { t } = useI18n();
  const { history, markAsRead, markAllAsRead, deleteNotification, clearHistory } =
    useNotification();

  const filterType = ref<'all' | 'success' | 'info' | 'warning' | 'error'>('all');
  const filterStatus = ref<'all' | 'read' | 'unread'>('all');

  const filteredHistory = computed(() => {
    let filtered = history.value;

    if (filterType.value !== 'all') {
      filtered = filtered.filter((n) => n.type === filterType.value);
    }

    if (filterStatus.value !== 'read') {
      if (filterStatus.value === 'unread') {
        filtered = filtered.filter((n) => !n.read);
      }
    }

    return filtered;
  });

  const unreadCount = computed(() => history.value.filter((n) => !n.read).length);

  function getIcon(type: string) {
    switch (type) {
      case 'success':
        return 'mdi-check-circle';
      case 'info':
        return 'mdi-information';
      case 'warning':
        return 'mdi-alert';
      case 'error':
        return 'mdi-close-circle';
      default:
        return 'mdi-information';
    }
  }

  function getColor(type: string) {
    switch (type) {
      case 'success':
        return 'success';
      case 'info':
        return 'info';
      case 'warning':
        return 'warning';
      case 'error':
        return 'error';
      default:
        return 'info';
    }
  }

  function formatTimestamp(timestamp: number) {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return t('notification.justNow');
    if (diffMins < 60) return t('notification.minutesAgo', { n: diffMins });
    if (diffHours < 24) return t('notification.hoursAgo', { n: diffHours });
    if (diffDays < 7) return t('notification.daysAgo', { n: diffDays });

    return date.toLocaleDateString();
  }
</script>

<template>
  <div class="notification-history">
    <div class="history-header">
      <h3>{{ t('notification.notificationHistory') }}</h3>
      <div class="header-actions">
        <v-chip v-if="unreadCount > 0" size="small" color="primary">
          {{ unreadCount }} {{ t('notification.unread').toLowerCase() }}
        </v-chip>
        <v-btn v-if="unreadCount > 0" size="small" variant="text" @click="markAllAsRead">
          {{ t('notification.markAllRead') }}
        </v-btn>
        <v-btn size="small" variant="text" color="error" @click="clearHistory">
          {{ t('notification.clearHistory') }}
        </v-btn>
      </div>
    </div>

    <div class="history-filters">
      <v-btn-toggle v-model="filterType" variant="outlined" divided>
        <v-btn value="all">{{ t('notification.all') }}</v-btn>
        <v-btn value="success">
          <v-icon icon="mdi-check-circle" size="small" />
        </v-btn>
        <v-btn value="info">
          <v-icon icon="mdi-information" size="small" />
        </v-btn>
        <v-btn value="warning">
          <v-icon icon="mdi-alert" size="small" />
        </v-btn>
        <v-btn value="error">
          <v-icon icon="mdi-close-circle" size="small" />
        </v-btn>
      </v-btn-toggle>

      <v-btn-toggle v-model="filterStatus" variant="outlined" divided>
        <v-btn value="all">{{ t('notification.all') }}</v-btn>
        <v-btn value="unread">{{ t('notification.unread') }}</v-btn>
        <v-btn value="read">{{ t('notification.read') }}</v-btn>
      </v-btn-toggle>
    </div>

    <div class="history-list">
      <v-list v-if="filteredHistory.length > 0">
        <v-list-item
          v-for="notification in filteredHistory"
          :key="notification.id"
          :class="{ 'notification-unread': !notification.read }"
          @click="markAsRead(notification.id)"
        >
          <template #prepend>
            <v-icon :icon="getIcon(notification.type)" :color="getColor(notification.type)" />
          </template>

          <v-list-item-title>
            {{ notification.title }}
          </v-list-item-title>
          <v-list-item-subtitle>
            {{ notification.message }}
          </v-list-item-subtitle>

          <template #append>
            <div class="notification-actions">
              <span class="notification-time">{{ formatTimestamp(notification.timestamp) }}</span>
              <v-btn
                icon="mdi-close"
                size="small"
                variant="text"
                @click.stop="deleteNotification(notification.id)"
              />
            </div>
          </template>
        </v-list-item>
      </v-list>

      <v-empty-state v-else :text="t('notification.noNotifications')">
        <v-icon icon="mdi-bell-off" size="64" />
      </v-empty-state>
    </div>
  </div>
</template>

<style scoped>
  .notification-history {
    padding: 16px;
  }

  .history-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .history-header h3 {
    margin: 0;
    font-size: 18px;
    font-weight: 500;
  }

  .header-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .history-filters {
    display: flex;
    gap: 16px;
    margin-bottom: 16px;
  }

  .history-list {
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 8px;
    overflow: hidden;
  }

  .notification-unread {
    background-color: rgba(var(--v-theme-primary), 0.08);
  }

  .notification-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .notification-time {
    font-size: 12px;
    opacity: 0.7;
  }
</style>
```

**Step 2: 运行 TypeScript 类型检查**

```bash
yarn tsc --noEmit
```

Expected: 无类型错误

**Step 3: Commit**

```bash
git add src/components/notification/NotificationHistory.vue
git commit -m "feat(notification): add NotificationHistory component"
```

---

### Task 8: 修改 StatusBar 组件集成通知

**Files:**

- Modify: `src/components/layout/StatusBar.vue`

**Step 1: 修改 StatusBar.vue 添加状态栏通知区域**

首先读取当前文件内容，然后在适当位置添加状态文本显示：

```vue
<!-- 在 status-bar-left div 中添加状态文本 -->
<div class="status-bar-left">
  <span v-if="statusText" class="status-text">{{ statusText }}</span>
  <span class="mr-4">{{ t("status.documents") }}: 0</span>
  <span>{{ t("status.version") }}: {{ APP_VERSION }}</span>
</div>

<!-- 在 script setup 中导入 composable -->
import { useNotification } from '@/composables/useNotification'; const { statusText } =
useNotification();
```

**Step 2: 运行 TypeScript 类型检查**

```bash
yarn tsc --noEmit
```

Expected: 无类型错误

**Step 3: Commit**

```bash
git add src/components/layout/StatusBar.vue
git commit -m "feat(notification): integrate status text display in StatusBar"
```

---

### Task 9: 修改 App.vue 集成全局组件

**Files:**

- Modify: `src/App.vue`

**Step 1: 在 App.vue 中添加全局通知组件**

```vue
<script setup lang="ts">
  import StatusBar from '@/components/layout/StatusBar.vue';
  import NotificationToast from '@/components/notification/NotificationToast.vue';
  import GlobalErrorDialog from '@/components/notification/GlobalErrorDialog.vue';
  // ... 其他导入
</script>

<template>
  <v-app>
    <router-view />
    <StatusBar />
    <NotificationToast />
    <GlobalErrorDialog />
  </v-app>
</template>
```

**Step 2: 运行 TypeScript 类型检查**

```bash
yarn tsc --noEmit
```

Expected: 无类型错误

**Step 3: Commit**

```bash
git add src/App.vue
git commit -m "feat(notification): integrate global notification components in App.vue"
```

---

## 阶段 3：国际化

### Task 10: 添加中文翻译

**Files:**

- Modify: `src/lib/i18n/zh.ts`

**Step 1: 在中文翻译文件中添加通知相关翻译**

```typescript
// 在 src/lib/i18n/zh.ts 的 exports 对象中添加
notification: {
  success: '成功',
  info: '信息',
  warning: '警告',
  error: '错误',
  all: '全部',
  operationSuccess: '操作成功',
  operationFailed: '操作失败',
  networkError: '网络连接失败',
  saveSuccess: '保存成功',
  deleteSuccess: '删除成功',
  details: '详情',
  copy: '复制',
  close: '关闭',
  clearHistory: '清空历史',
  markAllRead: '全部标为已读',
  unread: '未读',
  read: '已读',
  notificationHistory: '通知历史',
  noNotifications: '暂无通知',
  justNow: '刚刚',
  minutesAgo: '{n} 分钟前',
  hoursAgo: '{n} 小时前',
  daysAgo: '{n} 天前',
},
```

**Step 2: 运行 TypeScript 类型检查**

```bash
yarn tsc --noEmit
```

Expected: 无类型错误

**Step 3: Commit**

```bash
git add src/lib/i18n/zh.ts
git commit -m "feat(notification): add Chinese translations for notifications"
```

---

### Task 11: 添加英文翻译

**Files:**

- Modify: `src/lib/i18n/en.ts`

**Step 1: 在英文翻译文件中添加通知相关翻译**

```typescript
// 在 src/lib/i18n/en.ts 的 exports 对象中添加
notification: {
  success: 'Success',
  info: 'Info',
  warning: 'Warning',
  error: 'Error',
  all: 'All',
  operationSuccess: 'Operation successful',
  operationFailed: 'Operation failed',
  networkError: 'Network connection failed',
  saveSuccess: 'Saved successfully',
  deleteSuccess: 'Deleted successfully',
  details: 'Details',
  copy: 'Copy',
  close: 'Close',
  clearHistory: 'Clear History',
  markAllRead: 'Mark All as Read',
  unread: 'Unread',
  read: 'Read',
  notificationHistory: 'Notification History',
  noNotifications: 'No notifications',
  justNow: 'Just now',
  minutesAgo: '{n} min ago',
  hoursAgo: '{n} hr ago',
  daysAgo: '{n} days ago',
},
```

**Step 2: 运行 TypeScript 类型检查**

```bash
yarn tsc --noEmit
```

Expected: 无类型错误

**Step 3: Commit**

```bash
git add src/lib/i18n/en.ts
git commit -m "feat(notification): add English translations for notifications"
```

---

## 阶段 4：Rust 后端

### Task 12: 创建 Rust 通知类型定义

**Files:**

- Create: `src-tauri/src/notification/types.rs`

**Step 1: 创建 Rust 通知类型文件**

```rust
// src-tauri/src/notification/types.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationType {
    Success,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationDisplay {
    Toast,
    Status,
    Dialog,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPayload {
    #[serde(rename = "type")]
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<NotificationDisplay>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}
```

**Step 2: 编译检查**

```bash
cd src-tauri && cargo check
```

Expected: 编译成功，无错误

**Step 3: Commit**

```bash
git add src-tauri/src/notification/types.rs
git commit -m "feat(notification): add Rust notification type definitions"
```

---

### Task 13: 创建 Rust 通知发送器

**Files:**

- Create: `src-tauri/src/notification/emitter.rs`

**Step 1: 创建 Rust 通知发送器**

```rust
// src-tauri/src/notification/emitter.rs

use super::types::{NotificationDisplay, NotificationPayload, NotificationType};
use serde_json::json;
use tauri::{AppHandle, Manager};

pub struct NotificationEmitter<'a> {
    app_handle: &'a AppHandle,
    notification_type: NotificationType,
    title: String,
    message: String,
    display: Option<NotificationDisplay>,
    persistent: Option<bool>,
    duration: Option<u64>,
    details: Option<String>,
}

impl<'a> NotificationEmitter<'a> {
    pub fn new(app_handle: &'a AppHandle, notification_type: NotificationType) -> Self {
        Self {
            app_handle,
            notification_type,
            title: String::new(),
            message: String::new(),
            display: None,
            persistent: None,
            duration: None,
            details: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn display(mut self, display: NotificationDisplay) -> Self {
        self.display = Some(display);
        self
    }

    pub fn persistent(mut self, persistent: bool) -> Self {
        self.persistent = Some(persistent);
        self
    }

    pub fn duration(mut self, duration: u64) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn send(self) -> Result<(), Box<dyn std::error::Error>> {
        let event_name = format!("notification:{}", self.notification_type_as_str());
        let payload = NotificationPayload {
            notification_type: self.notification_type,
            title: self.title,
            message: self.message,
            display: self.display,
            persistent: self.persistent,
            duration: self.duration,
            details: self.details,
        };

        self.app_handle.emit(&event_name, payload)?;
        Ok(())
    }

    fn notification_type_as_str(&self) -> &'static str {
        match self.notification_type {
            NotificationType::Success => "success",
            NotificationType::Info => "info",
            NotificationType::Warning => "warning",
            NotificationType::Error => "error",
        }
    }
}

// 便捷构造函数
impl<'a> NotificationEmitter<'a> {
    pub fn success(app_handle: &'a AppHandle) -> Self {
        Self::new(app_handle, NotificationType::Success)
            .title("Success")
    }

    pub fn info(app_handle: &'a AppHandle) -> Self {
        Self::new(app_handle, NotificationType::Info)
            .title("Info")
    }

    pub fn warning(app_handle: &'a AppHandle) -> Self {
        Self::new(app_handle, NotificationType::Warning)
            .title("Warning")
    }

    pub fn error(app_handle: &'a AppHandle) -> Self {
        Self::new(app_handle, NotificationType::Error)
            .title("Error")
    }
}
```

**Step 2: 编译检查**

```bash
cd src-tauri && cargo check
```

Expected: 编译成功，无错误

**Step 3: Commit**

```bash
git add src-tauri/src/notification/emitter.rs
git commit -m "feat(notification): add Rust notification emitter"
```

---

### Task 14: 创建 Rust 通知模块入口

**Files:**

- Create: `src-tauri/src/notification/mod.rs`

**Step 1: 创建模块入口文件**

```rust
// src-tauri/src/notification/mod.rs

pub mod emitter;
pub mod types;

pub use emitter::NotificationEmitter;
pub use types::{NotificationDisplay, NotificationPayload, NotificationType};
```

**Step 2: 编译检查**

```bash
cd src-tauri && cargo check
```

Expected: 编译成功，无错误

**Step 3: Commit**

```bash
git add src-tauri/src/notification/mod.rs
git commit -m "feat(notification): add notification module entry point"
```

---

### Task 15: 在 lib.rs 中注册通知模块

**Files:**

- Modify: `src-tauri/src/lib.rs`

**Step 1: 在 lib.rs 中添加通知模块**

```rust
// 在文件顶部的 mod 声明区域添加
mod notification;
```

**Step 2: 编译检查**

```bash
cd src-tauri && cargo check
```

Expected: 编译成功，无错误

**Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(notification): register notification module in lib.rs"
```

---

### Task 16: 在前端添加 Tauri 事件监听器

**Files:**

- Create: `src/lib/notification-events.ts`

**Step 1: 创建 Tauri 事件监听器**

```typescript
// src/lib/notification-events.ts
import { listen } from '@tauri-apps/api/event';
import type { NotificationPayload } from '@/types/notification';
import { useNotificationStore } from '@/stores/useNotificationStore';

interface TauriNotificationPayload {
  type: 'success' | 'info' | 'warning' | 'error';
  title: string;
  message: string;
  display?: 'toast' | 'status' | 'dialog' | 'system';
  persistent?: boolean;
  duration?: number;
  details?: string;
}

/**
 * 初始化 Tauri 通知事件监听器
 * 在应用启动时调用此函数
 */
export async function initNotificationListeners(): Promise<void> {
  const notificationStore = useNotificationStore();

  // 监听成功通知
  const unlistenSuccess = await listen<TauriNotificationPayload>(
    'notification:success',
    (event) => {
      const payload = event.payload;
      notificationStore.add({
        type: 'success',
        title: payload.title,
        message: payload.message,
        display: payload.display || 'toast',
        persistent: payload.persistent ?? true,
        duration: payload.duration,
        details: payload.details,
      });
    }
  );

  // 监听信息通知
  const unlistenInfo = await listen<TauriNotificationPayload>('notification:info', (event) => {
    const payload = event.payload;
    notificationStore.add({
      type: 'info',
      title: payload.title,
      message: payload.message,
      display: payload.display || 'toast',
      persistent: payload.persistent ?? true,
      duration: payload.duration,
      details: payload.details,
    });
  });

  // 监听警告通知
  const unlistenWarning = await listen<TauriNotificationPayload>(
    'notification:warning',
    (event) => {
      const payload = event.payload;
      notificationStore.add({
        type: 'warning',
        title: payload.title,
        message: payload.message,
        display: payload.display || 'toast',
        persistent: payload.persistent ?? true,
        duration: payload.duration,
        details: payload.details,
      });
    }
  );

  // 监听错误通知
  const unlistenError = await listen<TauriNotificationPayload>('notification:error', (event) => {
    const payload = event.payload;
    notificationStore.add({
      type: 'error',
      title: payload.title,
      message: payload.message,
      display: payload.display || 'toast',
      persistent: payload.persistent ?? true,
      duration: payload.duration,
      details: payload.details,
    });
  });

  console.info('Notification listeners initialized');

  // 返回清理函数
  return () => {
    unlistenSuccess();
    unlistenInfo();
    unlistenWarning();
    unlistenError();
  };
}
```

**Step 2: 运行 TypeScript 类型检查**

```bash
yarn tsc --noEmit
```

Expected: 无类型错误

**Step 3: Commit**

```bash
git add src/lib/notification-events.ts
git commit -m "feat(notification): add Tauri event listeners for backend notifications"
```

---

### Task 17: 在 App.vue 中初始化事件监听器

**Files:**

- Modify: `src/App.vue`

**Step 1: 在 App.vue 中添加事件监听器初始化**

```vue
<script setup lang="ts">
  import StatusBar from '@/components/layout/StatusBar.vue';
  import NotificationToast from '@/components/notification/NotificationToast.vue';
  import GlobalErrorDialog from '@/components/notification/GlobalErrorDialog.vue';
  import { initNotificationListeners } from '@/lib/notification-events';
  import { invokeCommand } from '@/lib/tauri';
  import { useAppStore } from '@/stores/useAppStore';
  import { onMounted, provide, ref, watch } from 'vue';
  import { useTheme } from 'vuetify';

  const appStore = useAppStore();
  const theme = useTheme();

  // Config version for tracking changes
  const configVersion = ref(0);

  // Function to reload config from backend
  async function reloadConfig() {
    try {
      const data = await invokeCommand<any>('get_app_config');
      if (data?.system?.llm_providers) {
        appStore.setLLMProviders(data.system.llm_providers);
      }
      if (data?.paper?.grobid?.servers) {
        appStore.setGrobidServers(data.paper.grobid.servers);
      }
      configVersion.value++;
      console.info('Configuration reloaded');
    } catch (error) {
      console.error('Failed to reload config:', error);
    }
  }

  // Provide the reload function to child components
  provide('reloadConfig', reloadConfig);

  // Initialize theme on mount
  onMounted(async () => {
    theme.change(appStore.isDark ? 'dark' : 'light');
    // 初始化通知事件监听器
    await initNotificationListeners();
  });

  // Watch for theme changes in store and update Vuetify theme
  watch(
    () => appStore.isDark,
    (isDark) => {
      theme.change(isDark ? 'dark' : 'light');
    }
  );
</script>

<template>
  <v-app>
    <router-view />
    <StatusBar />
    <NotificationToast />
    <GlobalErrorDialog />
  </v-app>
</template>

<style>
  /* Global styles */
</style>
```

**Step 2: 运行 TypeScript 类型检查**

```bash
yarn tsc --noEmit
```

Expected: 无类型错误

**Step 3: Commit**

```bash
git add src/App.vue
git commit -m "feat(notification): initialize notification event listeners in App.vue"
```

---

## 阶段 5：集成与测试

### Task 18: 在设置页面添加通知历史面板

**Files:**

- Modify: `src/pages/SettingsPage.vue` (or the appropriate settings component)

**Step 1: 在设置页面中添加通知历史组件**

首先检查设置页面的现有结构，然后添加通知历史面板：

```vue
<script setup lang="ts">
  import NotificationHistory from '@/components/notification/NotificationHistory.vue';
  // ... 其他导入
</script>

<template>
  <v-container>
    <!-- 现有的设置选项 -->
    <v-tabs>
      <!-- 现有标签页 -->
      <v-tab value="notifications">Notifications</v-tab>
    </v-tabs>

    <v-window-item value="notifications">
      <NotificationHistory />
    </v-window-item>
  </v-container>
</template>
```

**Step 2: 运行 TypeScript 类型检查**

```bash
yarn tsc --noEmit
```

Expected: 无类型错误

**Step 3: Commit**

```bash
git add src/pages/SettingsPage.vue
git commit -m "feat(notification): add notification history panel to settings page"
```

---

### Task 19: 更新现有组件使用新的通知系统

**Files:**

- Modify: `src/components/dialogs/AddCategoryDialog.vue` (示例)

**Step 1: 将 AddCategoryDialog 中的错误处理迁移到新通知系统**

```vue
<script setup lang="ts">
  import { useI18n } from '@/lib/i18n';
  import { invokeCommand } from '@/lib/tauri';
  import { useNotification } from '@/composables/useNotification';
  import { ref, watch } from 'vue';

  const { t } = useI18n();
  const { showSuccess, showError, setStatus, clearStatus } = useNotification();

  // ... 现有代码

  // 修改 submit 处理函数
  async function handleSubmit() {
    if (!name.value.trim()) {
      showError(t('dialog.categoryNameRequired'));
      return;
    }

    if (name.value.length > 50) {
      showError(t('dialog.categoryNameMaxLength'));
      return;
    }

    loading.value = true;
    setStatus('Creating category...');

    try {
      await invokeCommand('create_category', {
        name: name.value.trim(),
        parentId: props.parentId || null,
      });
      console.info('Category created successfully:', name.value.trim());
      showSuccess(t('notification.saveSuccess'));
      name.value = '';
      error.value = '';
      emit('categoryCreated');
      emit('update:modelValue', false);
    } catch (err) {
      const errorMessage =
        typeof err === 'string' ? err : (err as { message?: string })?.message || String(err);
      showError(t('notification.operationFailed'), {
        details: errorMessage,
        display: 'dialog',
      });
    } finally {
      loading.value = false;
      clearStatus();
    }
  }
</script>
```

**Step 2: 运行 TypeScript 类型检查**

```bash
yarn tsc --noEmit
```

Expected: 无类型错误

**Step 3: Commit**

```bash
git add src/components/dialogs/AddCategoryDialog.vue
git commit -m "refactor(notification): migrate AddCategoryDialog to new notification system"
```

---

### Task 20: 示例 Rust 命令使用通知系统

**Files:**

- Modify: `src-tauri/src/command/category_command.rs` (示例)

**Step 1: 在现有命令中添加通知发送**

```rust
use crate::notification::NotificationEmitter;
use tauri::AppHandle;

// 在命令函数中添加通知
#[tauri::command]
pub async fn create_category(
    name: String,
    parent_id: Option<i32>,
    app_handle: AppHandle,
) -> Result<(), String> {
    // 发送开始通知
    NotificationEmitter::info(&app_handle)
        .title("Creating Category")
        .message(&format!("Creating category: {}", name))
        .display(crate::notification::NotificationDisplay::Status)
        .send()
        .map_err(|e| e.to_string())?;

    // ... 业务逻辑

    // 发送成功通知
    NotificationEmitter::success(&app_handle)
        .title("Category Created")
        .message(&format!("Category '{}' created successfully", name))
        .display(crate::notification::NotificationDisplay::Toast)
        .persistent(true)
        .send()
        .map_err(|e| e.to_string())?;

    Ok(())
}
```

**Step 2: 编译检查**

```bash
cd src-tauri && cargo check
```

Expected: 编译成功，无错误

**Step 3: Commit**

```bash
git add src-tauri/src/command/category_command.rs
git commit -m "refactor(notification): migrate create_category to use notification system"
```

---

## 阶段 6：测试与验证

### Task 21: 手动测试通知系统

**Step 1: 启动开发服务器**

```bash
yarn tauri:dev
```

**Step 2: 测试场景清单**

- [ ] Toast 通知：触发成功操作，验证绿色 toast 显示在状态栏上方
- [ ] Info 通知：验证蓝色信息通知显示
- [ ] Warning 通知：验证黄色警告通知显示
- [ ] Error toast：验证红色错误 toast 显示
- [ ] Error 对话框：触发错误，验证全局错误对话框弹出
- [ ] 状态栏固定状态：设置状态文本，验证显示在状态栏左侧
- [ ] 系统通知：最小化应用，触发通知，验证系统通知弹出
- [ ] 通知历史：打开设置页面，验证通知历史显示
- [ ] 筛选通知：按类型/状态筛选，验证过滤正确
- [ ] 标记已读：点击通知，验证标记为已读
- [ ] 清空历史：点击清空按钮，验证历史清空
- [ ] 国际化：切换语言，验证通知文本正确翻译

**Step 3: 记录测试结果**

创建测试报告文档 `docs/notification-testing-report.md` 记录测试结果。

---

### Task 22: 运行代码质量检查

**Step 1: 运行 ESLint 检查**

```bash
yarn lint
```

Expected: 无错误

**Step 2: 运行 Prettier 格式化**

```bash
yarn format
```

**Step 3: 运行 Rust Clippy**

```bash
cd src-tauri && cargo clippy
```

Expected: 无警告（或已修复）

**Step 4: Commit 代码格式调整**

```bash
git add -A
git commit -m "style(notification): format code and fix linting issues"
```

---

### Task 23: 构建测试

**Step 1: 构建前端**

```bash
yarn build
```

Expected: 构建成功，无错误

**Step 2: 构建 Tauri 应用**

```bash
yarn tauri build
```

Expected: 构建成功，生成安装包

**Step 3: 验证构建产物**

检查生成的安装包是否正确：

```bash
ls -lh src-tauri/target/release/bundle/
```

---

## 验收标准

完成所有任务后，通知系统应该满足以下要求：

1. ✅ 用户操作成功时显示绿色 toast 通知
2. ✅ 发生错误时显示全局对话框（带详情）
3. ✅ 应用最小化时使用系统通知
4. ✅ 通知历史保存并可查看
5. ✅ 支持四种通知类型：成功、信息、警告、错误
6. ✅ 支持四种显示方式：toast、status、dialog、system
7. ✅ 前后端都能发送通知
8. ✅ 国际化支持（中英文）
9. ✅ TypeScript 类型安全
10. ✅ 通过所有代码质量检查

---

## 关键文件总结

**新增文件 (9个)**:

- `src/types/notification.ts`
- `src/stores/useNotificationStore.ts`
- `src/lib/notification.ts`
- `src/composables/useNotification.ts`
- `src/lib/notification-events.ts`
- `src/components/notification/NotificationToast.vue`
- `src/components/notification/GlobalErrorDialog.vue`
- `src/components/notification/NotificationHistory.vue`
- `src-tauri/src/notification/mod.rs`
- `src-tauri/src/notification/types.rs`
- `src-tauri/src/notification/emitter.rs`

**修改文件 (6个)**:

- `src/App.vue`
- `src/components/layout/StatusBar.vue`
- `src/lib/i18n/zh.ts`
- `src/lib/i18n/en.ts`
- `src/components/dialogs/AddCategoryDialog.vue` (示例)
- `src-tauri/src/lib.rs`
- `src-tauri/src/command/category_command.rs` (示例)
