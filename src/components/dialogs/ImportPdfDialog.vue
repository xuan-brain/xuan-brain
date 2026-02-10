<script setup lang="ts">
import { ref, watch } from "vue";
import { invokeCommand } from "@/lib/tauri";

interface Props {
  modelValue: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  paperImported: [];
}>();

// State
const selectedFile = ref<string | null>(null);
const loading = ref(false);
const error = ref("");

// Reset form when dialog opens
watch(
  () => props.modelValue,
  (isOpen) => {
    if (isOpen) {
      selectedFile.value = null;
      error.value = "";
    }
  },
);

// Close dialog
function handleClose() {
  selectedFile.value = null;
  error.value = "";
  emit("update:modelValue", false);
}

// Open file picker
async function selectFile() {
  try {
    // Use Tauri file dialog
    const { open } = await import("@tauri-apps/plugin-dialog");

    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "PDF",
          extensions: ["pdf"],
        },
      ],
    });

    if (selected && typeof selected === "string") {
      selectedFile.value = selected;
      error.value = "";
    }
  } catch (err) {
    console.error("Failed to open file dialog:", err);
    error.value = "Failed to open file selector";
  }
}

// Submit form
async function handleSubmit() {
  if (!selectedFile.value) {
    error.value = "Please select a PDF file";
    return;
  }

  loading.value = true;
  try {
    await invokeCommand("import_paper_by_pdf", {
      filePath: selectedFile.value,
      categoryId: null,
    });
    console.info("Paper imported successfully from PDF:", selectedFile.value);
    selectedFile.value = null;
    error.value = "";
    emit("paperImported");
    emit("update:modelValue", false);
  } catch (err) {
    error.value = err as string;
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="emit('update:modelValue', $event)"
    max-width="500"
  >
    <v-card>
      <v-card-title>
        <v-icon start>mdi-file-pdf-box</v-icon>
        Import PDF File
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

        <!-- File selection area -->
        <div class="file-drop-zone">
          <v-icon size="64" color="primary">mdi-file-upload</v-icon>
          <div class="text-h6 mt-4">Select PDF File</div>
          <div class="text-caption text-grey mt-2">
            {{
              selectedFile ? selectedFile : "Click to browse or drag and drop"
            }}
          </div>

          <v-btn
            color="primary"
            class="mt-4"
            @click="selectFile"
            :disabled="loading"
          >
            <v-icon start>mdi-folder-open</v-icon>
            Browse Files
          </v-btn>
        </div>

        <v-alert type="info" density="compact" class="mt-4">
          <div class="text-caption">
            The PDF will be imported and metadata will be extracted
            automatically. You can edit the metadata after import.
          </div>
        </v-alert>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn @click="handleClose" :disabled="loading"> Cancel </v-btn>
        <v-btn
          color="primary"
          @click="handleSubmit"
          :loading="loading"
          :disabled="!selectedFile"
        >
          Import
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
</style>
