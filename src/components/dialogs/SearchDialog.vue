<script setup lang="ts">
  import { useI18n } from '@/lib/i18n';
  import { invokeCommand } from '@/lib/tauri';
  import { ref, watch, computed } from 'vue';

  interface SearchResult {
    id: string;
    title: string;
    abstract_text: string | null;
    doi: string | null;
    publication_year: number | null;
    journal_name: string | null;
    score: number;
    matched_labels: string[];
    matched_attachments: string[];
  }

  interface Props {
    modelValue: boolean;
  }

  const props = defineProps<Props>();
  const emit = defineEmits<{
    'update:modelValue': [value: boolean];
    'paperSelect': [paperId: string];
  }>();

  const { t } = useI18n();

  const isOpen = computed({
    get: () => props.modelValue,
    set: (value) => emit('update:modelValue', value),
  });

  const searchQuery = ref('');
  const results = ref<SearchResult[]>([]);
  const loading = ref(false);
  const searched = ref(false);
  const error = ref('');
  const indexing = ref(false);
  const indexEmpty = ref(false);

  // Check FTS index status when dialog opens
  async function checkFtsIndex() {
    try {
      const count = await invokeCommand<number>('check_fts_index_status');
      indexEmpty.value = count === 0;
      if (indexEmpty.value) {
        console.warn('FTS index is empty, search may not work properly');
      }
    } catch (err) {
      console.error('Failed to check FTS index status:', err);
    }
  }

  // Rebuild FTS index
  async function rebuildIndex() {
    indexing.value = true;
    error.value = '';
    try {
      await invokeCommand('rebuild_search_index');
      console.info('FTS index rebuilt successfully');
      indexEmpty.value = false;
    } catch (err) {
      console.error('Failed to rebuild index:', err);
      error.value = String(err);
    } finally {
      indexing.value = false;
    }
  }

  async function handleSearch() {
    const query = searchQuery.value.trim();
    if (!query) {
      return;
    }

    loading.value = true;
    error.value = '';
    searched.value = true;

    try {
      const data = await invokeCommand<SearchResult[]>('search_papers_fts', {
        query,
        limit: 100,
      });
      results.value = data;
      console.info(`FTS search found ${data.length} results for: ${query}`);

      // If no results and index might be empty, show hint
      if (data.length === 0 && indexEmpty.value) {
        error.value = t('search.indexEmptyHint');
      }
    } catch (err) {
      console.error('Search failed:', err);
      error.value = String(err);
      results.value = [];
    } finally {
      loading.value = false;
    }
  }

  function handleClose() {
    isOpen.value = false;
    searchQuery.value = '';
    results.value = [];
    searched.value = false;
    error.value = '';
  }

  function handleResultClick(row: SearchResult) {
    emit('paperSelect', row.id);
  }

  function getScoreColor(score: number): string {
    if (score >= 80) return 'success';
    if (score >= 60) return 'info';
    if (score >= 40) return 'warning';
    return 'default';
  }

  function highlightMatch(text: string | null | undefined, query: string): string {
    if (!text || !query) return text || '';
    const regex = new RegExp(`(${query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi');
    return text.replace(regex, '<mark>$1</mark>');
  }

  // Auto-search on Enter key
  function handleKeyPress(event: KeyboardEvent) {
    if (event.key === 'Enter' && !loading.value) {
      handleSearch();
    }
  }

  // Watch for dialog open to focus input and check index
  watch(isOpen, (newValue) => {
    if (newValue) {
      // Reset state when dialog opens
      searchQuery.value = '';
      results.value = [];
      searched.value = false;
      error.value = '';
      // Check FTS index status
      checkFtsIndex();
    }
  });
</script>

<template>
  <v-dialog v-model="isOpen" fullscreen :scrim="false" transition="dialog-bottom-transition">
    <v-card class="search-dialog">
      <!-- Search Header -->
      <v-card-title class="search-header pa-4">
        <div class="d-flex align-center">
          <v-icon start>mdi-magnify</v-icon>
          <span class="text-h5">{{ t('search.title') }}</span>
        </div>
        <v-spacer />
        <v-btn
          variant="tonal"
          :loading="indexing"
          @click="rebuildIndex"
          class="mr-2"
        >
          <v-icon start>mdi-database-refresh</v-icon>
          {{ t('search.rebuildIndex') }}
        </v-btn>
        <v-btn icon="mdi-close" @click="handleClose" />
      </v-card-title>

      <v-divider />

      <!-- Search Input Section -->
      <v-card-text class="search-input-section pa-4">
        <v-text-field
          v-model="searchQuery"
          :label="t('search.placeholder')"
          prepend-inner-icon="mdi-magnify"
          variant="outlined"
          hide-details
          clearable
          autofocus
          @keyup="handleKeyPress"
        >
          <template #append-inner>
            <v-chip v-if="results.length > 0 && !indexing" size="small" color="primary">
              {{ results.length }} {{ t('search.results') }}
            </v-chip>
            <v-chip v-else-if="indexing" size="small" color="info">
              {{ t('search.indexing') }}
            </v-chip>
          </template>
        </v-text-field>

        <v-alert v-if="indexEmpty" type="warning" variant="tonal" class="mt-3" closable>
          <template #prepend>
            <v-icon>mdi-alert-circle</v-icon>
          </template>
          {{ t('search.indexEmptyWarning') }}
        </v-alert>
        <v-alert v-if="error" type="error" variant="tonal" class="mt-3" :text="error" closable />
      </v-card-text>

      <v-divider />

      <!-- Search Results -->
      <v-card-text class="search-results pa-0">
        <!-- Loading State -->
        <div v-if="loading" class="d-flex justify-center align-center pa-8">
          <v-progress-circular indeterminate color="primary" size="48" />
          <span class="ml-4">{{ t('search.searching') }}</span>
        </div>

        <!-- No Results State -->
        <div
          v-else-if="searched && results.length === 0"
          class="d-flex flex-column justify-center align-center pa-8"
        >
          <v-icon size="80" color="grey">mdi-file-search-outline</v-icon>
          <div class="text-h6 mt-4 text-grey">{{ t('search.noResults') }}</div>
        </div>

        <!-- Results Table -->
        <vxe-table
          v-else-if="results.length > 0"
          :data="results"
          :row-config="{ isHover: true, keyField: 'id' }"
          @cell-click="({ row }) => handleResultClick(row)"
          class="search-table"
        >
          <!-- Score Column -->
          <vxe-column width="90" field="score" :title="t('search.score')" fixed="left">
            <template #default="{ row }">
              <v-chip :color="getScoreColor(row.score)" size="small" label>
                {{ row.score.toFixed(0) }}%
              </v-chip>
            </template>
          </vxe-column>

          <!-- Title Column -->
          <vxe-column min-width="300" field="title" :title="t('document.title')">
            <template #default="{ row }">
              <div class="result-title">
                <span v-html="highlightMatch(row.title, searchQuery)" />
              </div>
            </template>
          </vxe-column>

          <!-- Abstract Column -->
          <vxe-column min-width="350" field="abstract_text" :title="t('details.title')">
            <template #default="{ row }">
              <div class="text-body-2 text-truncate" style="max-width: 350px">
                <span v-html="highlightMatch(row.abstract_text, searchQuery)" />
              </div>
            </template>
          </vxe-column>

          <!-- Year Column -->
          <vxe-column width="100" field="publication_year" :title="t('document.year')">
            <template #default="{ row }">
              {{ row.publication_year || '-' }}
            </template>
          </vxe-column>

          <!-- Journal Column -->
          <vxe-column min-width="200" field="journal_name" :title="t('document.source')">
            <template #default="{ row }">
              {{ row.journal_name || '-' }}
            </template>
          </vxe-column>
        </vxe-table>

        <!-- Initial State -->
        <div
          v-else
          class="d-flex flex-column justify-center align-center pa-8"
          style="height: 400px"
        >
          <v-icon size="80" color="grey-lighten-1">mdi-magnify</v-icon>
          <div class="text-h6 mt-4 text-grey">{{ t('search.placeholder') }}</div>
        </div>
      </v-card-text>
    </v-card>
  </v-dialog>
</template>

<style scoped>
  .search-dialog {
    height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .search-header {
    flex-shrink: 0;
  }

  .search-input-section {
    flex-shrink: 0;
    background-color: rgb(var(--v-theme-surface));
  }

  .search-results {
    flex: 1;
    overflow: auto;
    background-color: rgb(var(--v-theme-background));
  }

  .search-table {
    width: 100%;
  }

  .search-table :deep(.vxe-body--row) {
    cursor: pointer;
  }

  .search-table :deep(.vxe-body--row:hover) {
    background-color: rgb(var(--v-theme-surface-variant));
  }

  .result-title {
    font-weight: 500;
    line-height: 1.4;
  }

  .result-title :deep(mark) {
    background-color: rgb(var(--v-theme-primary));
    color: rgb(var(--v-theme-on-primary));
    padding: 0 2px;
    border-radius: 2px;
  }
</style>
