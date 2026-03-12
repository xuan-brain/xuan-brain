<script setup lang="ts">
  import { useI18n } from '@/lib/i18n';
  import { onUnmounted, ref, watch } from 'vue';

  interface Props {
    modelValue: boolean;
  }

  const props = defineProps<Props>();

  const emit = defineEmits<{
    'update:modelValue': [value: boolean];
    success: [];
  }>();

  const { t } = useI18n();

  interface BatchImportResult {
    total: number;
    imported: number;
    skipped: number;
    failed: number;
    papers: Array<{
      id: string;
      title: string;
      authors: string[];
    }>;
    errors: string[];
  }

  interface ImportProgress {
    current: number;
    total: number;
    current_title: string;
    status: string;
  }

  // State
  const selectedFile = ref<string | null>(null);
  const loading = ref(false);
  const error = ref('');
  const importResult = ref<BatchImportResult | null>(null);
  const showErrors = ref(false);
  const showPapers = ref(false);

  // Progress state
  const importProgress = ref<ImportProgress | null>(null);
  const progressPercent = ref(0);

  // Unlisten function for progress event
  let unlistenProgress: (() => void) | null = null;

  // Reset form when dialog opens
  watch(
    () => props.modelValue,
    async (isOpen) => {
      if (isOpen) {
        selectedFile.value = null;
        error.value = '';
        importResult.value = null;
        showErrors.value = false;
        showPapers.value = false;
        importProgress.value = null;
        progressPercent.value = 0;

        // Listen for import progress events
        const { listen } = await import('@tauri-apps/api/event');
        unlistenProgress = await listen<ImportProgress>('zotero:import-progress', (event) => {
          importProgress.value = event.payload;
          if (event.payload.total > 0) {
            progressPercent.value = Math.round((event.payload.current / event.payload.total) * 100);
          }
          console.info('Import progress:', event.payload);
        });
      } else {
        // Cleanup listener when dialog closes
        if (unlistenProgress) {
          unlistenProgress();
          unlistenProgress = null;
        }
      }
    }
  );

  // Cleanup on unmount
  onUnmounted(() => {
    if (unlistenProgress) {
      unlistenProgress();
      unlistenProgress = null;
    }
  });

  // Close dialog
  function handleClose() {
    selectedFile.value = null;
    error.value = '';
    importResult.value = null;
    showErrors.value = false;
    showPapers.value = false;
    importProgress.value = null;
    progressPercent.value = 0;
    emit('update:modelValue', false);
  }

  // Open file picker
  async function selectFile() {
    try {
      // Use Tauri file dialog
      const { open } = await import('@tauri-apps/plugin-dialog');

      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'RDF',
            extensions: ['rdf'],
          },
        ],
      });

      if (selected && typeof selected === 'string') {
        selectedFile.value = selected;
        error.value = '';
        importResult.value = null;
      }
    } catch (err) {
      console.error('Failed to open file dialog:', err);
      error.value = 'Failed to open file selector';
    }
  }

  // Submit form
  async function handleSubmit() {
    if (!selectedFile.value) {
      error.value = 'Please select an RDF file';
      return;
    }

    loading.value = true;
    error.value = '';
    importResult.value = null;
    importProgress.value = null;
    progressPercent.value = 0;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke<BatchImportResult>('import_papers_from_zotero_rdf', {
        filePath: selectedFile.value,
        categoryId: null,
      });

      console.info('Zotero RDF import completed:', result);
      importResult.value = result;

      if (result.imported > 0) {
        emit('success');
      }
    } catch (err) {
      // Handle different error formats from Tauri
      if (typeof err === 'string') {
        error.value = err;
      } else if (err && typeof err === 'object') {
        // Tauri errors often have a message property
        const errorObj = err as Record<string, unknown>;
        error.value = errorObj.message || errorObj.errorMessage || JSON.stringify(err);
      } else {
        error.value = String(err);
      }
    } finally {
      loading.value = false;
      importProgress.value = null;
    }
  }
</script>

<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="emit('update:modelValue', $event)"
    max-width="700"
  >
    <v-card>
      <v-card-title>
        <v-icon start>mdi-database-import</v-icon>
        {{ t('zotero.title') }}
      </v-card-title>

      <v-card-text>
        <v-alert
          v-if="error"
          type="error"
          :text="error"
          class="mb-4"
          closable
          @click:close="error = ''"
        />

        <!-- Import result -->
        <template v-if="importResult">
          <!-- Statistics Cards -->
          <div class="d-flex mb-4 gap-3">
            <v-card class="flex-1-0 stat-card stat-card-success" variant="outlined">
              <v-card-text class="pa-3 text-center">
                <div class="text-h4 font-weight-bold text-success">{{ importResult.imported }}</div>
                <div class="text-caption">{{ t('zotero.imported') }}</div>
              </v-card-text>
            </v-card>
            <v-card class="flex-1-0 stat-card stat-card-warning" variant="outlined">
              <v-card-text class="pa-3 text-center">
                <div class="text-h4 font-weight-bold text-warning">{{ importResult.skipped }}</div>
                <div class="text-caption">{{ t('zotero.skipped') }}</div>
              </v-card-text>
            </v-card>
            <v-card class="flex-1-0 stat-card stat-card-error" variant="outlined">
              <v-card-text class="pa-3 text-center">
                <div class="text-h4 font-weight-bold text-error">{{ importResult.failed }}</div>
                <div class="text-caption">{{ t('zotero.failed') }}</div>
              </v-card-text>
            </v-card>
          </div>

          <!-- Errors Section -->
          <v-card v-if="importResult.errors.length > 0" variant="outlined" class="mb-4">
            <v-card-title class="cursor-pointer px-4 py-2" @click="showErrors = !showErrors">
              <v-icon start :class="{ 'rotate-180': showErrors }">mdi-chevron-down</v-icon>
              <span class="text-subtitle-1">
                {{ t('zotero.errors') }} ({{ importResult.errors.length }})
              </span>
            </v-card-title>
            <v-card-text v-if="showErrors" class="pt-0">
              <div class="error-list">
                <div
                  v-for="(err, i) in importResult.errors"
                  :key="i"
                  class="text-caption error-item py-1"
                >
                  <v-icon size="small" color="error" class="mr-1">mdi-alert-circle-outline</v-icon>
                  {{ err }}
                </div>
              </div>
            </v-card-text>
          </v-card>

          <!-- Imported Papers Section -->
          <v-card v-if="importResult.papers.length > 0" variant="outlined">
            <v-card-title class="cursor-pointer px-4 py-2" @click="showPapers = !showPapers">
              <v-icon start :class="{ 'rotate-180': showPapers }">mdi-chevron-down</v-icon>
              <span class="text-subtitle-1">
                {{ t('zotero.importedPapers') }} ({{ importResult.papers.length }})
              </span>
            </v-card-title>
            <v-card-text v-if="showPapers" class="pt-0">
              <div class="paper-list">
                <div v-for="paper in importResult.papers" :key="paper.id" class="paper-item py-2">
                  <div class="text-body-2 font-weight-medium">{{ paper.title }}</div>
                  <div v-if="paper.authors.length > 0" class="text-caption text-medium-emphasis">
                    {{ paper.authors.join(', ') }}
                  </div>
                </div>
              </div>
            </v-card-text>
          </v-card>
        </template>

        <!-- File selection area -->
        <div v-if="!importResult && !loading" class="file-drop-zone">
          <v-icon size="64" color="primary">mdi-file-xml-box</v-icon>
          <div class="text-h6 mt-4">{{ t('zotero.selectRdfFile') }}</div>
          <div class="text-caption text-grey mt-2">
            {{ selectedFile ? selectedFile.split(/[\\/]/).pop() : t('zotero.clickToBrowse') }}
          </div>

          <v-btn color="primary" class="mt-4" @click="selectFile" :disabled="loading">
            <v-icon start>mdi-folder-open</v-icon>
            {{ t('zotero.browseFiles') }}
          </v-btn>
        </div>

        <!-- Progress area during import -->
        <div v-if="loading && importProgress" class="progress-area">
          <div class="d-flex align-center mb-4 justify-center">
            <v-progress-circular
              :model-value="progressPercent"
              :size="120"
              :width="8"
              color="primary"
            >
              <template #default>
                <span class="text-h5">{{ progressPercent }}%</span>
              </template>
            </v-progress-circular>
          </div>

          <div class="text-subtitle-1 mb-2 text-center">
            {{ t('zotero.importing') }} {{ importProgress.current }} / {{ importProgress.total }}
          </div>

          <div class="text-caption text-grey text-truncate px-4 text-center">
            {{ importProgress.current_title || t('zotero.processing') }}
          </div>

          <v-progress-linear
            :model-value="progressPercent"
            color="primary"
            class="mt-4"
            height="6"
            rounded
          />
        </div>

        <!-- Loading state without progress (fallback) -->
        <div v-if="loading && !importProgress" class="progress-area">
          <div class="d-flex align-center mb-4 justify-center">
            <v-progress-circular indeterminate size="64" color="primary" />
          </div>
          <div class="text-subtitle-1 text-center">
            {{ t('zotero.preparing') }}
          </div>
        </div>

        <v-alert v-if="!importResult" type="info" density="compact" class="mt-4">
          <div class="text-caption">
            {{ t('zotero.info') }}
          </div>
        </v-alert>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn @click="handleClose" :disabled="loading">
          {{ importResult ? t('dialog.close') : t('dialog.cancel') }}
        </v-btn>
        <v-btn
          v-if="!importResult && !loading"
          color="primary"
          @click="handleSubmit"
          :loading="loading"
          :disabled="!selectedFile"
        >
          {{ t('zotero.import') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
  .file-drop-zone {
    border: 2px dashed rgba(255, 255, 255, 0.2);
    border-radius: 8px;
    padding: 32px;
    text-align: center;
    transition: all 0.3s;
  }

  .file-drop-zone:hover {
    border-color: rgb(var(--v-theme-primary));
    background-color: rgba(var(--v-theme-primary), 0.05);
  }

  .progress-area {
    padding: 32px;
    text-align: center;
  }

  .gap-3 {
    gap: 12px;
  }

  .gap-4 {
    gap: 16px;
  }

  .stat-card {
    min-width: 100px;
  }

  .cursor-pointer {
    cursor: pointer;
    user-select: none;
  }

  .rotate-180 {
    transform: rotate(180deg);
  }

  .error-list,
  .paper-list {
    max-height: 200px;
    overflow-y: auto;
  }

  .error-item {
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .error-item:last-child {
    border-bottom: none;
  }

  .paper-item {
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .paper-item:last-child {
    border-bottom: none;
  }
</style>
