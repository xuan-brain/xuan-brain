// src/composables/useNotification.ts
import { toRefs } from 'vue';
import { useNotificationStore } from '@/stores/useNotificationStore';
import type { NotificationOptions } from '@/types/notification';
import { NotificationDisplay } from '@/types/notification';

/**
 * 通知 composable，提供便捷的通知调用方法
 */
export function useNotification() {
  const notificationStore = useNotificationStore();

  // Use toRefs to maintain reactivity when destructuring from Pinia store
  const {
    toasts,
    history,
    showDialog,
    dialogContent,
    unreadCount,
  } = toRefs(notificationStore);

  return {
    // Toast 通知
    success: (message: string, options?: NotificationOptions) =>
      notificationStore.success(message, { display: NotificationDisplay.Toast, ...options }),
    showSuccess: (message: string, options?: NotificationOptions) =>
      notificationStore.success(message, { display: NotificationDisplay.Toast, ...options }),
    info: (message: string, options?: NotificationOptions) =>
      notificationStore.info(message, { display: NotificationDisplay.Toast, ...options }),
    warning: (message: string, options?: NotificationOptions) =>
      notificationStore.warning(message, { display: NotificationDisplay.Toast, ...options }),
    error: (message: string, options?: NotificationOptions) =>
      notificationStore.error(message, { display: NotificationDisplay.Toast, ...options }),
    showError: (message: string, options?: NotificationOptions) =>
      notificationStore.error(message, { display: NotificationDisplay.Toast, ...options }),

    // 对话框通知
    dialogSuccess: (message: string, options?: NotificationOptions) =>
      notificationStore.success(message, { display: NotificationDisplay.Dialog, ...options }),
    dialogError: (message: string, options?: NotificationOptions) =>
      notificationStore.error(message, { display: NotificationDisplay.Dialog, ...options }),

    // Store 访问 (保持响应式)
    toasts,
    history,
    showDialog,
    dialogContent,
    unreadCount,

    // 历史记录管理
    markAsRead: (id: string) => notificationStore.markAsRead(id),
    markAllAsRead: () => notificationStore.markAllAsRead(),
    deleteNotification: (id: string) => notificationStore.deleteNotification(id),
    clearHistory: () => notificationStore.clearHistory(),
    closeDialog: () => notificationStore.closeDialog(),
  };
}
