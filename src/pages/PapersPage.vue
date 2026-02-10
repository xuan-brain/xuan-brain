<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
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

// Panel widths (in percentage)
const STORAGE_KEY = "papers-page-panel-widths";
const defaultWidths = { left: 60, right: 40 }; // Only two panels now

const panelWidths = ref({ ...defaultWidths });

// Load saved widths from localStorage
onMounted(() => {
  const saved = localStorage.getItem(STORAGE_KEY);
  if (saved) {
    try {
      const parsed = JSON.parse(saved);
      // Validate and apply saved widths
      if (parsed && typeof parsed === "object") {
        const total = (parsed.left || 0) + (parsed.right || 0);
        if (total === 100) {
          panelWidths.value = parsed;
        }
      }
    } catch (e) {
      console.error("Failed to parse panel widths:", e);
    }
  }
});

// Save widths to localStorage
function saveWidths() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(panelWidths.value));
}

// Dragging state
const isDragging = ref(false);
const startX = ref(0);
const startWidths = ref({ left: 0, right: 0 });

// Calculate panel styles
const leftPanelStyle = computed(() => ({
  width: `${panelWidths.value.left}%`,
  minWidth: "20%",
  maxWidth: "80%",
}));

const rightPanelStyle = computed(() => ({
  width: `${panelWidths.value.right}%`,
  minWidth: "20%",
  maxWidth: "80%",
}));

// Divider style
const dividerStyle = computed(() => ({
  left: `${panelWidths.value.left}%`,
}));

// Start dragging divider
function startDrag(e: MouseEvent) {
  isDragging.value = true;
  startX.value = e.clientX;
  startWidths.value = { ...panelWidths.value };

  document.addEventListener("mousemove", onDrag);
  document.addEventListener("mouseup", stopDrag);

  e.preventDefault();
}

// Drag divider
function onDrag(e: MouseEvent) {
  if (!isDragging.value) return;

  const containerWidth =
    (e.target as HTMLElement).parentElement?.offsetWidth || window.innerWidth;
  const deltaX = e.clientX - startX.value;
  const deltaPercent = (deltaX / containerWidth) * 100;

  // Calculate new widths
  let newLeft = startWidths.value.left + deltaPercent;
  let newRight = startWidths.value.right - deltaPercent;

  // Constrain widths (min 20%, max 80%)
  newLeft = Math.max(20, Math.min(80, newLeft));
  newRight = Math.max(20, Math.min(80, newRight));

  panelWidths.value = { left: newLeft, right: newRight };
}

// Stop dragging divider
function stopDrag() {
  if (isDragging.value) {
    isDragging.value = false;
    saveWidths();
  }
  document.removeEventListener("mousemove", onDrag);
  document.removeEventListener("mouseup", stopDrag);
}

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

// Cleanup event listeners on unmount
onUnmounted(() => {
  document.removeEventListener("mousemove", onDrag);
  document.removeEventListener("mouseup", stopDrag);
});
</script>

<template>
  <div class="papers-page">
    <!-- Two Panel Layout -->
    <div class="panels-container">
      <!-- Left Panel: Document List -->
      <div class="panel left-panel" :style="leftPanelStyle">
        <div class="panel-header">
          <v-toolbar density="compact" color="surface" class="pa-0">
            <v-toolbar-title class="text-subtitle-2">{{
              t("main.documents")
            }}</v-toolbar-title>
            <v-spacer />

            <v-btn
              icon="mdi-label-plus"
              size="small"
              @click="showAddTag"
              :title="t('dialog.addLabel')"
            >
              <v-icon size="small">mdi-label-plus</v-icon>
            </v-btn>

            <v-btn
              icon="mdi-folder-plus"
              size="small"
              @click="showAddCategory"
              :title="t('dialog.addCategory')"
            >
              <v-icon size="small">mdi-folder-plus</v-icon>
            </v-btn>

            <v-menu>
              <template #activator="{ props: menuProps }">
                <v-btn
                  icon="mdi-plus"
                  size="small"
                  v-bind="menuProps"
                  :title="t('toolbar.import')"
                >
                  <v-icon size="small">mdi-plus</v-icon>
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
        </div>
        <div class="panel-content scrollable">
          <DocumentList
            ref="documentListRef"
            :category-path="props.selectedCategory"
            @paper-select="handlePaperSelect"
          />
        </div>
      </div>

      <!-- Divider (Drag Handle) -->
      <div
        class="divider"
        :class="{ dragging: isDragging }"
        :style="dividerStyle"
        @mousedown="startDrag"
      >
        <div class="divider-handle"></div>
      </div>

      <!-- Right Panel: Document Details -->
      <div class="panel right-panel" :style="rightPanelStyle">
        <div class="panel-header">
          <span class="text-caption font-weight-medium px-4 py-2 d-block"
            >Details</span
          >
        </div>
        <div class="panel-content scrollable">
          <DocumentDetails :paper-id="selectedPaperId" />
        </div>
      </div>
    </div>

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
  </div>
</template>

<style scoped>
.papers-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.panels-container {
  display: flex;
  height: 100%;
  position: relative;
}

.panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  flex-shrink: 0;
}

.panel-header {
  flex-shrink: 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.12);
}

.panel-content {
  flex: 1;
  overflow: hidden;
}

.panel-content.scrollable {
  overflow-y: auto;
  overflow-x: hidden;
}

/* Divider (Drag Handle) */
.divider {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 4px;
  cursor: col-resize;
  z-index: 10;
  background: transparent;
}

.divider:hover,
.divider.dragging {
  background: rgb(var(--v-theme-primary));
}

.divider-handle {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.divider-handle::before {
  content: "";
  width: 2px;
  height: 24px;
  background: rgba(255, 255, 255, 0.3);
  border-radius: 1px;
}

.divider:hover .divider-handle::before,
.divider.dragging .divider-handle::before {
  background: rgb(var(--v-theme-on-primary));
}

/* Panel borders */
.left-panel {
  border-right: 1px solid rgba(255, 255, 255, 0.12);
}

.right-panel {
  /* No border on rightmost panel */
}

/* Disable Vuetify transitions in this component */
* {
  transition: none !important;
  animation-duration: 0s !important;
  animation-delay: 0s !important;
}
</style>
