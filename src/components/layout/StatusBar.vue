<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n, setLocale } from "@/lib/i18n";
import { useAppStore } from "@/stores/useAppStore";

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
</script>

<template>
  <v-footer height="36" class="status-bar">
    <div class="status-bar-left">
      <span class="mr-4">{{ t("status.documents") }}: 0</span>
      <span>{{ t("status.version") }}: 0.1.0</span>
    </div>

    <div class="status-bar-right">
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
