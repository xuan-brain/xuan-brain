<script setup lang="ts">
  import StatusBar from '@/components/layout/StatusBar.vue';
  import GlobalErrorDialog from '@/components/notification/GlobalErrorDialog.vue';
  import NotificationToast from '@/components/notification/NotificationToast.vue';
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
    // Initialize Tauri notification event listeners
    try {
      await initNotificationListeners();
    } catch (error) {
      console.error('Failed to initialize notification listeners:', error);
    }
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
