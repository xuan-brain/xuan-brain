<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useRoute } from "vue-router";
import PDFViewer from "@/components/pdf/PDFViewer.vue";
import { invokeCommand } from "@/lib/tauri";

const route = useRoute();

// Get paper ID from route params
const paperId = computed(() => parseInt(route.params.paperId as string));

interface PaperDetail {
  id: number;
  title: string;
  authors: string[];
  publication_year?: number;
  journal_name?: string;
  attachment_count?: number;
}

const paper = ref<PaperDetail | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);
const pdfPath = ref<string | null>(null);

// Load paper details
async function loadPaperDetails() {
  if (!paperId.value) return;

  loading.value = true;
  error.value = null;

  try {
    const details = await invokeCommand<PaperDetail>("get_paper", {
      id: paperId.value,
    });
    paper.value = details;

    // Get PDF attachment path
    if (details.attachment_count && details.attachment_count > 0) {
      const attachments = await invokeCommand<any[]>("get_attachments", {
        paperId: paperId.value,
      });
      const pdfAttachment = attachments.find((a: any) => a.file_type === "pdf");
      if (pdfAttachment) {
        const path = await invokeCommand<string>("get_pdf_attachment_path", {
          attachmentId: pdfAttachment.id,
        });
        pdfPath.value = path;
      }
    }
  } catch (err) {
    error.value = err as string;
    console.error("Failed to load paper details:", err);
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  loadPaperDetails();
});
</script>

<template>
  <div class="paper-reader-page">
    <!-- Loading state -->
    <div v-if="loading" class="d-flex justify-center align-center h-100">
      <v-progress-circular indeterminate size="64" />
    </div>

    <!-- Error state -->
    <v-alert v-else-if="error" type="error" :text="error" class="ma-4" />

    <!-- No PDF attachment -->
    <v-alert v-else-if="!pdfPath" type="info" class="ma-4">
      <v-alert-title>No PDF Attachment</v-alert-title>
      <p>This paper doesn't have a PDF attachment yet.</p>
      <v-btn color="primary" class="mt-4">Import PDF</v-btn>
    </v-alert>

    <!-- PDF Viewer -->
    <PDFViewer v-else-if="pdfPath" :src="pdfPath" />
  </div>
</template>

<style scoped>
.paper-reader-page {
  height: 100%;
  display: flex;
  flex-direction: column;
}
</style>
