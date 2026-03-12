<!-- src/components/notification/NotificationToast.vue -->
<script setup lang="ts">
  import { useNotificationStore } from '@/stores/useNotificationStore';
  import { computed, ref, watch } from 'vue';

  const notificationStore = useNotificationStore();

  // 只显示第一个 toast，避免多个 snackbar 同时显示
  const currentToast = computed(() => notificationStore.toasts[0] || null);

  // 控制显示状态
  const showSnackbar = ref(false);

  // 监听 toast 变化，自动显示/隐藏
  watch(() => notificationStore.toasts.length, (length) => {
    showSnackbar.value = length > 0;
  }, { immediate: true });

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

  function handleClose() {
    if (currentToast.value) {
      notificationStore.removeToast(currentToast.value.id);
    }
    showSnackbar.value = false;
  }
</script>

<template>
  <v-snackbar
    v-model="showSnackbar"
    v-if="currentToast"
    :color="getColor(currentToast.type)"
    :timeout="currentToast.duration || 3000"
    location="bottom"
  >
    {{ currentToast.message }}
    <template #actions>
      <v-btn variant="text" @click="handleClose">
        关闭
      </v-btn>
    </template>
  </v-snackbar>
</template>

<style scoped>
/* 样式由 Vuetify 处理 */
</style>
