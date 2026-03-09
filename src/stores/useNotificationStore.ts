// src/stores/useNotificationStore.ts
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Notification, NotificationOptions } from '@/types/notification';
import { NotificationType, NotificationDisplay } from '@/types/notification';

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

    // Add notification (main entry point)
    function add(notification: Partial<Notification>): void {
      const fullNotification: Notification = {
        id: `notif-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        type: notification.type || NotificationType.Info,
        title: notification.title || '',
        message: notification.message || '',
        display: notification.display || NotificationDisplay.Toast,
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
        case NotificationDisplay.StatusBar:
          statusText.value = fullNotification.message;
          break;
        case NotificationDisplay.Dialog:
          dialogContent.value = fullNotification;
          showDialog.value = true;
          break;
        case NotificationDisplay.Toast:
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
      add({ type: NotificationType.Success, title: 'Success', message, ...options });
    }

    function info(message: string, options: NotificationOptions = {}): void {
      add({ type: NotificationType.Info, title: 'Info', message, ...options });
    }

    function warning(message: string, options: NotificationOptions = {}): void {
      add({ type: NotificationType.Warning, title: 'Warning', message, ...options });
    }

    function error(message: string, options: NotificationOptions = {}): void {
      add({ type: NotificationType.Error, title: 'Error', message, ...options });
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
