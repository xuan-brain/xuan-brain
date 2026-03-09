// src/types/notification.ts
export enum NotificationType {
  Success = 'success',
  Info = 'info',
  Warning = 'warning',
  Error = 'error'
}

export enum NotificationDisplay {
  Toast = 'toast',
  StatusBar = 'status',
  Dialog = 'dialog',
  System = 'system'
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
