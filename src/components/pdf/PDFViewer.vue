<script setup lang="ts">
  import { invokeCommand } from '@/lib/tauri';
  import { PDFViewer } from '@embedpdf/vue-pdf-viewer';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { readBinaryFile } from '@tauri-apps/plugin-fs';
  import { onBeforeUnmount, onMounted, ref } from 'vue';

  const loading = ref(true);
  const error = ref('');
  const pdfUrl = ref('');
  const paperTitle = ref('');
  let objectUrl: string | null = null;

  // Close window function
  async function closeWindow() {
    const currentWindow = getCurrentWindow();
    await currentWindow.close();
  }

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
      const info = await invokeCommand<{
        file_path: string;
        file_name: string;
        paper_id: number;
        paper_title: string;
      }>('get_pdf_attachment_path', { paperId: id });
      console.info('pdf info ', info);

      // Frontend: read file directly and build a blob URL
      const data = await readBinaryFile(info.file_path);
      const blob = new Blob([data], { type: 'application/pdf' });
      objectUrl = URL.createObjectURL(blob);

      pdfUrl.value = objectUrl;
      paperTitle.value = info.paper_title;
      await currentWindow.setTitle(info.paper_title);
    } catch (err) {
      console.error('Failed to load PDF:', err);
      error.value = err instanceof Error ? err.message : String(err);
    } finally {
      loading.value = false;
    }
  });

  onBeforeUnmount(() => {
    if (objectUrl) {
      URL.revokeObjectURL(objectUrl);
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
      <PDFViewer
        v-if="pdfUrl"
        :config="{
          src: pdfUrl,
          theme: { preference: 'light' },
        }"
        :style="{ width: '100%', height: '100%' }"
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
  }
</style>
