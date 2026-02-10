<script setup lang="ts">
import { onMounted, watch } from "vue";
import { useTheme } from "vuetify";
import StatusBar from "@/components/layout/StatusBar.vue";
import { useAppStore } from "@/stores/useAppStore";

const appStore = useAppStore();
const theme = useTheme();

// Initialize theme on mount
onMounted(() => {
  theme.global.name.value = appStore.isDark ? "dark" : "light";
});

// Watch for theme changes in store and update Vuetify theme
watch(
  () => appStore.isDark,
  (isDark) => {
    theme.global.name.value = isDark ? "dark" : "light";
  },
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
