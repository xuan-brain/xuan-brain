<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from "vue";
import { useRoute } from "vue-router";
import { useI18n, setLocale } from "@/lib/i18n";
import { useAppStore } from "@/stores/useAppStore";
import GlobalSidebar from "@/components/layout/GlobalSidebar.vue";
import Navigation from "@/components/navigation/Navigation.vue";

const { t, locale: localeRef, availableLocales } = useI18n();
const appStore = useAppStore();

// Current locale is already a string when using legacy: false
const currentLocale = computed(
  () => localeRef.value as keyof typeof availableLocales,
);

// Determine if we should show left navigation drawer for papers page
const route = useRoute();
const isPapersPage = computed(() => route.path.startsWith("/papers"));

// Selected category path (for communication with PapersPage)
const selectedCategory = ref<string | null>(null);

// Handle category selection from navigation
function handleCategorySelect(path: string | null) {
  selectedCategory.value = path;
}

// Category drawer width management
const STORAGE_KEY = "main-layout-category-width";
const DEFAULT_WIDTH = 280;
const MIN_WIDTH = 200;
const MAX_WIDTH = 500;

const categoryDrawerWidth = ref(DEFAULT_WIDTH);
const isResizing = ref(false);
const startX = ref(0);
const startWidth = ref(0);

// Load saved width from localStorage
onMounted(() => {
  const saved = localStorage.getItem(STORAGE_KEY);
  if (saved) {
    const width = parseInt(saved, 10);
    if (!isNaN(width) && width >= MIN_WIDTH && width <= MAX_WIDTH) {
      categoryDrawerWidth.value = width;
    }
  }
});

// Save width to localStorage
function saveWidth(width: number) {
  localStorage.setItem(STORAGE_KEY, width.toString());
}

// Start resizing
function startResize(e: MouseEvent) {
  isResizing.value = true;
  startX.value = e.clientX;
  startWidth.value = categoryDrawerWidth.value;

  document.addEventListener("mousemove", onResize);
  document.addEventListener("mouseup", stopResize);

  e.preventDefault();
}

// Resize
function onResize(e: MouseEvent) {
  if (!isResizing.value) return;

  const deltaX = e.clientX - startX.value;
  const newWidth = startWidth.value + deltaX;

  // Constrain width
  const clampedWidth = Math.max(MIN_WIDTH, Math.min(MAX_WIDTH, newWidth));
  categoryDrawerWidth.value = clampedWidth;
}

// Stop resizing
function stopResize() {
  if (isResizing.value) {
    isResizing.value = false;
    saveWidth(categoryDrawerWidth.value);
  }
  document.removeEventListener("mousemove", onResize);
  document.removeEventListener("mouseup", stopResize);
}

// Cleanup
onUnmounted(() => {
  document.removeEventListener("mousemove", onResize);
  document.removeEventListener("mouseup", stopResize);
});

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
  <v-layout class="main-layout">
    <!-- Global sidebar (Rail mode) -->
    <GlobalSidebar />

    <!-- Left navigation drawer (categories and labels) - only on papers page -->
    <template v-if="isPapersPage">
      <div
        class="category-drawer-wrapper"
        :style="{ width: `${categoryDrawerWidth}px` }"
      >
        <div class="category-drawer-content">
          <Navigation @category-select="handleCategorySelect" />
        </div>

        <!-- Resize handle -->
        <div
          class="resize-handle"
          :class="{ resizing: isResizing }"
          @mousedown="startResize"
        >
          <div class="resize-handle-line"></div>
        </div>
      </div>
    </template>

    <!-- Main content area -->
    <v-main>
      <router-view :selected-category="selectedCategory" />
    </v-main>

    <!-- Bottom status bar -->
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
                {{
                  appStore.isDark ? "mdi-weather-night" : "mdi-weather-sunny"
                }}
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
  </v-layout>
</template>

<style scoped>
.main-layout {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

/* Ensure v-main takes available space but leaves room for footer */
.main-layout :deep(.v-main) {
  flex: 1;
  min-height: 0;
}

/* Ensure footer is visible at bottom */
.main-layout :deep(.v-footer) {
  flex-shrink: 0;
}

.category-drawer-wrapper {
  position: relative;
  height: 100%;
  display: flex;
  flex-shrink: 0;
}

.category-drawer-content {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  border-right: 1px solid rgba(255, 255, 255, 0.12);
}

/* Resize handle */
.resize-handle {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  width: 4px;
  cursor: col-resize;
  z-index: 100;
  background: transparent;
}

.resize-handle:hover,
.resize-handle.resizing {
  background: rgb(var(--v-theme-primary));
}

.resize-handle-line {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.resize-handle-line::before {
  content: "";
  width: 2px;
  height: 24px;
  background: rgba(255, 255, 255, 0.3);
  border-radius: 1px;
}

.resize-handle:hover .resize-handle-line::before,
.resize-handle.resizing .resize-handle-line::before {
  background: rgb(var(--v-theme-on-primary));
}

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

/* Disable transitions */
* {
  transition: none !important;
  animation-duration: 0s !important;
  animation-delay: 0s !important;
}
</style>
