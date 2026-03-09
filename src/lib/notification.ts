// src/lib/notification.ts
import { getCurrentWindow } from '@tauri-apps/api/window';
import { sendNotification as sendTauriNotification, isPermissionGranted, requestPermission } from '@tauri-apps/plugin-notification';
import { useNotificationStore } from '@/stores/useNotificationStore';
import type { Notification, NotificationOptions } from '@/types/notification';
import { NotificationType, NotificationDisplay } from '@/types/notification';

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
      await sendTauriNotification({
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
  type: NotificationType,
  title: string,
  message: string,
  options: NotificationOptions = {}
): Promise<void> {
  const notificationStore = useNotificationStore();

  // 如果指定了 system 显示方式，检查应用状态
  if (options.display === NotificationDisplay.System || (await shouldUseSystemNotification())) {
    const notification: Notification = {
      id: `notif-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      type,
      title,
      message,
      display: NotificationDisplay.System,
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
    sendNotification(NotificationType.Success, 'Success', message, options),
  info: (message: string, options?: NotificationOptions) =>
    sendNotification(NotificationType.Info, 'Info', message, options),
  warning: (message: string, options?: NotificationOptions) =>
    sendNotification(NotificationType.Warning, 'Warning', message, options),
  error: (message: string, options?: NotificationOptions) =>
    sendNotification(NotificationType.Error, 'Error', message, options),
};
