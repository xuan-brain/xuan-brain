<script setup lang="ts">
  import { useI18n } from '@/lib/i18n';
  import { invokeCommand } from '@/lib/tauri';
  import { open } from '@tauri-apps/plugin-dialog';
  import { ref } from 'vue';

  interface Props {
    onRefresh?: () => void;
    selectedCategoryId?: number | null;
  }

  const props = withDefaults(defineProps<Props>(), {
    onRefresh: undefined,
    selectedCategoryId: null,
  });

  const { t } = useI18n();

  // DOI Dialog states
  const doiDialogOpen = ref(false);
  const doiInput = ref('');
  const doiLoading = ref(false);
  const doiError = ref('');

  // arXiv Dialog states
  const arxivDialogOpen = ref(false);
  const arxivInput = ref('');
  const arxivLoading = ref(false);
  const arxivError = ref('');

  // PubMed Dialog states
  const pmidDialogOpen = ref(false);
  const pmidInput = ref('');
  const pmidLoading = ref(false);
  const pmidError = ref('');

  // DOI import handlers
  function handleDoiButtonClick() {
    doiDialogOpen.value = true;
    doiInput.value = '';
    doiError.value = '';
  }

  function handleDoiDialogClose() {
    doiDialogOpen.value = false;
    doiInput.value = '';
    doiError.value = '';
  }

  async function handleDoiSubmit() {
    const trimmed = doiInput.value.trim();
    if (!trimmed) {
      doiError.value = t('toolbar.doiRequired');
      return;
    }

    doiLoading.value = true;
    doiError.value = '';

    try {
      await invokeCommand('import_paper_by_doi', {
        doi: trimmed,
        categoryId: props.selectedCategoryId,
      });

      // Refresh the document list
      if (props.onRefresh) {
        await props.onRefresh();
      }

      handleDoiDialogClose();
    } catch (error) {
      console.error('Failed to import paper by DOI:', error);
      doiError.value = String(error);
    } finally {
      doiLoading.value = false;
    }
  }

  // arXiv import handlers
  function handleArxivButtonClick() {
    arxivDialogOpen.value = true;
    arxivInput.value = '';
    arxivError.value = '';
  }

  function handleArxivDialogClose() {
    arxivDialogOpen.value = false;
    arxivInput.value = '';
    arxivError.value = '';
  }

  async function handleArxivSubmit() {
    const trimmed = arxivInput.value.trim();
    if (!trimmed) {
      arxivError.value = t('toolbar.arxivRequired');
      return;
    }

    arxivLoading.value = true;
    arxivError.value = '';

    try {
      await invokeCommand('import_paper_by_arxiv_id', {
        arxivId: trimmed,
        categoryId: props.selectedCategoryId,
      });

      // Refresh the document list
      if (props.onRefresh) {
        await props.onRefresh();
      }

      handleArxivDialogClose();
    } catch (error) {
      console.error('Failed to import paper by arXiv ID:', error);
      arxivError.value = String(error);
    } finally {
      arxivLoading.value = false;
    }
  }

  // PubMed import handlers
  function handlePmidButtonClick() {
    pmidDialogOpen.value = true;
    pmidInput.value = '';
    pmidError.value = '';
  }

  function handlePmidDialogClose() {
    pmidDialogOpen.value = false;
    pmidInput.value = '';
    pmidError.value = '';
  }

  async function handlePmidSubmit() {
    const trimmed = pmidInput.value.trim();
    if (!trimmed) {
      pmidError.value = t('toolbar.pmidRequired');
      return;
    }

    pmidLoading.value = true;
    pmidError.value = '';

    try {
      await invokeCommand('import_paper_by_pmid', {
        pmid: trimmed,
        categoryId: props.selectedCategoryId,
      });

      // Refresh the document list
      if (props.onRefresh) {
        await props.onRefresh();
      }

      handlePmidDialogClose();
    } catch (error) {
      console.error('Failed to import paper by PMID:', error);
      pmidError.value = String(error);
    } finally {
      pmidLoading.value = false;
    }
  }

  // PDF import handler
  async function handlePdfImport() {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [{ name: 'PDF', extensions: ['pdf'] }],
      });

      if (selected) {
        const filePath = Array.isArray(selected) ? selected[0] : selected;
        if (filePath) {
          try {
            await invokeCommand('import_paper_by_pdf', {
              filePath: filePath,
              categoryId: props.selectedCategoryId,
            });

            if (props.onRefresh) {
              await props.onRefresh();
            }
          } catch (error) {
            console.error('Failed to import PDF:', error);
            throw error;
          }
        }
      }
    } catch (error) {
      console.error('Failed to select PDF:', error);
    }
  }
</script>

<template>
  <!-- Toolbar -->
  <div class="document-toolbar">
    <div class="toolbar-actions">
      <v-btn variant="tonal" prepend-icon="mdi-file-pdf-box" class="toolbar-btn" @click="handlePdfImport">
        {{ t('toolbar.importPdf') }}
      </v-btn>
      <v-btn
        variant="tonal"
        prepend-icon="mdi-identifier"
        class="toolbar-btn"
        @click="handleDoiButtonClick"
      >
        {{ t('toolbar.doi') }}
      </v-btn>
      <v-btn
        variant="tonal"
        prepend-icon="mdi-file-document-outline"
        class="toolbar-btn"
        @click="handleArxivButtonClick"
      >
        {{ t('toolbar.arxiv') }}
      </v-btn>
      <v-btn
        variant="tonal"
        prepend-icon="mdi-database-search"
        class="toolbar-btn"
        @click="handlePmidButtonClick"
      >
        {{ t('toolbar.pubmed') }}
      </v-btn>
    </div>
  </div>

  <!-- DOI Import Dialog -->
  <v-dialog v-model="doiDialogOpen" max-width="480" @click:outside="handleDoiDialogClose">
    <v-card>
      <v-card-title>{{ t('toolbar.importByDoi') }}</v-card-title>
      <v-card-text>
        <div class="dialog-description">{{ t('toolbar.doiDescription') }}</div>
        <v-text-field
          v-model="doiInput"
          :label="t('toolbar.doiPlaceholder')"
          :placeholder="t('toolbar.doiPlaceholder')"
          :error-messages="doiError"
          autofocus
          variant="outlined"
          density="compact"
          :disabled="doiLoading"
          @keyup.enter="handleDoiSubmit"
        />
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn @click="handleDoiDialogClose" :disabled="doiLoading">
          {{ t('dialog.cancel') }}
        </v-btn>
        <v-btn
          color="primary"
          variant="tonal"
          @click="handleDoiSubmit"
          :loading="doiLoading"
          :disabled="!doiInput.trim()"
        >
          {{ t('toolbar.import') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <!-- arXiv Import Dialog -->
  <v-dialog v-model="arxivDialogOpen" max-width="480" @click:outside="handleArxivDialogClose">
    <v-card>
      <v-card-title>{{ t('toolbar.importByArxiv') }}</v-card-title>
      <v-card-text>
        <div class="dialog-description">
          {{ t('toolbar.arxivDescription') }}
        </div>
        <v-text-field
          v-model="arxivInput"
          :label="t('toolbar.arxivPlaceholder')"
          :placeholder="t('toolbar.arxivPlaceholder')"
          :error-messages="arxivError"
          autofocus
          variant="outlined"
          density="compact"
          :disabled="arxivLoading"
          @keyup.enter="handleArxivSubmit"
        />
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn @click="handleArxivDialogClose" :disabled="arxivLoading">
          {{ t('dialog.cancel') }}
        </v-btn>
        <v-btn
          color="primary"
          variant="tonal"
          @click="handleArxivSubmit"
          :loading="arxivLoading"
          :disabled="!arxivInput.trim()"
        >
          {{ t('toolbar.import') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <!-- PubMed Import Dialog -->
  <v-dialog v-model="pmidDialogOpen" max-width="480" @click:outside="handlePmidDialogClose">
    <v-card>
      <v-card-title>{{ t('toolbar.importByPubmed') }}</v-card-title>
      <v-card-text>
        <div class="dialog-description">
          {{ t('toolbar.pubmedDescription') }}
        </div>
        <v-text-field
          v-model="pmidInput"
          :label="t('toolbar.pubmedPlaceholder')"
          :placeholder="t('toolbar.pubmedPlaceholder')"
          :error-messages="pmidError"
          autofocus
          variant="outlined"
          density="compact"
          :disabled="pmidLoading"
          @keyup.enter="handlePmidSubmit"
        />
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn @click="handlePmidDialogClose" :disabled="pmidLoading">
          {{ t('dialog.cancel') }}
        </v-btn>
        <v-btn
          color="primary"
          variant="tonal"
          @click="handlePmidSubmit"
          :loading="pmidLoading"
          :disabled="!pmidInput.trim()"
        >
          {{ t('toolbar.import') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
  .document-toolbar {
    border-bottom: 1px solid var(--vxe-table-border-color);
    min-height: 56px;
    padding: 10px 12px;
    background-color: var(--vxe-table-body-background-color);
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  .toolbar-actions {
    display: flex;
    gap: 10px;
    align-items: center;
  }

  .toolbar-btn {
    height: 36px !important;
    padding: 0 16px !important;
  }

  .toolbar-btn :deep(.v-icon) {
    font-size: 20px !important;
  }

  .dialog-description {
    margin-bottom: 16px;
    color: rgba(var(--v-theme-on-surface), 0.7);
    font-size: 14px;
    line-height: 1.5;
  }
</style>
