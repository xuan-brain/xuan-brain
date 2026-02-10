<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "@/lib/i18n";
import DocumentList from "@/components/document/DocumentList.vue";
import DocumentDetails from "@/components/document/DocumentDetails.vue";
import AddCategoryDialog from "@/components/dialogs/AddCategoryDialog.vue";
import AddTagDialog from "@/components/dialogs/AddTagDialog.vue";
import ImportByDoiDialog from "@/components/dialogs/ImportByDoiDialog.vue";
import ImportByArxivDialog from "@/components/dialogs/ImportByArxivDialog.vue";
import ImportPdfDialog from "@/components/dialogs/ImportPdfDialog.vue";

const { t } = useI18n();

// Props
interface Props {
  selectedCategory?: string | null;
}

const props = withDefaults(defineProps<Props>(), {
  selectedCategory: null,
});

// State
const selectedPaperId = ref<number | null>(null);
const documentListRef = ref<InstanceType<typeof DocumentList> | null>(null);

// Dialog states
const showAddCategoryDialog = ref(false);
const showAddTagDialog = ref(false);
const showImportDoiDialog = ref(false);
const showImportArxivDialog = ref(false);
const showImportPdfDialog = ref(false);

// Handle paper selection from document list
function handlePaperSelect(paperId: number) {
  selectedPaperId.value = paperId;
}

// Refresh document list
function refreshDocumentList() {
  documentListRef.value?.loadPapers();
}

// Show add category dialog
function showAddCategory() {
  showAddCategoryDialog.value = true;
}

// Show add tag dialog
function showAddTag() {
  showAddTagDialog.value = true;
}

// Import paper functions
function importByDoi() {
  showImportDoiDialog.value = true;
}

function importByArxiv() {
  showImportArxivDialog.value = true;
}

function importPdf() {
  showImportPdfDialog.value = true;
}
</script>

<template>
  <v-container fluid class="pa-0 fill-height papers-page">
    <v-row no-gutters class="fill-height">
      <!-- Document List -->
      <v-col cols="12" md="6" lg="7" class="document-list-col">
        <!-- Toolbar -->
        <v-toolbar density="compact" color="surface">
          <v-toolbar-title>{{ t("main.documents") }}</v-toolbar-title>
          <v-spacer />

          <v-btn
            icon="mdi-label-plus"
            @click="showAddTag"
            :title="t('dialog.addLabel')"
          >
            <v-icon>mdi-label-plus</v-icon>
          </v-btn>

          <v-btn
            icon="mdi-folder-plus"
            @click="showAddCategory"
            :title="t('dialog.addCategory')"
          >
            <v-icon>mdi-folder-plus</v-icon>
          </v-btn>

          <v-menu>
            <template #activator="{ props }">
              <v-btn
                icon="mdi-plus"
                v-bind="props"
                :title="t('toolbar.import')"
              >
                <v-icon>mdi-plus</v-icon>
              </v-btn>
            </template>
            <v-list>
              <v-list-item @click="importByDoi">
                <v-list-item-title>Import by DOI</v-list-item-title>
              </v-list-item>
              <v-list-item @click="importByArxiv">
                <v-list-item-title>Import by arXiv ID</v-list-item-title>
              </v-list-item>
              <v-list-item @click="importPdf">
                <v-list-item-title>Import PDF File</v-list-item-title>
              </v-list-item>
            </v-list>
          </v-menu>
        </v-toolbar>

        <!-- Document List -->
        <div class="document-list-wrapper">
          <DocumentList
            ref="documentListRef"
            :category-path="props.selectedCategory"
            @paper-select="handlePaperSelect"
          />
        </div>
      </v-col>

      <!-- Document Details -->
      <v-col cols="12" md="6" lg="5" class="document-details-col">
        <DocumentDetails :paper-id="selectedPaperId" />
      </v-col>
    </v-row>

    <!-- Dialogs -->
    <AddCategoryDialog
      v-model="showAddCategoryDialog"
      :parent-path="props.selectedCategory"
      @category-created="refreshDocumentList"
    />

    <AddTagDialog
      v-model="showAddTagDialog"
      @label-created="refreshDocumentList"
    />

    <ImportByDoiDialog
      v-model="showImportDoiDialog"
      @paper-imported="refreshDocumentList"
    />

    <ImportByArxivDialog
      v-model="showImportArxivDialog"
      @paper-imported="refreshDocumentList"
    />

    <ImportPdfDialog
      v-model="showImportPdfDialog"
      @paper-imported="refreshDocumentList"
    />
  </v-container>
</template>

<style scoped>
.papers-page {
  height: 100%;
  overflow: hidden;
}

.document-list-col,
.document-details-col {
  height: 100%;
  overflow: hidden;
  border-left: 1px solid rgba(255, 255, 255, 0.12);
}

.document-list-wrapper {
  height: calc(100% - 48px); /* Subtract toolbar height */
  overflow: auto;
}
</style>
