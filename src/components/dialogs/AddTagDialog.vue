<script setup lang="ts">
import { ref, watch } from "vue";
import { invokeCommand } from "@/lib/tauri";
import { useI18n } from "@/lib/i18n";

const { t } = useI18n();

interface Props {
  modelValue: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  labelCreated: [];
}>();

// Available colors for labels
const labelColors = [
  { value: "red", color: "#f44336" },
  { value: "pink", color: "#e91e63" },
  { value: "purple", color: "#9c27b0" },
  { value: "deep-purple", color: "#673ab7" },
  { value: "indigo", color: "#3f51b5" },
  { value: "blue", color: "#2196f3" },
  { value: "light-blue", color: "#03a9f4" },
  { value: "cyan", color: "#00bcd4" },
  { value: "teal", color: "#009688" },
  { value: "green", color: "#4caf50" },
  { value: "light-green", color: "#8bc34a" },
  { value: "lime", color: "#cddc39" },
  { value: "yellow", color: "#ffeb3b" },
  { value: "amber", color: "#ffc107" },
  { value: "orange", color: "#ff9800" },
  { value: "deep-orange", color: "#ff5722" },
  { value: "brown", color: "#795548" },
  { value: "grey", color: "#9e9e9e" },
  { value: "blue-grey", color: "#607d8b" },
];

// State
const name = ref("");
const selectedColor = ref("blue");
const error = ref("");
const loading = ref(false);

// Reset form when dialog opens
watch(
  () => props.modelValue,
  (isOpen) => {
    if (isOpen) {
      name.value = "";
      selectedColor.value = "blue";
      error.value = "";
    }
  },
);

// Close dialog
function handleClose() {
  name.value = "";
  selectedColor.value = "blue";
  error.value = "";
  emit("update:modelValue", false);
}

// Submit form
async function handleSubmit() {
  if (!name.value.trim()) {
    error.value = t("dialog.labelNameRequired");
    return;
  }

  if (name.value.length > 30) {
    error.value = t("dialog.labelNameMaxLength");
    return;
  }

  loading.value = true;
  try {
    await invokeCommand("create_label", {
      name: name.value.trim(),
      color: selectedColor.value,
    });
    console.info("Label created successfully:", name.value.trim());
    name.value = "";
    selectedColor.value = "blue";
    error.value = "";
    emit("labelCreated");
    emit("update:modelValue", false);
  } catch (err) {
    error.value = err as string;
  } finally {
    loading.value = false;
  }
}

// Handle Enter key
function handleKeyPress(event: KeyboardEvent) {
  if (
    event.key === "Enter" &&
    !loading.value &&
    name.value.trim() &&
    name.value.length <= 30
  ) {
    handleSubmit();
  }
}
</script>

<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="emit('update:modelValue', $event)"
    max-width="480"
  >
    <v-card>
      <v-card-title>
        <v-icon start>mdi-label</v-icon>
        {{ t("dialog.addLabel") }}
      </v-card-title>

      <v-card-text>
        <v-alert v-if="error" type="error" :text="error" class="mb-4" />

        <v-text-field
          v-model="name"
          autofocus
          :label="t('dialog.enterLabelName')"
          variant="outlined"
          :error-messages="error ? [error] : []"
          :disabled="loading"
          @keyup="handleKeyPress"
        />

        <div class="mt-4">
          <div class="text-caption text-grey mb-2">
            {{ t("dialog.selectColor") }}
          </div>
          <div class="color-selector">
            <div
              v-for="colorOption in labelColors"
              :key="colorOption.value"
              class="color-option"
              :class="{ selected: selectedColor === colorOption.value }"
              :style="{ backgroundColor: colorOption.color }"
              @click="selectedColor = colorOption.value"
            >
              <v-icon
                v-if="selectedColor === colorOption.value"
                color="white"
                size="small"
              >
                mdi-check
              </v-icon>
            </div>
          </div>
        </div>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn @click="handleClose" :disabled="loading">
          {{ t("dialog.cancel") }}
        </v-btn>
        <v-btn
          color="primary"
          @click="handleSubmit"
          :loading="loading"
          :disabled="!name.trim() || name.length > 30"
        >
          {{ t("dialog.add") }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
.color-selector {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.color-option {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  border: 2px solid transparent;
  transition: all 0.2s;
}

.color-option:hover {
  transform: scale(1.1);
}

.color-option.selected {
  border-color: rgb(var(--v-theme-primary));
  box-shadow: 0 0 8px rgba(var(--v-theme-primary), 0.5);
}
</style>
