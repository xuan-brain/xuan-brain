<script setup lang="ts">
  import StatusBar from '@/components/layout/StatusBar.vue';
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
  onMounted(() => {
    theme.change(appStore.isDark ? 'dark' : 'light');
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
  </v-app>
</template>

<style>
  /* Global styles */
</style>
