<script setup lang="ts">
  import type { ClippingResponse, Comment } from '@/lib/api/clips';
  import { getClip } from '@/lib/api/clips';
  import { useAppStore } from '@/stores/useAppStore';
  import DOMPurify from 'dompurify';
  import { marked } from 'marked';
  import { computed, ref, watch } from 'vue';
  import ClipComments from './ClipComments.vue';

  const appStore = useAppStore();

  interface Tag {
    id: string;
    name: string;
    color: string;
  }

  interface ClipDetail {
    id: string;
    title: string;
    url: string;
    source_domain: string;
    author?: string;
    published_date?: string;
    content: string; // Markdown content
    notes?: string;
    tags: Tag[];
    comments: Comment[];
    read_status: number; // 0 = unread, 1 = read
    created_at: string;
    updated_at: string;
    image_paths: string[];
  }

  interface Props {
    clipId?: string | null;
  }

  const props = defineProps<Props>();

  // Emit event for parent updates
  const emit = defineEmits<{
    clipUpdated: [clip: ClipDetail];
  }>();

  // State
  const details = ref<ClipDetail | null>(null);
  const loading = ref(false);
  const actionLoading = ref(false);

  // Configure marked for rendering
  marked.setOptions({
    breaks: true,
    gfm: true,
  });

  // Convert API response to ClipDetail format
  function apiToClipDetail(api: ClippingResponse): ClipDetail {
    return {
      id: api.id,
      title: api.title,
      url: api.url,
      source_domain: api.source_domain,
      author: api.author ?? undefined,
      published_date: api.published_date ?? undefined,
      content: api.content,
      notes: api.notes ?? undefined,
      tags: api.tags.map((tag, _) => ({
        id: `${api.id}-${tag}`,
        name: tag,
        color: 'primary',
      })),
      comments: api.comments,
      read_status: api.read_status,
      created_at: api.created_at,
      updated_at: api.updated_at,
      image_paths: api.image_paths,
    };
  }

  // Load clip details from API
  async function loadClipDetails() {
    if (!props.clipId) return;

    loading.value = true;
    try {
      console.info('Loading clip details for:', props.clipId);
      const apiData = await getClip(props.clipId);
      details.value = apiToClipDetail(apiData);
      console.info('Clip details loaded successfully');
    } catch (error) {
      console.error('Failed to load clip details:', error);
    } finally {
      loading.value = false;
    }
  }

  // Toggle read status
  async function toggleReadStatus() {
    if (!details.value) return;

    actionLoading.value = true;
    try {
      // TODO: Replace with actual API call
      // await updateClipReadStatus(details.value.id, details.value.read_status === 0 ? 1 : 0);

      // For now, just toggle locally
      details.value.read_status = details.value.read_status === 0 ? 1 : 0;

      if (details.value) {
        emit('clipUpdated', details.value);
      }
    } catch (error) {
      console.error('Failed to update read status:', error);
    } finally {
      actionLoading.value = false;
    }
  }

  // Render markdown content
  const renderedContent = computed(() => {
    if (!details.value) return '';

    try {
      let content = details.value.content;

      // Process image URLs to use the full API path
      content = content.replace(
        /!\[([^\]]*)\]\((\/clips\/images\/[^)]+)\)/g,
        (_match, alt, path) => {
          const fullUrl = `http://localhost:3030${path}`;
          return `![${alt}](${fullUrl})`;
        }
      );

      const rawHtml = marked.parse(content) as string;
      return DOMPurify.sanitize(rawHtml);
    } catch (error) {
      console.error('Failed to render markdown:', error);
      return '<p>Failed to render content</p>';
    }
  });

  // Format date for display
  function formatDate(dateString: string): string {
    try {
      const date = new Date(dateString);
      return date.toLocaleDateString(undefined, {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return dateString;
    }
  }

  // Notify parent of update
  function notifyUpdate(data: ClipDetail) {
    emit('clipUpdated', data);
  }

  // Handle comments updated from ClipComments component
  function handleCommentsUpdated(updatedComments: Comment[]) {
    if (details.value) {
      details.value = {
        ...details.value,
        comments: updatedComments,
      };
    }
  }

  // Watch clip ID changes
  watch(
    () => props.clipId,
    () => {
      if (props.clipId) {
        loadClipDetails();
      } else {
        details.value = null;
      }
    },
    { immediate: true }
  );
</script>

<template>
  <div class="clip-details">
    <!-- Loading state -->
    <div v-if="loading && !details" class="loading-container">
      <v-progress-circular indeterminate size="48" />
    </div>

    <!-- No clip selected - show blank -->
    <div v-else-if="!clipId" class="no-selection"></div>

    <!-- Details view -->
    <div v-else-if="details" class="details-view">
      <!-- Header -->
      <v-card class="clip-header" :theme="appStore.currentTheme" variant="flat">
        <v-card-item>
          <v-card-title class="clip-title">
            {{ details.title }}
          </v-card-title>
          <v-card-subtitle class="clip-url">
            <a :href="details.url" target="_blank" rel="noopener noreferrer" class="url-link">
              <v-icon start size="small">mdi-link</v-icon>
              {{ details.url }}
            </a>
          </v-card-subtitle>
        </v-card-item>

        <v-card-text>
          <!-- Metadata -->
          <div class="metadata-section">
            <div v-if="details.source_domain" class="metadata-item">
              <v-icon size="small" start>mdi-web</v-icon>
              <span class="text-caption">{{ details.source_domain }}</span>
            </div>
            <div v-if="details.author" class="metadata-item">
              <v-icon size="small" start>mdi-account</v-icon>
              <span class="text-caption">{{ details.author }}</span>
            </div>
            <div v-if="details.published_date" class="metadata-item">
              <v-icon size="small" start>mdi-calendar</v-icon>
              <span class="text-caption">{{ details.published_date }}</span>
            </div>
          </div>
        </v-card-text>
      </v-card>

      <!-- Body: Markdown Content -->
      <v-card class="clip-body" :theme="appStore.currentTheme" variant="flat">
        <v-card-text>
          <div
            class="markdown-content"
            :class="appStore.isDark ? 'markdown-dark' : 'markdown-light'"
            v-html="renderedContent"
          ></div>
        </v-card-text>
      </v-card>

      <!-- Sidebar: Notes, Tags, Read Status, Comments -->
      <v-card class="clip-sidebar" :theme="appStore.currentTheme" variant="flat">
        <v-card-text>
          <!-- Read Status Toggle -->
          <div class="sidebar-section">
            <div class="section-title">
              <v-icon size="small" start>mdi-eye-check</v-icon>
              Read Status
            </div>
            <v-switch
              v-model="details.read_status"
              :label="details.read_status === 1 ? 'Read' : 'Unread'"
              color="success"
              hide-details
              density="compact"
              :loading="actionLoading"
              @update:model-value="toggleReadStatus"
            />
          </div>

          <v-divider class="my-3" />

          <!-- Tags -->
          <div class="sidebar-section">
            <div class="section-title">
              <v-icon size="small" start>mdi-tag-multiple</v-icon>
              Tags
            </div>
            <div class="tags-container">
              <v-chip
                v-for="tag in details.tags"
                :key="tag.id"
                :color="tag.color"
                size="small"
                class="mr-1 mb-1"
              >
                {{ tag.name }}
              </v-chip>
              <v-chip v-if="details.tags.length === 0" size="small" color="grey" class="mr-1 mb-1">
                No tags
              </v-chip>
            </div>
          </div>

          <v-divider class="my-3" />

          <!-- Notes -->
          <div class="sidebar-section">
            <div class="section-title">
              <v-icon size="small" start>mdi-note-text</v-icon>
              Notes
            </div>
            <v-card variant="tonal" density="compact" class="notes-card">
              <p class="text-body-2 notes-text mb-0">
                {{ details.notes || 'No notes' }}
              </p>
            </v-card>
          </div>

          <v-divider class="my-3" />

          <!-- Comments -->
          <div class="sidebar-section">
            <ClipComments
              :clip-id="details.id"
              :comments="details.comments"
              @comments-updated="handleCommentsUpdated"
            />
          </div>
        </v-card-text>
      </v-card>

      <!-- Footer -->
      <v-card class="clip-footer" :theme="appStore.currentTheme" variant="flat">
        <v-card-text>
          <div class="footer-info">
            <span class="text-caption text-grey">
              Created: {{ formatDate(details.created_at) }}
            </span>
            <v-divider vertical class="mx-2" />
            <span class="text-caption text-grey">
              Updated: {{ formatDate(details.updated_at) }}
            </span>
          </div>
        </v-card-text>
      </v-card>
    </div>
  </div>
</template>

<style scoped>
  .clip-details {
    height: 100%;
    overflow-y: auto;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .loading-container {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .no-selection {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .details-view {
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .clip-header {
    flex-shrink: 0;
  }

  .clip-title {
    font-size: 18px;
    font-weight: 600;
    line-height: 1.4;
  }

  .clip-url {
    margin-top: 8px;
  }

  .url-link {
    color: inherit;
    text-decoration: none;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .url-link:hover {
    text-decoration: underline;
  }

  .metadata-section {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    margin-top: 8px;
  }

  .metadata-item {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .clip-body {
    flex: 1;
    min-height: 200px;
    overflow-y: auto;
  }

  .markdown-content {
    line-height: 1.6;
    font-size: 14px;
  }

  .markdown-content :deep(img) {
    max-width: 100%;
    height: auto;
    border-radius: 8px;
    margin: 16px 0;
  }

  .markdown-content :deep(h1),
  .markdown-content :deep(h2),
  .markdown-content :deep(h3),
  .markdown-content :deep(h4),
  .markdown-content :deep(h5),
  .markdown-content :deep(h6) {
    margin-top: 24px;
    margin-bottom: 12px;
    font-weight: 600;
  }

  .markdown-content :deep(h1) {
    font-size: 24px;
  }

  .markdown-content :deep(h2) {
    font-size: 20px;
  }

  .markdown-content :deep(h3) {
    font-size: 18px;
  }

  .markdown-content :deep(p) {
    margin-bottom: 12px;
  }

  .markdown-content :deep(code) {
    background-color: rgba(128, 128, 128, 0.2);
    padding: 2px 6px;
    border-radius: 4px;
    font-family: monospace;
    font-size: 0.9em;
  }

  .markdown-content :deep(pre) {
    background-color: rgba(128, 128, 128, 0.1);
    padding: 12px;
    border-radius: 8px;
    overflow-x: auto;
    margin: 12px 0;
  }

  .markdown-content :deep(pre code) {
    background-color: transparent;
    padding: 0;
  }

  .markdown-content :deep(blockquote) {
    border-left: 4px solid rgba(128, 128, 128, 0.5);
    padding-left: 16px;
    margin: 12px 0;
    color: rgba(128, 128, 128, 1);
  }

  .markdown-content :deep(ul),
  .markdown-content :deep(ol) {
    padding-left: 24px;
    margin-bottom: 12px;
  }

  .markdown-content :deep(li) {
    margin-bottom: 4px;
  }

  .markdown-content :deep(a) {
    color: rgb(var(--v-theme-primary));
    text-decoration: none;
  }

  .markdown-content :deep(a:hover) {
    text-decoration: underline;
  }

  .clip-sidebar {
    flex-shrink: 0;
  }

  .sidebar-section {
    margin-bottom: 16px;
  }

  .sidebar-section:last-child {
    margin-bottom: 0;
  }

  .section-title {
    font-weight: 500;
    font-size: 14px;
    margin-bottom: 8px;
    display: flex;
    align-items: center;
  }

  .tags-container {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .notes-card {
    padding: 12px;
  }

  .notes-text {
    white-space: pre-wrap;
    font-size: 13px;
  }

  .clip-footer {
    flex-shrink: 0;
  }

  .footer-info {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
  }
</style>
