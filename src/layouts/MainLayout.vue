<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from "vue";
import { useRouter, useRoute } from "vue-router";
import { useI18n } from "@/lib/i18n";
import Navigation from "@/components/navigation/Navigation.vue";

const router = useRouter();
const route = useRoute();
const { t } = useI18n();

// Determine if we should show left navigation drawer for papers page
const isPapersPage = computed(() => route.path.startsWith("/papers"));

// Selected category path (for communication with PapersPage)
const selectedCategory = ref<string | null>(null);

// Current view state (library, favorites, trash)
const currentView = ref<"library" | "favorites" | "trash">("library");

// Handle category selection from navigation
function handleCategorySelect(path: string | null) {
  selectedCategory.value = path;
}

// Handle view change from navigation
function handleViewChange(view: "library" | "favorites" | "trash") {
  currentView.value = view;
  // Clear category selection when switching views
  if (view !== "library") {
    selectedCategory.value = null;
  }
}

// Navigation menu items
const menuItems = [
  { icon: "mdi-file-document", value: "papers", title: "navigation.papers" },
  { icon: "mdi-content-cut", value: "clips", title: "navigation.clips" },
  { icon: "mdi-pencil", value: "writing", title: "navigation.writing" },
  {
    icon: "mdi-rss",
    value: "subscriptions",
    title: "navigation.subscriptions",
  },
];

// Current route
const currentRoute = computed(() => route.path);

// Navigate to route
function navigateTo(path: string) {
  router.push(path);
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
</script>

<template>
  <div class="main-layout">
    <!-- Global sidebar (Rail mode) -->
    <div class="global-sidebar">
      <div class="sidebar-content">
        <!-- User avatar placeholder -->
        <div class="user-avatar">
          <v-avatar color="primary" size="40">
            <span class="text-h6">U</span>
          </v-avatar>
        </div>

        <v-divider class="my-2" />

        <!-- Navigation menu -->
        <v-list density="compact">
          <v-list-item
            v-for="item in menuItems"
            :key="item.value"
            :prepend-icon="item.icon"
            :value="item.value"
            :title="t(item.title)"
            rounded="lg"
            :active="currentRoute === `/${item.value}`"
            @click="navigateTo(`/${item.value}`)"
          />
        </v-list>

        <v-spacer />

        <!-- Settings at bottom -->
        <v-list density="compact">
          <v-list-item
            prepend-icon="mdi-cog"
            value="settings"
            :title="t('navigation.settings')"
            rounded="lg"
            :active="currentRoute === '/settings'"
            @click="navigateTo('/settings')"
          />
        </v-list>
      </div>
    </div>

    <!-- Left navigation drawer (categories and labels) - only on papers page -->
    <template v-if="isPapersPage">
      <div
        class="category-drawer-wrapper"
        :style="{ width: `${categoryDrawerWidth}px` }"
      >
        <div class="category-drawer-content">
          <Navigation
            @category-select="handleCategorySelect"
            @view-change="handleViewChange"
          />
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
    <div class="main-content">
      <router-view
        :selected-category="selectedCategory"
        :current-view="currentView"
      />
    </div>
  </div>
</template>

<style scoped>
.main-layout {
  display: flex;
  height: calc(100vh - 36px);
}

.global-sidebar {
  width: 72px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  border-right: 1px solid rgba(255, 255, 255, 0.12);
}

.sidebar-content {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 8px;
}

.user-avatar {
  display: flex;
  justify-content: center;
  padding: 8px 0;
}

.main-content {
  flex: 1;
  overflow: hidden;
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
</style>
