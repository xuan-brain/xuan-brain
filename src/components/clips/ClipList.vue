<script setup lang="ts">
  import type { ClippingResponse } from '@/lib/api/clips';
  import { useAppStore } from '@/stores/useAppStore';
  import { onMounted, ref } from 'vue';

  interface Props {
    clippings?: ClippingResponse[];
    selectedId?: string | null;
  }

  const props = withDefaults(defineProps<Props>(), {
    clippings: () => [],
    selectedId: null,
  });

  const emit = defineEmits<{
    select: [clippingId: string];
  }>();

  const appStore = useAppStore();
  const loading = ref(false);

  // Format relative time for display
  function formatRelativeTime(dateString: string): string {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffSecs = Math.floor(diffMs / 1000);
    const diffMins = Math.floor(diffSecs / 60);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);

    if (diffSecs < 60) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  }

  // Truncate excerpt to ~100 characters
  function truncateExcerpt(excerpt: string | null, maxLength = 100): string {
    if (!excerpt) return '';
    if (excerpt.length <= maxLength) return excerpt;
    return excerpt.substring(0, maxLength).trim() + '...';
  }

  // Handle click on clipping item
  function handleClippingClick(clipping: ClippingResponse) {
    emit('select', clipping.id);
  }

  // Get read status icon
  function getReadStatusIcon(readStatus: number): string {
    return readStatus === 1 ? 'mdi-check-circle' : 'mdi-eye-circle-outline';
  }

  // Get read status color
  function getReadStatusColor(readStatus: number): string {
    return readStatus === 1 ? 'success' : 'grey';
  }

  // Load function for parent component refresh
  function loadClippings() {
    // This will be implemented when API is available
    // For now, the component accepts clippings as props
  }

  onMounted(() => {
    loadClippings();
  });

  defineExpose({
    loadClippings,
  });
</script>

<template>
  <div class="clip-list">
    <!-- Loading state -->
    <div v-if="loading" class="loading-container">
      <v-progress-circular indeterminate size="48" />
    </div>

    <!-- Empty state -->
    <div v-else-if="clippings.length === 0" class="empty-state">
      <v-icon size="48" color="grey">mdi-bookmark-outline</v-icon>
      <p class="text-body-2 text-grey mt-4">No clips yet</p>
      <p class="text-caption text-grey">Save web pages to get started</p>
    </div>

    <!-- Clipping list -->
    <v-list v-else class="clip-list-items">
      <v-list-item
        v-for="clipping in clippings"
        :key="clipping.id"
        :class="{ 'clip-item-selected': selectedId === clipping.id }"
        class="clip-item"
        @click="handleClippingClick(clipping)"
      >
        <!-- Title -->
        <v-list-item-title class="clip-title">
          {{ clipping.title }}
        </v-list-item-title>

        <!-- Source domain and read status -->
        <template #subtitle>
          <div class="clip-subtitle">
            <v-icon size="x-small" class="mr-1">mdi-web</v-icon>
            <span class="text-caption">{{ clipping.source_domain }}</span>
            <v-spacer />
            <v-icon
              :icon="getReadStatusIcon(clipping.readStatus)"
              :color="getReadStatusColor(clipping.readStatus)"
              size="x-small"
            />
          </div>
        </template>

        <!-- Excerpt -->
        <div class="clip-excerpt">
          {{ truncateExcerpt(clipping.excerpt) }}
        </div>

        <!-- Created at -->
        <div class="clip-meta">
          <v-icon size="x-small" class="mr-1">mdi-clock-outline</v-icon>
          <span class="text-caption text-grey">{{ formatRelativeTime(clipping.created_at) }}</span>
        </div>
      </v-list-item>
    </v-list>
  </div>
</template>

<style scoped>
  .clip-list {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .loading-container {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 20px;
    text-align: center;
  }

  .clip-list-items {
    flex: 1;
    overflow-y: auto;
    background-color: transparent;
    padding: 4px 0;
  }

  .clip-item {
    cursor: pointer;
    border-radius: 8px;
    margin: 4px 8px;
    padding: 12px 16px !important;
    min-height: auto !important;
    transition:
      background-color 0.2s ease,
      box-shadow 0.2s ease;
  }

  .clip-item:hover {
    background-color: rgba(255, 255, 255, 0.08);
  }

  .clip-item-selected {
    background-color: rgb(var(--v-theme-primary));
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  }

  .clip-item-selected .clip-title,
  .clip-item-selected .clip-excerpt,
  .clip-item-selected .clip-meta {
    color: rgb(var(--v-theme-on-primary));
  }

  .clip-title {
    font-weight: 600;
    font-size: 14px;
    margin-bottom: 4px;
  }

  .clip-subtitle {
    display: flex;
    align-items: center;
    margin-bottom: 6px;
  }

  .clip-excerpt {
    font-size: 12px;
    opacity: 0.8;
    line-height: 1.4;
    margin-bottom: 8px;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .clip-meta {
    display: flex;
    align-items: center;
  }

  /* Custom scrollbar for list */
  .clip-list-items::-webkit-scrollbar {
    width: 6px;
  }

  .clip-list-items::-webkit-scrollbar-track {
    background: transparent;
  }

  .clip-list-items::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.2);
    border-radius: 3px;
  }

  .clip-list-items::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.3);
  }

  /* Dark theme adjustments */
  :deep(.v-theme--dark) .clip-item:hover {
    background-color: rgba(255, 255, 255, 0.08);
  }

  :deep(.v-theme--light) .clip-item:hover {
    background-color: rgba(0, 0, 0, 0.04);
  }

  :deep(.v-theme--light) .clip-item-selected {
    background-color: rgb(var(--v-theme-primary));
  }
</style>
