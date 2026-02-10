<script setup lang="ts">
import { computed, ref, onMounted } from "vue";
import { useI18n, setLocale, getCurrentLocale } from "@/lib/i18n";
import { useAppStore } from "@/stores/useAppStore";

const { t } = useI18n();
const appStore = useAppStore();

// Theme settings
const isDark = computed({
  get: () => appStore.isDark,
  set: (value) => appStore.setTheme(value),
});

const accentColor = computed({
  get: () => appStore.accentColor,
  set: (value) => appStore.setAccentColor(value),
});

// Color themes
const colorThemes = [
  { name: "blue", value: "#3b82f6", color: "#3b82f6" },
  { name: "purple", value: "#8b5cf6", color: "#8b5cf6" },
  { name: "pink", value: "#ec4899", color: "#ec4899" },
  { name: "red", value: "#ef4444", color: "#ef4444" },
  { name: "orange", value: "#f97316", color: "#f97316" },
  { name: "green", value: "#22c55e", color: "#22c55e" },
  { name: "teal", value: "#14b8a6", color: "#14b8a6" },
  { name: "cyan", value: "#06b6d4", color: "#06b6d4" },
];

// Language settings
const availableLocales = [
  { code: "en", name: "English" },
  { code: "zh", name: "中文" },
];

// Get current locale from i18n instance
const currentLocale = ref(getCurrentLocale());

// Handle language change
function handleLanguageChange(localeCode: string) {
  currentLocale.value = localeCode;
  setLocale(localeCode);
}

// Handle theme toggle
function handleThemeToggle() {
  isDark.value = !isDark.value;
}

// Handle accent color change
function handleAccentColorChange(color: string) {
  accentColor.value = color;
}
</script>

<template>
  <v-card>
    <v-card-title>
      <v-icon start>mdi-palette</v-icon>
      {{ t("settings.appearance") }}
    </v-card-title>

    <v-card-text>
      <!-- Theme Mode -->
      <div class="setting-section">
        <div class="setting-label">
          <v-icon class="mr-2">mdi-theme-light-dark</v-icon>
          <span>{{ t("theme.themeSelector") }}</span>
        </div>
        <v-btn-toggle
          :model-value="isDark"
          mandatory
          color="primary"
          @update:model-value="handleThemeToggle"
        >
          <v-btn :value="false">
            <v-icon start>mdi-white-balance-sunny</v-icon>
            {{ t("theme.lightMode") }}
          </v-btn>
          <v-btn :value="true">
            <v-icon start>mdi-weather-night</v-icon>
            {{ t("theme.darkMode") }}
          </v-btn>
        </v-btn-toggle>
      </div>

      <v-divider class="my-4" />

      <!-- Accent Color -->
      <div class="setting-section">
        <div class="setting-label">
          <v-icon class="mr-2">mdi-palette</v-icon>
          <span>{{ t("theme.accentColor") }}</span>
        </div>
        <div class="color-themes">
          <div
            v-for="theme in colorThemes"
            :key="theme.name"
            class="color-theme-item"
            :class="{
              'color-theme-active': accentColor === theme.value,
            }"
            :style="{ backgroundColor: theme.color }"
            @click="handleAccentColorChange(theme.value)"
          >
            <v-icon
              v-if="accentColor === theme.value"
              color="white"
              size="small"
            >
              mdi-check
            </v-icon>
          </div>
        </div>
      </div>

      <v-divider class="my-4" />

      <!-- Language -->
      <div class="setting-section">
        <div class="setting-label">
          <v-icon class="mr-2">mdi-translate</v-icon>
          <span>{{ t("language.title") }}</span>
        </div>
        <v-select
          v-model="currentLocale"
          :items="availableLocales"
          item-title="name"
          item-value="code"
          variant="outlined"
          density="compact"
          hide-details
          @update:model-value="handleLanguageChange"
        />
      </div>
    </v-card-text>
  </v-card>
</template>

<style scoped>
.setting-section {
  padding: 8px 0;
}

.setting-label {
  display: flex;
  align-items: center;
  font-weight: 500;
  margin-bottom: 12px;
}

.color-themes {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}

.color-theme-item {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  border: 2px solid transparent;
  transition: all 0.2s;
}

.color-theme-item:hover {
  transform: scale(1.1);
}

.color-theme-active {
  border-color: rgb(var(--v-theme-on-surface-variant));
  box-shadow: 0 0 0 2px rgb(var(--v-theme-on-surface-variant));
}
</style>
