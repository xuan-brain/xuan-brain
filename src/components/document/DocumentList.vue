<script setup lang="ts">
import { ref, computed, watch, onMounted, reactive } from "vue";
import { useRouter } from "vue-router";
import { invokeCommand } from "@/lib/tauri";
import { VxeTable, VxeColumn, VxeToolbar } from "vxe-table";
import "vxe-table/lib/style.css";

const router = useRouter();

interface Label {
  id: number;
  name: string;
  color: string;
}

interface PaperDto {
  id: number;
  title: string;
  authors: string[];
  publication_year?: number;
  journal_name?: string;
  labels: Label[];
  attachment_count?: number;
}

interface Props {
  categoryPath?: string | null;
  currentView?: "library" | "favorites" | "trash";
}

const props = withDefaults(defineProps<Props>(), {
  categoryPath: null,
  currentView: "library",
});

const emit = defineEmits<{
  paperSelect: [paperId: number];
}>();

// State
const loading = ref(false);
const papers = ref<PaperDto[]>([]);
const selectedRowIds = ref<number[]>([]);

// Table ref
const tableRef = ref<VxeTable>();

// Sort configuration
const sortConfig = reactive({
  defaultSort: {
    field: "" as string,
    order: "" as "asc" | "desc",
  },
});

// Table columns definition
const columns = computed(() => [
  { field: "title", title: "Title", sortable: true, minWidth: 300 },
  { field: "authors", title: "Authors", sortable: false, width: 200 },
  { field: "publication_year", title: "Year", sortable: true, width: 100 },
  { field: "journal_name", title: "Journal", sortable: true, width: 200 },
  { field: "labels", title: "Labels", sortable: false, width: 200 },
]);

// Load papers from backend based on current view
async function loadPapers() {
  loading.value = true;
  try {
    let data: PaperDto[];

    if (props.currentView === "trash") {
      // Load deleted papers
      data = await invokeCommand<PaperDto[]>("get_deleted_papers");
    } else {
      // Load all papers (library view)
      // TODO: implement category filtering later
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
function handleSortChange({
  sortList,
}: {
  sortList: Array<{ field: string; order: string }>;
}) {
  if (sortList.length > 0) {
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

// Handle selection change
function handleCheckboxChange({
  checked,
  row,
}: {
  checked: boolean;
  row: PaperDto;
}) {
  if (checked) {
    if (!selectedRowIds.value.includes(row.id)) {
      selectedRowIds.value.push(row.id);
    }
  } else {
    const index = selectedRowIds.value.indexOf(row.id);
    if (index > -1) {
      selectedRowIds.value.splice(index, 1);
    }
  }
}

// Handle select all
function handleSelectAll({ checked }: { checked: boolean }) {
  if (checked) {
    selectedRowIds.value = papers.value.map((p) => p.id);
  } else {
    selectedRowIds.value = [];
  }
}

// Check if row is selected
function isRowSelected(row: PaperDto) {
  return selectedRowIds.value.includes(row.id);
}

// Watch category path changes
watch(
  () => props.categoryPath,
  () => {
    loadPapers();
  },
);

// Watch current view changes
watch(
  () => props.currentView,
  () => {
    // Clear selection when view changes
    selectedRowIds.value = [];
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

    <div class="table-container">
      <vxe-table
        ref="tableRef"
        :data="papers"
        :loading="loading"
        :checkbox-config="{ checkField: 'checked' }"
        :sort-config="{
          trigger: 'cell',
          defaultSort: sortConfig.defaultSort.field
            ? {
                field: sortConfig.defaultSort.field,
                order: sortConfig.defaultSort.order,
              }
            : undefined,
        }"
        :row-config="{ isCurrent: true, isHover: true }"
        height="100%"
        stripe
        border
        resizable
        @cell-click="handleCellClick"
        @row-dblclick="handleRowDblclick"
        @sort-change="handleSortChange"
        @checkbox-change="handleCheckboxChange"
        @checkbox-all="handleSelectAll"
      >
        <!-- Checkbox column -->
        <vxe-column type="checkbox" width="50" fixed="left" />

        <!-- Title column -->
        <vxe-column
          field="title"
          title="Title"
          min-width="300"
          sortable
          show-overflow
        >
          <template #default="{ row }">
            <span class="text-truncate">{{ row.title }}</span>
          </template>
        </vxe-column>

        <!-- Authors column -->
        <vxe-column field="authors" title="Authors" width="200" show-overflow>
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
        <vxe-column
          field="publication_year"
          title="Year"
          width="100"
          sortable
        />

        <!-- Journal column -->
        <vxe-column
          field="journal_name"
          title="Journal"
          width="200"
          sortable
          show-overflow
        />

        <!-- Labels column -->
        <vxe-column field="labels" title="Labels" width="200" show-overflow>
          <template #default="{ row }">
            <v-chip
              v-for="label in row.labels"
              :key="label.id"
              size="x-small"
              :color="label.color"
              class="mr-1"
            >
              {{ label.name }}
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
  padding: 8px;
}

:deep(.vxe-table .vxe-header--column) {
  padding: 8px;
  font-weight: 600;
}
</style>
