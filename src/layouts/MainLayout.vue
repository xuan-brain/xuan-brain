<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from "vue";
import { useRoute } from "vue-router";
import GlobalSidebar from "@/components/layout/GlobalSidebar.vue";
import Navigation from "@/components/navigation/Navigation.vue";

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
  </v-layout>
</template>

<style scoped>
.main-layout {
  height: 100%;
  display: flex;
  flex-direction: column;
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
