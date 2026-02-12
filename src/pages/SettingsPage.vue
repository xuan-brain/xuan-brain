<script setup lang="ts">
import { ref, inject } from "vue";
import { useI18n } from "@/lib/i18n";
import AppearanceSettings from "@/components/settings/AppearanceSettings.vue";
import AISettings from "@/components/settings/AISettings.vue";
import PapersSettings from "@/components/settings/PapersSettings.vue";
import AboutSettings from "@/components/settings/AboutSettings.vue";
import UserSettings from "@/components/settings/UserSettings.vue";
import ClipsSettings from "@/components/settings/ClipsSettings.vue";
import WritingSettings from "@/components/settings/WritingSettings.vue";
import SubscriptionsSettings from "@/components/settings/SubscriptionsSettings.vue";

const { t } = useI18n();

// Inject reloadConfig function from App.vue
const reloadConfig = inject<(() => Promise<void>) | null>("reloadConfig", null);

// Active tab
const activeTab = ref("system");

// Tab items with i18n
const tabItems = [
  { key: "user", i18n: "navigation.user", icon: "mdi-account" },
  { key: "system", i18n: "navigation.system", icon: "mdi-desktop-classic" },
  { key: "ai", i18n: "navigation.ai", icon: "mdi-brain" },
  { key: "papers", i18n: "navigation.papers", icon: "mdi-file-document" },
  { key: "clips", i18n: "navigation.clips", icon: "mdi-content-cut" },
  { key: "writing", i18n: "navigation.writing", icon: "mdi-pencil" },
  { key: "subscriptions", i18n: "navigation.subscriptions", icon: "mdi-rss" },
  { key: "about", i18n: "navigation.about", icon: "mdi-information" },
];

// Handle config updated event from child components
async function handleConfigUpdated() {
  console.info("Settings configuration updated, reloading...");
  // Reload config from backend and update store
  if (reloadConfig) {
    await reloadConfig();
  }
}
</script>

<template>
  <div class="settings-page">
    <h1 class="text-h4 mb-6">{{ t("navigation.settings") }}</h1>

    <v-row>
      <!-- Left Navigation -->
      <v-col cols="2">
        <v-card>
          <v-list>
            <v-list-item
              v-for="item in tabItems"
              :key="item.key"
              :value="item.key"
              :active="activeTab === item.key"
              @click="activeTab = item.key"
            >
              <template #prepend>
                <v-icon>{{ item.icon }}</v-icon>
              </template>
              <v-list-item-title>{{ t(item.i18n) }}</v-list-item-title>
            </v-list-item>
          </v-list>
        </v-card>
      </v-col>

      <!-- Right Content -->
      <v-col cols="10">
        <div class="settings-content">
          <!-- User Settings -->
          <UserSettings v-if="activeTab === 'user'" />

          <!-- System Settings -->
          <AppearanceSettings v-if="activeTab === 'system'" />

          <!-- AI Settings -->
          <AISettings
            v-if="activeTab === 'ai'"
            @config-updated="handleConfigUpdated"
          />

          <!-- Papers Settings -->
          <PapersSettings
            v-if="activeTab === 'papers'"
            @config-updated="handleConfigUpdated"
          />

          <!-- Clips Settings -->
          <ClipsSettings v-if="activeTab === 'clips'" />

          <!-- Writing Settings -->
          <WritingSettings v-if="activeTab === 'writing'" />

          <!-- Subscriptions Settings -->
          <SubscriptionsSettings v-if="activeTab === 'subscriptions'" />

          <!-- About -->
          <AboutSettings v-if="activeTab === 'about'" />
        </div>
      </v-col>
    </v-row>
  </div>
</template>

<style scoped>
.settings-page {
  height: 100%;
  padding: 24px;
}

.settings-content {
  max-width: 800px;
}
</style>
