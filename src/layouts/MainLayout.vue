<script setup lang="ts">
import { computed, ref } from "vue";
import { useRoute } from "vue-router";
import { useI18n } from "@/lib/i18n";
import GlobalSidebar from "@/components/layout/GlobalSidebar.vue";
import Navigation from "@/components/navigation/Navigation.vue";

const { t } = useI18n();

// Determine if we should show left navigation drawer for papers page
const route = useRoute();
const isPapersPage = computed(() => route.path.startsWith("/papers"));

// Selected category path (for communication with PapersPage)
const selectedCategory = ref<string | null>(null);

// Handle category selection from navigation
function handleCategorySelect(path: string | null) {
  selectedCategory.value = path;
}
</script>

<template>
  <v-layout class="main-layout">
    <!-- Global sidebar (Rail mode) -->
    <GlobalSidebar />

    <!-- Left navigation drawer (categories and labels) - only on papers page -->
    <v-navigation-drawer
      v-if="isPapersPage"
      location="left"
      permanent
      width="280"
      class="category-drawer"
    >
      <Navigation @category-select="handleCategorySelect" />
    </v-navigation-drawer>

    <!-- Main content area -->
    <v-main>
      <!-- Pass selectedCategory to PapersPage via router-view or provide/inject -->
      <router-view :selected-category="selectedCategory" />
    </v-main>

    <!-- Bottom status bar -->
    <v-footer height="36" class="status-bar">
      <div
        class="d-flex align-center px-2"
        style="height: 100%; font-size: 12px"
      >
        <span class="mr-4">{{ t("status.documents") }}: 0</span>
        <span>{{ t("status.version") }}: 0.1.0</span>
      </div>
    </v-footer>
  </v-layout>
</template>

<style scoped>
.main-layout {
  height: 100vh;
}

.category-drawer {
  border-right: 1px solid rgba(255, 255, 255, 0.12);
}

.status-bar {
  padding: 0 8px;
  display: flex;
  align-items: center;
  font-size: 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.12);
}
</style>
