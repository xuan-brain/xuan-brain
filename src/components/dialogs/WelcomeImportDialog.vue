<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "@/lib/i18n";

interface Props {
  modelValue: boolean;
}

defineProps<Props>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  importZotero: [];
  goToImport: [];
}>();

const { t } = useI18n();

// Track if user doesn't want to see this again
const dontShowAgain = ref(false);

function handleClose() {
  // If user checked "don't show again", save to localStorage
  if (dontShowAgain.value) {
    localStorage.setItem("hide-welcome-import-dialog", "true");
  }
  emit("update:modelValue", false);
}

function handleImportZotero() {
  if (dontShowAgain.value) {
    localStorage.setItem("hide-welcome-import-dialog", "true");
  }
  emit("importZotero");
  emit("update:modelValue", false);
}

function handleGoToImport() {
  if (dontShowAgain.value) {
    localStorage.setItem("hide-welcome-import-dialog", "true");
  }
  emit("goToImport");
  emit("update:modelValue", false);
}
</script>

<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="emit('update:modelValue', $event)"
    max-width="500"
    persistent
  >
    <v-card>
      <v-card-title class="d-flex align-center">
        <v-icon start color="primary" size="28">mdi-book-open-page-variant</v-icon>
        {{ t('welcome.title') }}
      </v-card-title>

      <v-card-text>
        <p class="text-body-1 mb-4">
          {{ t('welcome.description') }}
        </p>

        <v-alert type="info" density="compact" class="mb-4">
          <div class="text-caption">
            {{ t('welcome.zoteroTip') }}
          </div>
        </v-alert>

        <v-checkbox
          v-model="dontShowAgain"
          :label="t('welcome.dontShowAgain')"
          density="compact"
          hide-details
        />
      </v-card-text>

      <v-card-actions class="px-4 pb-4">
        <v-btn variant="text" @click="handleClose">
          {{ t('welcome.later') }}
        </v-btn>
        <v-spacer />
        <v-btn
          color="primary"
          variant="outlined"
          class="mr-2"
          @click="handleGoToImport"
        >
          <v-icon start>mdi-book-plus-multiple</v-icon>
          {{ t('welcome.otherImport') }}
        </v-btn>
        <v-btn
          color="primary"
          @click="handleImportZotero"
        >
          <v-icon start>mdi-database-import</v-icon>
          {{ t('welcome.importZotero') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
/* Additional styles if needed */
</style>
