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
const pmid = ref("");
const loading = ref(false);
const error = ref("");

// Reset form when dialog opens
watch(
  () => props.modelValue,
  (isOpen) => {
    if (isOpen) {
      pmid.value = "";
      error.value = "";
    }
  },
);

// Close dialog
function handleClose() {
  pmid.value = "";
  error.value = "";
  emit("update:modelValue", false);
}

// Submit form
async function handleSubmit() {
  if (!pmid.value.trim()) {
    error.value = "PMID is required";
    return;
  }

  const trimmedPmid = pmid.value.trim();

  loading.value = true;
  try {
    await invokeCommand("import_paper_by_pmid", { pmid: trimmedPmid });
    console.info("Paper imported successfully by PMID:", trimmedPmid);
    pmid.value = "";
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
  if (event.key === "Enter" && !loading.value && pmid.value.trim()) {
    handleSubmit();
  }
}

// Handle paste from clipboard
async function handlePaste() {
  try {
    const text = await navigator.clipboard.readText();
    pmid.value = text;
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
        <v-icon start>mdi-database-search</v-icon>
        Import by PubMed ID
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
          v-model="pmid"
          autofocus
          label="PMID"
          placeholder="12345678 or https://pubmed.ncbi.nlm.nih.gov/12345678/"
          hint="Enter the PubMed ID (PMID) or PubMed URL of the paper"
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
            <br />• 32123456 <br />• https://pubmed.ncbi.nlm.nih.gov/32123456/
            <br />• PMID:32123456
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
          :disabled="!pmid.trim()"
        >
          Import
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>
