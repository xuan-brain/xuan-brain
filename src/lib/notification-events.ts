// src/lib/notification-events.ts
import { listen } from '@tauri-apps/api/event';
import { useNotificationStore } from '@/stores/useNotificationStore';
import { NotificationType, NotificationDisplay, type Notification } from '@/types/notification';

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

  // Helper function to map display string to NotificationDisplay enum
  function mapDisplayType(display?: string): NotificationDisplay {
    switch (display) {
      case 'status':
        return NotificationDisplay.StatusBar;
      case 'dialog':
        return NotificationDisplay.Dialog;
      case 'system':
        return NotificationDisplay.System;
      case 'toast':
      default:
        return NotificationDisplay.Toast;
    }
  }

  // Helper function to create notification from payload
  function createNotification(payload: TauriNotificationPayload, type: NotificationType): Partial<Notification> {
    return {
      type,
      title: payload.title,
      message: payload.message,
      display: mapDisplayType(payload.display),
      persistent: payload.persistent ?? true,
      duration: payload.duration,
      details: payload.details,
    };
  }

  // 监听成功通知
  await listen<TauriNotificationPayload>(
    'notification:success',
    (event) => {
      notificationStore.add(createNotification(event.payload, NotificationType.Success));
    }
  );

  // 监听信息通知
  await listen<TauriNotificationPayload>(
    'notification:info',
    (event) => {
      notificationStore.add(createNotification(event.payload, NotificationType.Info));
    }
  );

  // 监听警告通知
  await listen<TauriNotificationPayload>(
    'notification:warning',
    (event) => {
      notificationStore.add(createNotification(event.payload, NotificationType.Warning));
    }
  );

  // 监听错误通知
  await listen<TauriNotificationPayload>(
    'notification:error',
    (event) => {
      notificationStore.add(createNotification(event.payload, NotificationType.Error));
    }
  );

  console.info('Notification listeners initialized');
}
