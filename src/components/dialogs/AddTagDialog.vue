<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { invokeCommand } from "@/lib/tauri";
import { useI18n } from "@/lib/i18n";

const { t } = useI18n();

interface Props {
  modelValue: boolean;
  tagId?: number;
  tagName?: string;
  tagColor?: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  tagCreated: [];
}>();

// Available colors for labels
const labelColors = [
  { value: "red", color: "#ef4444" },
  { value: "orange", color: "#f97316" },
  { value: "amber", color: "#f59e0b" },
  { value: "yellow", color: "#eab308" },
  { value: "lime", color: "#84cc16" },
  { value: "green", color: "#22c55e" },
  { value: "emerald", color: "#10b981" },
  { value: "teal", color: "#14b8a6" },
  { value: "cyan", color: "#06b6d4" },
  { value: "sky", color: "#0ea5e9" },
  { value: "blue", color: "#3b82f6" },
  { value: "indigo", color: "#6366f1" },
  { value: "violet", color: "#8b5cf6" },
  { value: "purple", color: "#a855f7" },
  { value: "fuchsia", color: "#d946ef" },
  { value: "pink", color: "#ec4899" },
  { value: "rose", color: "#f43f5e" },
];

// State
const name = ref("");
const selectedColor = ref("blue");
const error = ref("");
const loading = ref(false);

// Check if editing mode
const isEditMode = computed(() => !!props.tagId);

// Dialog title
const dialogTitle = computed(() =>
  isEditMode.value ? t("dialog.editTag") : t("dialog.addTag"),
);

// Reset form when dialog opens
watch(
  () => props.modelValue,
  (isOpen) => {
    if (isOpen) {
      if (isEditMode.value) {
        name.value = props.tagName || "";
        selectedColor.value = props.tagColor || "blue";
      } else {
        name.value = "";
        selectedColor.value = "blue";
      }
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
    error.value = t("dialog.tagNameRequired");
    return;
  }

  if (name.value.length > 30) {
    error.value = t("dialog.tagNameMaxLength");
    return;
  }

  loading.value = true;
  try {
    if (isEditMode.value && props.tagId) {
      // Update existing tag
      await invokeCommand("update_label", {
        id: props.tagId,
        name: name.value.trim(),
        color: selectedColor.value,
      });
      console.info("Label updated successfully:", name.value.trim());
    } else {
      // Create new tag
      await invokeCommand("create_label", {
        name: name.value.trim(),
        color: selectedColor.value,
      });
      console.info("Label created successfully:", name.value.trim());
    }

    name.value = "";
    selectedColor.value = "blue";
    error.value = "";
    emit("tagCreated");
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
        {{ dialogTitle }}
      </v-card-title>

      <v-card-text>
        <v-alert v-if="error" type="error" :text="error" class="mb-4" />

        <v-text-field
          v-model="name"
          autofocus
          :label="t('dialog.enterTagName')"
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
          {{ t("dialog.save") }}
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
