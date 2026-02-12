<script setup lang="ts">
  import { loadPdfAsBlob, revokePdfBlobUrl, savePdfBlob } from '@/lib/api/pdf';
  import {
    DocumentManagerPlugin,
    ExportPlugin,
    PDFViewer,
    type ExportScope,
    type PluginRegistry,
  } from '@embedpdf/vue-pdf-viewer';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onBeforeUnmount, onMounted, ref } from 'vue';

  const loading = ref(true);
  const error = ref('');
  const pdfUrl = ref('');
  const paperTitle = ref('');
  const fileSizeMB = ref(0);
  const paperId = ref(0);
  const isSaving = ref(false);
  const saveSuccess = ref(false);
  // const registry = ref<PluginRegistry | null>(null);
  const exportScope = ref<ExportScope | null>(null);
  let objectUrl: string | null = null;
  let pdfBlob: Blob | null = null;
  let docId = ref('');

  // Close window function
  async function closeWindow() {
    const currentWindow = getCurrentWindow();
    await currentWindow.close();
  }

  const handleReady = (registry: PluginRegistry) => {
    const docManager = registry.getPlugin<DocumentManagerPlugin>('document-manager')?.provides();
    const exportPlugin = registry.getPlugin<ExportPlugin>('export')?.provides();
    docManager?.onDocumentOpened((doc) => {
      docId.value = doc.id;
      if (exportPlugin) {
        exportScope.value = exportPlugin.forDocument(docId.value);
      }
    });
  };

  async function savePdfWithPlugin() {
    // 1. Get the PDF data as an ArrayBuffer
    const arrayBuffer = await exportScope.value?.saveAsCopy().toPromise();
    if (!arrayBuffer) return;

    // 2. Convert ArrayBuffer to a Blob/File
    const blob = new Blob([arrayBuffer], { type: 'application/pdf' });

    isSaving.value = true;
    try {
      const response = await savePdfBlob(paperId.value, blob);
      saveSuccess.value = true;
      error.value = '';
      console.info('PDF saved successfully:', response.message);

      // Show success message for 3 seconds
      setTimeout(() => {
        saveSuccess.value = false;
      }, 3000);
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
      console.error('Failed to save PDF:', err);
    } finally {
      isSaving.value = false;
    }
  }

  // Save PDF function

  onMounted(async () => {
    try {
      const currentWindow = getCurrentWindow();
      const label = currentWindow.label;

      const idMatch = label.match(/pdf-viewer-(\d+)/);

      if (!idMatch) {
        error.value = 'Invalid PDF viewer window';
        loading.value = false;
        return;
      }

      const id = parseInt(idMatch[1], 10);
      paperId.value = id;
      console.info('Loading PDF for paper:', id);

      // Load PDF as blob from Rust backend
      const { blobUrl, fileName, paperTitle: title, sizeMB } = await loadPdfAsBlob(id);

      // Fetch the blob for saving
      const response = await fetch(blobUrl);
      pdfBlob = await response.blob();

      objectUrl = blobUrl;
      pdfUrl.value = blobUrl;
      paperTitle.value = title;
      fileSizeMB.value = sizeMB;

      console.info(`Successfully loaded PDF: ${fileName} (${sizeMB.toFixed(2)} MB)`);
      await currentWindow.setTitle(title);
    } catch (err) {
      console.error('Failed to load PDF:', err);
      error.value = err instanceof Error ? err.message : String(err);
    } finally {
      loading.value = false;
    }
  });

  onBeforeUnmount(() => {
    if (objectUrl) {
      revokePdfBlobUrl(objectUrl);
      objectUrl = null;
    }
  });
</script>

<template>
  <div class="pdf-viewer">
    <!-- Loading state -->
    <div v-if="loading" class="loading-container">
      <v-progress-circular indeterminate size="64" />
      <p class="mt-4">Loading PDF...</p>
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="error-container">
      <v-alert type="error" :text="error" />
      <v-btn class="mt-4" @click="closeWindow">Close</v-btn>
    </div>

    <!-- PDF viewer -->
    <div v-else class="pdf-container">
      <div class="pdf-header">
        <div class="header-content">
          <h1>{{ paperTitle }}</h1>
          <span class="file-size">{{ fileSizeMB.toFixed(2) }} MB</span>
        </div>
        <div class="header-actions">
          <v-btn
            :disabled="isSaving"
            :loading="isSaving"
            size="small"
            variant="tonal"
            @click="savePdfWithPlugin"
          >
            {{ isSaving ? 'Saving...' : 'Save' }}
          </v-btn>
        </div>
      </div>

      <!-- Save success message -->
      <v-alert
        v-if="saveSuccess"
        type="success"
        class="save-alert"
        text="PDF saved successfully!"
        closable
      />

      <PDFViewer
        v-if="pdfUrl"
        :config="{
          src: pdfUrl,
          theme: { preference: 'light' },
        }"
        :style="{ width: '100%', height: 'calc(100% - 60px)' }"
        @ready="handleReady"
      />
    </div>
  </div>
</template>

<style scoped>
  .pdf-viewer {
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background-color: #fff;
  }

  .loading-container,
  .error-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 20px;
  }

  .pdf-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .pdf-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid #e0e0e0;
    background-color: #f5f5f5;
    gap: 16px;
  }

  .header-content {
    display: flex;
    align-items: center;
    gap: 16px;
    flex: 1;
    min-width: 0;
  }

  .pdf-header h1 {
    margin: 0;
    font-size: 18px;
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-size {
    color: #666;
    font-size: 14px;
    white-space: nowrap;
  }

  .header-actions {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }

  .save-alert {
    margin: 0;
    border-radius: 0;
  }
</style>
