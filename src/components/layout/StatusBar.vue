<script setup lang="ts">
import { computed, ref, onMounted } from "vue";
import { useI18n, setLocale } from "@/lib/i18n";
import { useAppStore } from "@/stores/useAppStore";
import { invokeCommand } from "@/lib/tauri";

const { t, locale: localeRef, availableLocales } = useI18n();
const appStore = useAppStore();

// Current locale
const currentLocale = computed(
  () => localeRef.value as keyof typeof availableLocales,
);

// Available accent colors
const accentColors = [
  { name: "Blue", value: "#3b82f6" },
  { name: "Purple", value: "#8b5cf6" },
  { name: "Pink", value: "#ec4899" },
  { name: "Red", value: "#ef4444" },
  { name: "Orange", value: "#f97316" },
  { name: "Green", value: "#22c55e" },
  { name: "Teal", value: "#14b8a6" },
  { name: "Cyan", value: "#06b6d4" },
];

// Status bar menus
const showLanguageMenu = ref(false);
const showColorMenu = ref(false);
const showThemeMenu = ref(false);
const showLLMMenu = ref(false);
const showGrobidMenu = ref(false);

// Load config on mount
onMounted(async () => {
  try {
    const data = await invokeCommand<any>("get_app_config");
    if (data?.system?.llm_providers) {
      appStore.setLLMProviders(data.system.llm_providers);
    }
    if (data?.paper?.grobid?.servers) {
      appStore.setGrobidServers(data.paper.grobid.servers);
    }
  } catch (error) {
    console.error("Failed to load config for status bar:", error);
  }
});

// Handle LLM Provider selection
async function handleLLMProviderSelect(providerId: string) {
  appStore.setSelectedLLMProvider(providerId);

  // Update default provider and save to backend
  const newProviders = appStore.llmProviders.map((p) => ({
    ...p,
    is_default: p.id === providerId,
  }));

  try {
    await invokeCommand("save_app_config", {
      config: {
        system: {
          llm_providers: newProviders,
        },
      },
    });
    // Update local store
    appStore.setLLMProviders(newProviders);
    showLLMMenu.value = false;
  } catch (error) {
    console.error("Failed to save LLM provider selection:", error);
  }
}

// Handle GROBID Server selection
async function handleGrobidServerSelect(serverId: string) {
  appStore.setSelectedGrobidServer(serverId);

  // Update default server and save to backend
  const newServers = appStore.grobidServers.map((s) => ({
    ...s,
    is_default: s.id === serverId,
  }));

  try {
    await invokeCommand("save_app_config", {
      config: {
        paper: {
          grobid: {
            servers: newServers,
          },
        },
      },
    });
    // Update local store
    appStore.setGrobidServers(newServers);
    showGrobidMenu.value = false;
  } catch (error) {
    console.error("Failed to save GROBID server selection:", error);
  }
}
</script>

<template>
  <v-footer height="36" class="status-bar">
    <div class="status-bar-left">
      <span class="mr-4">{{ t("status.documents") }}: 0</span>
      <span>{{ t("status.version") }}: 0.1.0</span>
    </div>

    <div class="status-bar-right">
      <!-- LLM Provider selector -->
      <v-menu
        v-model="showLLMMenu"
        location="top"
        :close-on-content-click="false"
      >
        <template #activator="{ props }">
          <v-btn
            v-bind="props"
            size="small"
            variant="text"
            class="status-bar-btn"
            :disabled="appStore.llmProviders.length === 0"
          >
            <v-icon size="small" class="mr-1">mdi-brain</v-icon>
            <span class="text-truncate" style="max-width: 100px">
              {{ appStore.currentLLMProvider?.name || t("status.noLLM") }}
            </span>
            <v-icon size="small" class="ml-1">mdi-chevron-up</v-icon>
          </v-btn>
        </template>
        <v-list density="compact">
          <v-list-item
            v-for="provider in appStore.llmProviders"
            :key="provider.id"
            @click="handleLLMProviderSelect(provider.id)"
            :active="appStore.selectedLLMProvider === provider.id"
          >
            <template #prepend>
              <v-icon size="small">mdi-brain</v-icon>
            </template>
            <v-list-item-title>{{ provider.name }}</v-list-item-title>
            <v-list-item-subtitle>{{
              provider.model_name
            }}</v-list-item-subtitle>
            <template #append v-if="provider.is_default">
              <v-chip size="x-small" color="success">{{
                t("settings.default")
              }}</v-chip>
            </template>
          </v-list-item>
          <v-list-item v-if="appStore.llmProviders.length === 0" disabled>
            <v-list-item-title class="text-grey">
              {{ t("status.noLLMConfigured") }}
            </v-list-item-title>
          </v-list-item>
        </v-list>
      </v-menu>

      <!-- GROBID Server selector -->
      <v-menu
        v-model="showGrobidMenu"
        location="top"
        :close-on-content-click="false"
      >
        <template #activator="{ props }">
          <v-btn
            v-bind="props"
            size="small"
            variant="text"
            class="status-bar-btn"
            :disabled="appStore.grobidServers.length === 0"
          >
            <v-icon size="small" class="mr-1">mdi-file-document-outline</v-icon>
            <span class="text-truncate" style="max-width: 100px">
              {{ appStore.currentGrobidServer?.name || t("status.noGrobid") }}
            </span>
            <v-icon size="small" class="ml-1">mdi-chevron-up</v-icon>
          </v-btn>
        </template>
        <v-list density="compact">
          <v-list-item
            v-for="server in appStore.grobidServers"
            :key="server.id"
            @click="handleGrobidServerSelect(server.id)"
            :active="appStore.selectedGrobidServer === server.id"
          >
            <template #prepend>
              <v-icon size="small">mdi-server</v-icon>
            </template>
            <v-list-item-title>{{ server.name }}</v-list-item-title>
            <v-list-item-subtitle
              class="text-truncate"
              style="max-width: 200px"
            >
              {{ server.url }}
            </v-list-item-subtitle>
            <template #append v-if="server.is_default">
              <v-chip size="x-small" color="success">{{
                t("settings.default")
              }}</v-chip>
            </template>
          </v-list-item>
          <v-list-item v-if="appStore.grobidServers.length === 0" disabled>
            <v-list-item-title class="text-grey">
              {{ t("status.noGrobidConfigured") }}
            </v-list-item-title>
          </v-list-item>
        </v-list>
      </v-menu>

      <!-- Language selector -->
      <v-menu
        v-model="showLanguageMenu"
        location="top"
        :close-on-content-click="false"
      >
        <template #activator="{ props }">
          <v-btn
            v-bind="props"
            size="small"
            variant="text"
            class="status-bar-btn"
          >
            <span class="status-bar-flag">{{
              availableLocales[currentLocale]?.flag || "🌐"
            }}</span>
            <v-icon size="small" class="ml-1">mdi-chevron-up</v-icon>
          </v-btn>
        </template>
        <v-list density="compact">
          <v-list-item
            v-for="(loc, code) in availableLocales"
            :key="code"
            @click="
              setLocale(code as any);
              showLanguageMenu = false;
            "
            :active="currentLocale === code"
          >
            <template #prepend>
              <span class="mr-2">{{ loc.flag }}</span>
            </template>
            <v-list-item-title>{{ loc.nativeName }}</v-list-item-title>
            <template #append v-if="currentLocale === code">
              <v-icon size="small" color="success">mdi-check</v-icon>
            </template>
          </v-list-item>
        </v-list>
      </v-menu>

      <!-- Color selector -->
      <v-menu
        v-model="showColorMenu"
        location="top"
        :close-on-content-click="false"
      >
        <template #activator="{ props }">
          <v-btn
            v-bind="props"
            size="small"
            variant="text"
            class="status-bar-btn"
          >
            <div
              class="color-dot"
              :style="{ backgroundColor: appStore.accentColor }"
            ></div>
            <v-icon size="small" class="ml-1">mdi-chevron-up</v-icon>
          </v-btn>
        </template>
        <v-list density="compact">
          <v-list-item
            v-for="color in accentColors"
            :key="color.value"
            @click="
              appStore.setAccentColor(color.value);
              showColorMenu = false;
            "
          >
            <template #prepend>
              <div
                class="color-dot"
                :style="{ backgroundColor: color.value }"
                :class="{
                  'color-dot-active': appStore.accentColor === color.value,
                }"
              ></div>
            </template>
            <v-list-item-title>{{ color.name }}</v-list-item-title>
            <template #append v-if="appStore.accentColor === color.value">
              <v-icon size="small" color="success">mdi-check</v-icon>
            </template>
          </v-list-item>
        </v-list>
      </v-menu>

      <!-- Theme selector -->
      <v-menu
        v-model="showThemeMenu"
        location="top"
        :close-on-content-click="false"
      >
        <template #activator="{ props }">
          <v-btn
            v-bind="props"
            size="small"
            variant="text"
            class="status-bar-btn"
          >
            <v-icon size="small">
              {{ appStore.isDark ? "mdi-weather-night" : "mdi-weather-sunny" }}
            </v-icon>
            <v-icon size="small" class="ml-1">mdi-chevron-up</v-icon>
          </v-btn>
        </template>
        <v-list density="compact">
          <v-list-item
            @click="
              appStore.setTheme(true);
              showThemeMenu = false;
            "
          >
            <template #prepend>
              <v-icon>mdi-weather-night</v-icon>
            </template>
            <v-list-item-title>{{ t("theme.dark") }}</v-list-item-title>
            <template #append v-if="appStore.isDark">
              <v-icon size="small" color="success">mdi-check</v-icon>
            </template>
          </v-list-item>
          <v-list-item
            @click="
              appStore.setTheme(false);
              showThemeMenu = false;
            "
          >
            <template #prepend>
              <v-icon>mdi-weather-sunny</v-icon>
            </template>
            <v-list-item-title>{{ t("theme.light") }}</v-list-item-title>
            <template #append v-if="!appStore.isDark">
              <v-icon size="small" color="success">mdi-check</v-icon>
            </template>
          </v-list-item>
        </v-list>
      </v-menu>
    </div>
  </v-footer>
</template>

<style scoped>
.status-bar {
  padding: 0 8px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.12);
}

.status-bar-left {
  display: flex;
  align-items: center;
}

.status-bar-right {
  display: flex;
  align-items: center;
  gap: 4px;
}

.status-bar-btn {
  min-width: auto;
  height: 28px;
  padding: 0 8px;
}

.status-bar-flag {
  font-size: 14px;
}

.color-dot {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.color-dot-active {
  border: 2px solid rgb(var(--v-theme-primary));
}
</style>
