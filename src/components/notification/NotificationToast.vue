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
