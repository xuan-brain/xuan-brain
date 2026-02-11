<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invokeCommand } from "@/lib/tauri";
import { getCurrentWindow } from "@tauri-apps/api/window";

const loading = ref(true);
const error = ref("");
const pdfDataUrl = ref("");
const paperTitle = ref("");
const windowLabel = ref("");

// Close window function
async function closeWindow() {
  const currentWindow = getCurrentWindow();
  await currentWindow.close();
}

onMounted(async () => {
  try {
    const currentWindow = getCurrentWindow();
    const label = await currentWindow.label();
    windowLabel.value = label;

    const idMatch = label.match(/pdf-viewer-(\d+)/);

    if (!idMatch) {
      error.value = "Invalid PDF viewer window";
      loading.value = false;
      return;
    }

    const id = parseInt(idMatch[1], 10);

    // Get PDF base64 content from backend
    const info = await invokeCommand<{
      file_name: string;
      base64_content: string;
      paper_title: string;
    }>("get_pdf_attachment_base64", { paperId: id });

    // Convert base64 to blob URL
    const byteCharacters = atob(info.base64_content);
    const byteArray = new Uint8Array(byteCharacters.length);
    for (let i = 0; i < byteCharacters.length; i++) {
      byteArray[i] = byteCharacters.charCodeAt(i);
    }
    const blob = new Blob([byteArray], { type: "application/pdf" });
    pdfDataUrl.value = URL.createObjectURL(blob);
    paperTitle.value = info.paper_title;
    await currentWindow.setTitle(info.paper_title);
  } catch (err) {
    console.error("Failed to load PDF:", err);
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    loading.value = false;
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
      <embed
        v-if="pdfDataUrl"
        :src="pdfDataUrl"
        type="application/pdf"
        class="pdf-embed"
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

.pdf-embed {
  width: 100%;
  height: 100%;
  border: none;
}
</style>
