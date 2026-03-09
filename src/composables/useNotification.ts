// src/composables/useNotification.ts
import { useNotificationStore } from '@/stores/useNotificationStore';
import type { NotificationOptions } from '@/types/notification';
import { NotificationDisplay } from '@/types/notification';

/**
 * 通知 composable，提供便捷的通知调用方法
 */
export function useNotification() {
  const notificationStore = useNotificationStore();

  return {
    // Toast 通知
    success: (message: string, options?: NotificationOptions) =>
      notificationStore.success(message, { display: NotificationDisplay.Toast, ...options }),
    info: (message: string, options?: NotificationOptions) =>
      notificationStore.info(message, { display: NotificationDisplay.Toast, ...options }),
    warning: (message: string, options?: NotificationOptions) =>
      notificationStore.warning(message, { display: NotificationDisplay.Toast, ...options }),
    error: (message: string, options?: NotificationOptions) =>
      notificationStore.error(message, { display: NotificationDisplay.Toast, ...options }),

    // 对话框通知
    dialogSuccess: (message: string, options?: NotificationOptions) =>
      notificationStore.success(message, { display: NotificationDisplay.Dialog, ...options }),
    dialogError: (message: string, options?: NotificationOptions) =>
      notificationStore.error(message, { display: NotificationDisplay.Dialog, ...options }),

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
