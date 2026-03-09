<!-- src/components/notification/NotificationHistory.vue -->
<script setup lang="ts">
import { useNotification } from '@/composables/useNotification';
import { useI18n } from '@/lib/i18n';
import { ref, computed } from 'vue';

const { t } = useI18n();
const {
  history,
  markAsRead,
  markAllAsRead,
  deleteNotification,
  clearHistory,
} = useNotification();

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
        <v-btn
          v-if="unreadCount > 0"
          size="small"
          variant="text"
          @click="markAllAsRead"
        >
          {{ t('notification.markAllRead') }}
        </v-btn>
        <v-btn
          size="small"
          variant="text"
          color="error"
          @click="clearHistory"
        >
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
