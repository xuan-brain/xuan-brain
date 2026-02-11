<script setup lang="ts">
import { ref, computed, watch, onMounted, reactive } from "vue";
import { useRouter } from "vue-router";
import { invokeCommand } from "@/lib/tauri";
import type { VxeTablePropTypes } from "vxe-table";
import DocumentToolbar from "./DocumentToolbar.vue";
import { useI18n } from "@/lib/i18n";
import { open } from "@tauri-apps/plugin-dialog";

const { t } = useI18n();
const router = useRouter();

interface Label {
  id: number;
  name: string;
  color: string;
}

interface Attachment {
  id: number;
  paper_id: number;
  file_name: string | null;
  file_type: string | null;
  created_at: string | null;
}

interface PaperDto {
  id: number;
  title: string;
  authors: string[];
  publication_year?: number;
  journal_name?: string;
  labels: Label[];
  attachment_count?: number;
  attachments?: Attachment[];
}

interface Props {
  categoryId?: number | null;
  currentView?: "library" | "favorites" | "trash";
}

const props = withDefaults(defineProps<Props>(), {
  categoryId: null,
  currentView: "library",
});

const emit = defineEmits<{
  paperSelect: [paperId: number];
}>();

// State
const loading = ref(false);
const papers = ref<PaperDto[]>([]);

// Table ref

// Expand configuration
const expandRowIds = ref<number[]>([]);

const expandConfig = computed<VxeTablePropTypes.ExpandConfig>(() => ({
  showIcon: true,
  trigger: "row",
  expandRowKeys: expandRowIds.value,
  accordion: true, // 手风琴模式：展开一行时其他行自动折叠
  visibleMethod: ({ row }) => {
    // 只对有附件的行显示展开图标
    return ((row as PaperDto).attachment_count ?? 0) > 0;
  },
  toggleMethod({ expanded, row }) {
    const paper = row as PaperDto;
    if (expanded) {
      if ((paper.attachment_count ?? 0) === 0) {
        return false; // 没有附件，禁止展开
      }
      return true;
    } else {
      if ((paper.attachment_count ?? 0) === 0) {
        return false; // 没有附件，禁止展开
      }
      return true; // 有附件，允许展开
    }
  },
}));

// Sort configuration
const sortConfig = reactive({
  defaultSort: {
    field: "" as string,
    order: "" as "asc" | "desc",
  },
});

// Context menu configuration
const contextMenuConfig = computed(() => {
  // Build menu items based on current view (must be 2D array)
  if (props.currentView === "trash") {
    // Trash view menu
    return {
      body: {
        options: [
          [
            {
              code: "restore",
              name: t("dialog.restore"),
              prefixConfig: { icon: "vxe-icon-undo" },
            },
            {
              code: "permanently_delete",
              name: t("dialog.permanentlyDelete"),
              prefixConfig: { icon: "vxe-icon-delete" },
            },
          ],
        ],
      },
    };
  } else {
    // Normal view menu
    return {
      body: {
        options: [
          [
            {
              code: "add_attachment",
              name: "添加附件",
              prefixConfig: { icon: "vxe-icon-file" },
            },
            {
              code: "open_folder",
              name: "打开附件文件夹",
              prefixConfig: { icon: "vxe-icon-folder-open" },
            },
          ],
          [
            {
              code: "delete",
              name: t("dialog.delete"),
              prefixConfig: { icon: "vxe-icon-folder-open" },
            },
          ],
        ],
      },
    };
  }
});

// Handle context menu click
async function handleContextMenuClick({ menu, row }: any) {
  const paper = row as PaperDto;
  console.info("Context menu clicked:", menu.code, paper);

  switch (menu.code) {
    case "add_attachment":
      await handleAddAttachment(paper);
      break;
    case "open_folder":
      await handleOpenFolder(paper);
      break;
    case "delete":
      await handleDeletePaper(paper);
      break;
    case "restore":
      await handleRestorePaper(paper);
      break;
    case "permanently_delete":
      await handlePermanentlyDeletePaper(paper);
      break;
  }
}

// Handle context menu shown (for debugging)
function handleContextMenuVisible({ type, row, column }: any) {
  console.info("Context menu shown:", { type, row, column });
}

// Add attachment to paper
async function handleAddAttachment(paper: PaperDto) {
  try {
    const selected = await open({
      multiple: false,
      directory: false,
    });
    if (selected) {
      const filePath = Array.isArray(selected) ? selected[0] : selected;
      if (filePath) {
        await invokeCommand("add_attachment", {
          paperId: paper.id,
          filePath: filePath,
        });
        // Reload papers to show updated attachment count
        await loadPapers();
        console.info("Attachment added successfully");
      }
    }
  } catch (error) {
    console.error("Failed to add attachment:", error);
  }
}

// Open paper folder
async function handleOpenFolder(paper: PaperDto) {
  try {
    await invokeCommand("open_paper_folder", { paperId: paper.id });
  } catch (error) {
    console.error("Failed to open folder:", error);
  }
}

// Delete paper (move to trash)
async function handleDeletePaper(paper: PaperDto) {
  try {
    await invokeCommand("delete_paper", { id: paper.id });
    await loadPapers();
  } catch (error) {
    console.error("Failed to delete paper:", error);
  }
}

// Restore paper from trash
async function handleRestorePaper(paper: PaperDto) {
  try {
    await invokeCommand("restore_paper", { id: paper.id });
    await loadPapers();
  } catch (error) {
    console.error("Failed to restore paper:", error);
  }
}

// Permanently delete paper
async function handlePermanentlyDeletePaper(paper: PaperDto) {
  try {
    await invokeCommand("permanently_delete_paper", { id: paper.id });
    await loadPapers();
  } catch (error) {
    console.error("Failed to permanently delete paper:", error);
  }
}

// Table columns definition

// Load papers from backend based on current view
async function loadPapers() {
  loading.value = true;
  try {
    let data: PaperDto[];

    if (props.currentView === "trash") {
      // Load deleted papers
      data = await invokeCommand<PaperDto[]>("get_deleted_papers");
    } else if (props.categoryId) {
      // Load papers for specific category
      data = await invokeCommand<PaperDto[]>("get_papers_by_category", {
        categoryId: props.categoryId,
      });
    } else {
      // Load all papers (library view)
      data = await invokeCommand<PaperDto[]>("get_all_papers");
    }

    papers.value = data;
  } catch (error) {
    console.error("Failed to load papers:", error);
  } finally {
    loading.value = false;
  }
}

// Handle row click - emit paper selection
function handleCellClick({ row }: { row: PaperDto }) {
  emit("paperSelect", row.id);
}

// Handle row double click - navigate to paper detail
function handleRowDblclick({ row }: { row: PaperDto }) {
  router.push(`/papers/${row.id}`);
}

// Handle sort change
function handleSortChange(params: any) {
  const { sortList } = params;
  if (sortList && sortList.length > 0) {
    const { field, order } = sortList[0];
    sortConfig.defaultSort.field = field;
    sortConfig.defaultSort.order = order as "asc" | "desc";

    // Sort papers locally
    papers.value.sort((a, b) => {
      const aVal = (a as any)[field];
      const bVal = (b as any)[field];

      if (aVal === undefined || aVal === null) return 1;
      if (bVal === undefined || bVal === null) return -1;
      if (aVal < bVal) return order === "asc" ? -1 : 1;
      if (aVal > bVal) return order === "asc" ? 1 : -1;
      return 0;
    });
  }
}

// Handle expand row toggle
function handleToggleExpandChange({ row }: { row: any }) {
  const index = expandRowIds.value.indexOf(row.id);

  console.info(
    "Toggle expand clicked for row:",
    row.id,
    "current expanded rows:",
    expandRowIds.value,
  );

  if (index > -1) {
    // Collapse
    expandRowIds.value.splice(index, 1);
    console.info("Collapsed row:", row.id);
  } else {
    // Expand - attachments are already loaded from backend
    expandRowIds.value.push(row.id);
    console.info(
      "Expanded row:",
      row.id,
      "attachment_count:",
      row.attachment_count,
      "attachments:",
      row.attachments?.length || 0,
    );
  }
}

// Handle expand row change event (fallback)

// Check if row is expanded

// Get file icon based on file type
function getFileIcon(fileType: string | null): string {
  if (!fileType) return "mdi-file";
  const type = fileType.toLowerCase();
  if (type.includes("pdf")) return "mdi-file-pdf-box";
  if (type.includes("doc") || type.includes("word")) return "mdi-file-word-box";
  if (
    type.includes("xls") ||
    type.includes("excel") ||
    type.includes("spreadsheet")
  )
    return "mdi-file-excel-box";
  if (type.includes("ppt") || type.includes("powerpoint"))
    return "mdi-file-powerpoint-box";
  if (type.includes("txt")) return "mdi-file-document-box";
  if (type.includes("zip") || type.includes("rar") || type.includes("archive"))
    return "mdi-file-zip-box";
  return "mdi-file";
}

// Watch category ID changes
watch(
  () => props.categoryId,
  () => {
    loadPapers();
  },
);

// Watch current view changes
watch(
  () => props.currentView,
  () => {
    // Clear expanded rows when view changes
    expandRowIds.value = [];
    loadPapers();
  },
);

// Load on mount
onMounted(() => {
  loadPapers();
});

// Expose load function for parent component refresh
defineExpose({
  loadPapers,
});
</script>

<template>
  <div class="document-list">
    <!-- View indicator -->
    <div v-if="currentView === 'trash'" class="view-indicator">
      <v-icon size="small" color="warning" class="mr-2">mdi-delete</v-icon>
      <span class="text-caption">{{ $t("navigation.trash") }}</span>
    </div>

    <!-- Toolbar -->
    <DocumentToolbar
      :on-refresh="loadPapers"
      :selected-category-id="categoryId"
    />

    <div class="table-container">
      <!-- Loading overlay -->
      <div v-if="loading" class="loading-overlay">
        <v-progress-circular indeterminate size="48" />
      </div>

      <vxe-table
        ref="tableRef"
        :data="papers"
        :expand-config="expandConfig"
        :column-config="{ resizable: true }"
        :menu-config="contextMenuConfig"
        :sort-config="{
          trigger: 'cell',
          defaultSort: sortConfig.defaultSort.field
            ? {
                field: sortConfig.defaultSort.field,
                order: sortConfig.defaultSort.order,
              }
            : undefined,
        }"
        :row-config="{ isCurrent: true, isHover: true, keyField: 'id' }"
        :cell-config="{ height: 32 }"
        :style="{
          '--vxe-ui-table-row-current-background-color':
            'rgba(var(--v-theme-primary), 0.2)',
          '--vxe-ui-table-row-hover-current-background-color':
            'rgba(var(--v-theme-primary), 0.3)',
        }"
        height="100%"
        stripe
        border
        size="mini"
        @cell-click="handleCellClick"
        @row-dblclick="handleRowDblclick"
        @sort-change="handleSortChange"
        @toggle-expand-change="handleToggleExpandChange"
        @menu-click="handleContextMenuClick"
        @menu-visible="handleContextMenuVisible"
      >
        <!-- Expand column (only for papers with attachments) -->
        <vxe-column type="expand" width="40" fixed="left">
          <template #content="{ row }">
            <div class="expand-row-content">
              <div
                v-if="row.attachments && row.attachments.length > 0"
                class="attachments-list"
              >
                <div class="attachments-header">
                  <v-icon size="small" class="mr-2">mdi-paperclip</v-icon>
                  <span class="text-subtitle-2"
                    >Attachments ({{ row.attachments.length }})</span
                  >
                </div>
                <v-list density="compact" class="attachments-list-items">
                  <v-list-item
                    v-for="attachment in row.attachments"
                    :key="attachment.id"
                    class="attachment-item"
                  >
                    <template #prepend>
                      <v-icon
                        :icon="getFileIcon(attachment.file_type)"
                        size="small"
                      />
                    </template>
                    <v-list-item-title>
                      {{ attachment.file_name || "Unnamed file" }}
                    </v-list-item-title>
                    <v-list-item-subtitle v-if="attachment.file_type">
                      {{ attachment.file_type }}
                      <span v-if="attachment.created_at">
                        •
                        {{
                          new Date(attachment.created_at).toLocaleDateString()
                        }}
                      </span>
                    </v-list-item-subtitle>
                  </v-list-item>
                </v-list>
              </div>
              <div v-else class="no-attachments">
                <v-icon size="small" class="mr-2">mdi-information</v-icon>
                <span class="text-caption">No attachments</span>
              </div>
            </div>
          </template>
        </vxe-column>

        <!-- Title column -->
        <vxe-column
          field="title"
          title="Title"
          min-width="200"
          sortable
          show-overflow
        >
          <template #default="{ row }">
            <span class="text-truncate">{{ row.title }}</span>
          </template>
        </vxe-column>

        <!-- Authors column -->
        <vxe-column field="authors" title="Authors" width="15%" show-overflow>
          <template #default="{ row }">
            <v-chip
              v-if="row.authors && row.authors.length > 0"
              size="x-small"
              class="mr-1"
            >
              {{ row.authors[0] }}
            </v-chip>
            <v-chip v-if="row.authors && row.authors.length > 1" size="x-small">
              +{{ row.authors.length - 1 }}
            </v-chip>
          </template>
        </vxe-column>

        <!-- Year column -->
        <vxe-column field="publication_year" title="Year" width="80" sortable />

        <!-- Journal column -->
        <vxe-column
          field="journal_name"
          title="Journal"
          width="20%"
          sortable
          show-overflow
        />

        <!-- Labels column -->
        <vxe-column field="labels" title="Labels" width="15%" show-overflow>
          <template #default="{ row }">
            <v-chip
              v-if="row.labels && row.labels.length > 0"
              size="x-small"
              :color="row.labels[0].color"
              class="mr-1"
            >
              {{ row.labels[0].name }}
            </v-chip>
            <v-chip v-if="row.labels && row.labels.length > 1" size="x-small">
              +{{ row.labels.length - 1 }}
            </v-chip>
          </template>
        </vxe-column>
      </vxe-table>
    </div>
  </div>
</template>

<style scoped>
.document-list {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.view-indicator {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  background-color: rgba(255, 152, 0, 0.1);
  border-bottom: 1px solid rgba(255, 255, 255, 0.12);
  flex-shrink: 0;
}

.table-container {
  flex: 1;
  overflow: hidden;
  position: relative;
}

.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: rgba(0, 0, 0, 0.5);
  z-index: 100;
}

.text-truncate {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

:deep(.vxe-table) {
  background-color: transparent;
}

:deep(.vxe-table--body .vxe-body--row.row--checked) {
  background-color: rgba(var(--v-theme-primary), 0.1);
}

:deep(.vxe-table--body .vxe-body--row:hover) {
  cursor: pointer;
}

:deep(.vxe-table .vxe-body--column) {
  padding: 4px 8px;
}

:deep(.vxe-table .vxe-header--column) {
  padding: 6px 8px;
  font-weight: 600;
  font-size: 13px;
}

:deep(.vxe-table--render-default .vxe-body--column) {
  font-size: 13px;
}

:deep(.vxe-table--render-default.size--mini .vxe-body--column) {
  padding: 2px 8px;
}

:deep(.vxe-table--render-default.size--mini .vxe-header--column) {
  padding: 4px 8px;
  font-size: 12px;
}

/* Expand row styles */
.expand-row-content {
  padding: 12px 16px;
  background-color: var(--vxe-table-body-background-color);
}

.attachments-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.attachments-header {
  display: flex;
  align-items: center;
  font-weight: 600;
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.12);
}

.attachments-list-items {
  background-color: transparent;
  padding: 0;
}

.attachment-item {
  padding: 4px 8px;
  min-height: auto;
}

.attachment-item :deep(.v-list-item-title) {
  font-size: 13px;
}

.attachment-item :deep(.v-list-item-subtitle) {
  font-size: 11px;
  opacity: 0.7;
}

.no-attachments {
  display: flex;
  align-items: center;
  opacity: 0.6;
  font-style: italic;
}

/* Expand icon rotation */
.rotate-90 {
  transform: rotate(90deg);
}

/* Expand row styling */
:deep(.vxe-table--expand-icon) {
  transition: transform 0.2s ease;
}

:deep(.vxe-body--row.row--expanded .vxe-body--expand-row) {
  background-color: var(--vxe-table-body-background-color);
}
</style>
