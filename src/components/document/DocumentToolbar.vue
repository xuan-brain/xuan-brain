<script setup lang="ts">
import { ref, watch } from "vue";
import { invokeCommand } from "@/lib/tauri";
import { open } from "@tauri-apps/plugin-dialog";
import { useI18n } from "@/lib/i18n";

interface Props {
  onRefresh?: () => void;
  selectedCategoryId?: number | null;
}

const props = withDefaults(defineProps<Props>(), {
  onRefresh: undefined,
  selectedCategoryId: null,
});

const { t } = useI18n();

// Dialog states
const doiDialogOpen = ref(false);
const doiInput = ref("");
const doiLoading = ref(false);
const doiError = ref("");

const arxivDialogOpen = ref(false);
const arxivInput = ref("");
const arxivLoading = ref(false);
const arxivError = ref("");

// DOI import handlers
function handleDoiButtonClick() {
  doiDialogOpen.value = true;
  doiInput.value = "";
  doiError.value = "";
}

function handleDoiDialogClose() {
  doiDialogOpen.value = false;
  doiInput.value = "";
  doiError.value = "";
}

async function handleDoiSubmit() {
  const trimmed = doiInput.value.trim();
  if (!trimmed) {
    doiError.value = t("toolbar.doiRequired");
    return;
  }

  doiLoading.value = true;
  doiError.value = "";

  try {
    await invokeCommand("import_paper_by_doi", {
      doi: trimmed,
      categoryId: props.selectedCategoryId,
    });

    // Refresh the document list
    if (props.onRefresh) {
      await props.onRefresh();
    }

    handleDoiDialogClose();
  } catch (error) {
    console.error("Failed to import paper by DOI:", error);
    doiError.value = String(error);
  } finally {
    doiLoading.value = false;
  }
}

// arXiv import handlers
function handleArxivButtonClick() {
  arxivDialogOpen.value = true;
  arxivInput.value = "";
  arxivError.value = "";
}

function handleArxivDialogClose() {
  arxivDialogOpen.value = false;
  arxivInput.value = "";
  arxivError.value = "";
}

async function handleArxivSubmit() {
  const trimmed = arxivInput.value.trim();
  if (!trimmed) {
    arxivError.value = t("toolbar.arxivRequired");
    return;
  }

  arxivLoading.value = true;
  arxivError.value = "";

  try {
    await invokeCommand("import_paper_by_arxiv_id", {
      arxivId: trimmed,
      categoryId: props.selectedCategoryId,
    });

    // Refresh the document list
    if (props.onRefresh) {
      await props.onRefresh();
    }

    handleArxivDialogClose();
  } catch (error) {
    console.error("Failed to import paper by arXiv ID:", error);
    arxivError.value = String(error);
  } finally {
    arxivLoading.value = false;
  }
}

// PDF import handler
async function handlePdfImport() {
  try {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });

    if (selected) {
      const filePath = Array.isArray(selected) ? selected[0] : selected;
      if (filePath) {
        try {
          await invokeCommand("import_paper_by_pdf", {
            filePath: filePath,
            categoryId: props.selectedCategoryId,
          });

          if (props.onRefresh) {
            await props.onRefresh();
          }
        } catch (error) {
          console.error("Failed to import PDF:", error);
          throw error;
        }
      }
    }
  } catch (error) {
    console.error("Failed to select PDF:", error);
  }
}
</script>

<template>
  <!-- Toolbar -->
  <div class="document-toolbar">
    <div class="toolbar-actions">
      <v-btn
        size="small"
        variant="tonal"
        prepend-icon="mdi-file-pdf-box"
        @click="handlePdfImport"
      >
        {{ t("toolbar.importPdf") }}
      </v-btn>
      <v-btn size="small" variant="tonal" prepend-icon="mdi-identifier" @click="handleDoiButtonClick">
        {{ t("toolbar.doi") }}
      </v-btn>
      <v-btn
        size="small"
        variant="tonal"
        prepend-icon="mdi-file-document-outline"
        @click="handleArxivButtonClick"
      >
        {{ t("toolbar.arxiv") }}
      </v-btn>
    </div>
  </div>

  <!-- DOI Import Dialog -->
  <v-dialog v-model="doiDialogOpen" max-width="480" @click:outside="handleDoiDialogClose">
    <v-card>
      <v-card-title>{{ t("toolbar.importByDoi") }}</v-card-title>
      <v-card-text>
        <div class="dialog-description">{{ t("toolbar.doiDescription") }}</div>
        <v-text-field
          v-model="doiInput"
          :label="t('toolbar.doiPlaceholder')"
          :placeholder="t('toolbar.doiPlaceholder')"
          :error-messages="doiError"
          autofocus
          variant="outlined"
          density="compact"
          :disabled="doiLoading"
          @keyup.enter="handleDoiSubmit"
        />
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn @click="handleDoiDialogClose" :disabled="doiLoading">
          {{ t("dialog.cancel") }}
        </v-btn>
        <v-btn
          color="primary"
          variant="tonal"
          @click="handleDoiSubmit"
          :loading="doiLoading"
          :disabled="!doiInput.trim()"
        >
          {{ t("toolbar.import") }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <!-- arXiv Import Dialog -->
  <v-dialog v-model="arxivDialogOpen" max-width="480" @click:outside="handleArxivDialogClose">
    <v-card>
      <v-card-title>{{ t("toolbar.importByArxiv") }}</v-card-title>
      <v-card-text>
        <div class="dialog-description">{{ t("toolbar.arxivDescription") }}</div>
        <v-text-field
          v-model="arxivInput"
          :label="t('toolbar.arxivPlaceholder')"
          :placeholder="t('toolbar.arxivPlaceholder')"
          :error-messages="arxivError"
          autofocus
          variant="outlined"
          density="compact"
          :disabled="arxivLoading"
          @keyup.enter="handleArxivSubmit"
        />
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn @click="handleArxivDialogClose" :disabled="arxivLoading">
          {{ t("dialog.cancel") }}
        </v-btn>
        <v-btn
          color="primary"
          variant="tonal"
          @click="handleArxivSubmit"
          :loading="arxivLoading"
          :disabled="!arxivInput.trim()"
        >
          {{ t("toolbar.import") }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
.document-toolbar {
  border-bottom: 1px solid var(--vxe-table-border-color);
  min-height: 48px;
  padding: 8px 12px;
  background-color: var(--vxe-table-body-background-color);
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.toolbar-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.dialog-description {
  margin-bottom: 16px;
  color: rgba(var(--v-theme-on-surface), 0.7);
  font-size: 14px;
  line-height: 1.5;
}
</style>
