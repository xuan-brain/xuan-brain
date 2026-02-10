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
const doi = ref("");
const loading = ref(false);
const error = ref("");

// Reset form when dialog opens
watch(
  () => props.modelValue,
  (isOpen) => {
    if (isOpen) {
      doi.value = "";
      error.value = "";
    }
  },
);

// Close dialog
function handleClose() {
  doi.value = "";
  error.value = "";
  emit("update:modelValue", false);
}

// Submit form
async function handleSubmit() {
  if (!doi.value.trim()) {
    error.value = "DOI is required";
    return;
  }

  const trimmedDoi = doi.value.trim();

  // Add "doi:" prefix if not present
  const doiInput = trimmedDoi.startsWith("doi:")
    ? trimmedDoi
    : `doi:${trimmedDoi}`;

  loading.value = true;
  try {
    await invokeCommand("import_paper_by_doi", { doi: doiInput });
    console.info("Paper imported successfully by DOI:", doiInput);
    doi.value = "";
    error.value = "";
    emit("paperImported");
    emit("update:modelValue", false);
  } catch (err) {
    error.value = err as string;
  } finally {
    loading.value = false;
  }
}

// Handle Enter key
function handleKeyPress(event: KeyboardEvent) {
  if (event.key === "Enter" && !loading.value && doi.value.trim()) {
    handleSubmit();
  }
}

// Handle paste from clipboard
async function handlePaste() {
  try {
    const text = await navigator.clipboard.readText();
    doi.value = text;
    error.value = "";
  } catch (err) {
    console.error("Failed to read clipboard:", err);
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
        <v-icon start>mdi-import</v-icon>
        Import by DOI
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

        <v-text-field
          v-model="doi"
          autofocus
          label="DOI"
          placeholder="10.1234/example.doi.12345"
          hint="Enter the DOI of the paper you want to import"
          variant="outlined"
          :error-messages="error ? [error] : []"
          :disabled="loading"
          @keyup="handleKeyPress"
        >
          <template #append-inner>
            <v-tooltip location="top">
              <template #activator="{ props }">
                <v-btn
                  icon="mdi-content-paste"
                  size="small"
                  variant="text"
                  v-bind="props"
                  @click="handlePaste"
                  :disabled="loading"
                />
              </template>
              <span>Paste from clipboard</span>
            </v-tooltip>
          </template>
        </v-text-field>

        <v-alert type="info" density="compact" class="mt-4">
          <div class="text-caption">
            Examples:
            <br />• 10.1038/s41586-021-03819-2 <br />• 10.1109/5.771073
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
          :disabled="!doi.trim()"
        >
          Import
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>
