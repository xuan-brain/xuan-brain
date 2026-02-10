<script setup lang="ts">
import { ref, watch, onUnmounted } from "vue";
import VuePdfEmbed from "vue-pdf-embed";
import { invokeCommand } from "@/lib/tauri";

interface Props {
  src: string; // PDF file path
}

const props = defineProps<Props>();

const currentPage = ref(1);
const totalPages = ref(0);
const loading = ref(true);
const error = ref<string | null>(null);
const scale = ref(1);

// PDF source as blob URL
const pdfSource = ref<string>("");

// Load PDF file
async function loadPdf() {
  loading.value = true;
  error.value = null;

  try {
    // Read PDF file from Tauri backend
    const arrayBuffer = await invokeCommand<number[]>("read_pdf_file", {
      filePath: props.src,
    });

    // Create blob URL
    const blob = new Blob([Uint8Array.from(arrayBuffer)], {
      type: "application/pdf",
    });

    // Revoke previous URL if exists
    if (pdfSource.value) {
      URL.revokeObjectURL(pdfSource.value);
    }

    pdfSource.value = URL.createObjectURL(blob);
  } catch (err) {
    error.value = err as string;
    console.error("Failed to load PDF:", err);
  } finally {
    loading.value = false;
  }
}

// Handle PDF loaded
function handleLoaded(pdf: any) {
  totalPages.value = pdf.numPages;
  loading.value = false;
}

// Page navigation
function nextPage() {
  if (currentPage.value < totalPages.value) {
    currentPage.value++;
  }
}

function prevPage() {
  if (currentPage.value > 1) {
    currentPage.value--;
  }
}

// Zoom functions
function zoomIn() {
  if (scale.value < 3) {
    scale.value += 0.25;
  }
}

function zoomOut() {
  if (scale.value > 0.5) {
    scale.value -= 0.25;
  }
}

function fitToWidth() {
  scale.value = 1;
}

// Watch src changes
watch(() => props.src, loadPdf, { immediate: true });

// Cleanup blob URL on unmount
onUnmounted(() => {
  if (pdfSource.value) {
    URL.revokeObjectURL(pdfSource.value);
  }
});
</script>

<template>
  <div class="pdf-viewer">
    <!-- Toolbar -->
    <v-toolbar density="compact" class="pdf-toolbar">
      <v-btn
        icon="mdi-chevron-left"
        @click="prevPage"
        :disabled="currentPage <= 1 || loading"
      />
      <span class="ml-2 text-caption"
        >{{ currentPage }} / {{ totalPages }}</span
      >
      <v-btn
        icon="mdi-chevron-right"
        @click="nextPage"
        :disabled="currentPage >= totalPages || loading"
      />

      <v-spacer />

      <v-btn icon="mdi-magnify-minus" @click="zoomOut" :disabled="loading" />
      <span class="ml-2 mr-2 text-caption">{{ Math.round(scale * 100) }}%</span>
      <v-btn icon="mdi-magnify-plus" @click="zoomIn" :disabled="loading" />
      <v-btn icon="mdi-fit-to-page" @click="fitToWidth" :disabled="loading" />
    </v-toolbar>

    <!-- PDF rendering area -->
    <div class="pdf-container">
      <!-- Loading state -->
      <div v-if="loading" class="d-flex justify-center align-center h-100">
        <v-progress-circular indeterminate size="64" />
      </div>

      <!-- Error state -->
      <v-alert v-else-if="error" type="error" :text="error" class="ma-4" />

      <!-- PDF content -->
      <div v-else class="pdf-content">
        <vue-pdf-embed
          :source="pdfSource"
          :page="currentPage"
          :scale="scale"
          @loaded="handleLoaded"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.pdf-viewer {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #525659;
}

.pdf-toolbar {
  flex-shrink: 0;
}

.pdf-container {
  flex: 1;
  overflow: auto;
  display: flex;
  justify-content: center;
  padding: 16px;
}

.pdf-content {
  box-shadow: 0 0 10px rgba(0, 0, 0, 0.5);
}

.vue-pdf-embed {
  display: block;
}
</style>
