<script setup lang="ts">
  import ClipDetails from '@/components/clips/ClipDetails.vue';
  import ClipList from '@/components/clips/ClipList.vue';
  import type { ClippingResponse } from '@/lib/api/clips';
  import { listClips } from '@/lib/api/clips';
  import { computed, onMounted, onUnmounted, ref } from 'vue';

  // Panel widths (in percentage)
  const STORAGE_KEY = 'clips-page-panel-widths';
  const defaultWidths = { left: 30, right: 70 };

  const panelWidths = ref({ ...defaultWidths });

  // Load saved widths from localStorage
  onMounted(() => {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      try {
        const parsed = JSON.parse(saved);
        // Validate and apply saved widths
        if (parsed && typeof parsed === 'object') {
          const total = (parsed.left || 0) + (parsed.right || 0);
          if (total === 100) {
            panelWidths.value = parsed;
          }
        }
      } catch (e) {
        console.error('Failed to parse panel widths:', e);
      }
    }

    // Load clippings from API
    loadClippings();
  });

  // Save widths to localStorage
  function saveWidths() {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(panelWidths.value));
  }

  // Dragging state
  const isDragging = ref(false);
  const startX = ref(0);
  const startWidths = ref({ left: 0, right: 0 });

  // Clippings state
  const clippings = ref<ClippingResponse[]>([]);
  const loading = ref(false);

  // Calculate panel styles
  const leftPanelStyle = computed(() => ({
    width: `${panelWidths.value.left}%`,
    minWidth: '20%',
    maxWidth: '80%',
  }));

  const rightPanelStyle = computed(() => ({
    width: `${panelWidths.value.right}%`,
    minWidth: '20%',
    maxWidth: '80%',
  }));

  // Divider style
  const dividerStyle = computed(() => ({
    left: `${panelWidths.value.left}%`,
  }));

  // Start dragging divider
  function startDrag(e: MouseEvent) {
    isDragging.value = true;
    startX.value = e.clientX;
    startWidths.value = { ...panelWidths.value };

    document.addEventListener('mousemove', onDrag);
    document.addEventListener('mouseup', stopDrag);

    e.preventDefault();
  }

  // Drag divider
  function onDrag(e: MouseEvent) {
    if (!isDragging.value) return;

    const containerWidth =
      (e.target as HTMLElement).parentElement?.offsetWidth || window.innerWidth;
    const deltaX = e.clientX - startX.value;
    const deltaPercent = (deltaX / containerWidth) * 100;

    // Calculate new widths
    let newLeft = startWidths.value.left + deltaPercent;
    let newRight = startWidths.value.right - deltaPercent;

    // Constrain widths (min 20%, max 80%)
    newLeft = Math.max(20, Math.min(80, newLeft));
    newRight = Math.max(20, Math.min(80, newRight));

    panelWidths.value = { left: newLeft, right: newRight };
  }

  // Stop dragging divider
  function stopDrag() {
    if (isDragging.value) {
      isDragging.value = false;
      saveWidths();
    }
    document.removeEventListener('mousemove', onDrag);
    document.removeEventListener('mouseup', stopDrag);
  }

  // State
  const selectedClipId = ref<string | null>(null);

  // Load clippings from API
  async function loadClippings() {
    loading.value = true;
    try {
      clippings.value = await listClips();
    } catch (error) {
      console.error('Failed to load clippings:', error);
    } finally {
      loading.value = false;
    }
  }

  // Handle clip selection from clip list
  function handleClipSelect(clipId: string) {
    selectedClipId.value = clipId;
  }

  // Cleanup event listeners on unmount
  onUnmounted(() => {
    document.removeEventListener('mousemove', onDrag);
    document.removeEventListener('mouseup', stopDrag);
  });
</script>

<template>
  <div class="clips-page">
    <!-- Two Panel Layout -->
    <div class="panels-container">
      <!-- Left Panel: Clip List -->
      <div class="panel left-panel" :style="leftPanelStyle">
        <div class="panel-content scrollable">
          <ClipList
            :clippings="clippings"
            :selected-id="selectedClipId"
            @select="handleClipSelect"
          />
        </div>
      </div>

      <!-- Divider (Drag Handle) -->
      <div
        class="divider"
        :class="{ dragging: isDragging }"
        :style="dividerStyle"
        @mousedown="startDrag"
      >
        <div class="divider-handle"></div>
      </div>

      <!-- Right Panel: Clip Details -->
      <div class="panel right-panel" :style="rightPanelStyle">
        <div class="panel-content scrollable">
          <ClipDetails :clip-id="selectedClipId" />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
  .clips-page {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* Leave space for status bar (36px) */
  .panels-container {
    display: flex;
    flex: 1;
    min-height: 0;
    position: relative;
  }

  .panel {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    flex-shrink: 0;
  }

  .panel-content {
    flex: 1;
    overflow: hidden;
  }

  .panel-content.scrollable {
    overflow-y: auto;
    overflow-x: hidden;
  }

  /* Divider (Drag Handle) */
  .divider {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 4px;
    cursor: col-resize;
    z-index: 10;
    background: transparent;
  }

  .divider:hover,
  .divider.dragging {
    background: rgb(var(--v-theme-primary));
  }

  .divider-handle {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .divider-handle::before {
    content: '';
    width: 2px;
    height: 24px;
    background: rgba(255, 255, 255, 0.3);
    border-radius: 1px;
  }

  .divider:hover .divider-handle::before,
  .divider.dragging .divider-handle::before {
    background: rgb(var(--v-theme-on-primary));
  }

  /* Panel borders */
  .left-panel {
    border-right: 1px solid rgba(255, 255, 255, 0.12);
  }

  /* Disable Vuetify transitions in this component */
  * {
    transition: none !important;
    animation-duration: 0s !important;
    animation-delay: 0s !important;
  }
</style>
