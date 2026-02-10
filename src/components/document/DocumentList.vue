<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { useRouter } from "vue-router";
import { invokeCommand } from "@/lib/tauri";

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
}

const props = defineProps<Props>();

// State
const loading = ref(false);
const papers = ref<PaperDto[]>([]);
const totalItems = ref(0);
const selected = ref<PaperDto[]>([]);

// Server options for v-data-table-server
const serverOptions = ref({
  page: 1,
  itemsPerPage: 50,
  sortBy: [] as Array<{ key: string; order: "asc" | "desc" }>,
});

// Table headers
const headers = computed(() => [
  { title: "Title", key: "title", sortable: true },
  { title: "Authors", key: "authors", sortable: false },
  { title: "Year", key: "publication_year", sortable: true },
  { title: "Journal", key: "journal_name", sortable: true },
  { title: "Labels", key: "labels", sortable: false },
]);

// Load papers from backend
async function loadPapers() {
  loading.value = true;
  try {
    // For now, load all papers. Later implement pagination with backend
    const data = await invokeCommand<PaperDto[]>("get_all_papers");
    papers.value = data;
    totalItems.value = data.length;
  } catch (error) {
    console.error("Failed to load papers:", error);
  } finally {
    loading.value = false;
  }
}

// Handle row click - navigate to paper detail
function handleRowClick(event: MouseEvent, item: { item: PaperDto }) {
  // Double click to open detail
  if (event.detail === 2) {
    router.push(`/papers/${item.item.id}`);
  }
}

// Watch server options for changes (pagination, sorting)
watch(serverOptions, () => {
  loadPapers();
}, { deep: true });

// Watch category path changes
watch(() => props.categoryPath, () => {
  loadPapers();
});

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
    <v-data-table-server
      v-model:selected="selected"
      v-model:items-per-page="serverOptions.itemsPerPage"
      v-model:page="serverOptions.page"
      v-model:sort-by="serverOptions.sortBy"
      :headers="headers"
      :items="papers"
      :items-length="totalItems"
      :loading="loading"
      loading-text="Loading papers..."
      density="compact"
      hover
      show-select
      select-strategy="page"
      item-value="id"
      @click:row="handleRowClick"
    >
      <!-- Custom title rendering with truncation -->
      <template #item.title="{ item }">
        <span class="text-truncate d-block">{{ item.title }}</span>
      </template>

      <!-- Custom authors rendering with chips -->
      <template #item.authors="{ item }">
        <v-chip size="x-small" class="mr-1">
          {{ item.authors[0] }}
        </v-chip>
        <v-chip v-if="item.authors.length > 1" size="x-small">
          +{{ item.authors.length - 1 }}
        </v-chip>
      </template>

      <!-- Custom labels rendering with colored chips -->
      <template #item.labels="{ item }">
        <v-chip
          v-for="label in item.labels"
          :key="label.id"
          size="x-small"
          :color="label.color"
          class="mr-1"
        >
          {{ label.name }}
        </v-chip>
      </template>
    </v-data-table-server>
  </div>
</template>

<style scoped>
.document-list {
  height: 100%;
  overflow: auto;
}

:deep(.v-data-table__tr:hover) {
  cursor: pointer;
}
</style>
