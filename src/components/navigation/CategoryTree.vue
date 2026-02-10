<script setup lang="ts">
import { ref, onMounted } from "vue";
import { Draggable } from "@he-tree/vue";
import "@he-tree/vue/style/default.css";
import { invokeCommand } from "@/lib/tauri";

interface CategoryNode {
  id: number;
  name: string;
  path: string;
  parent_id: number | null;
  children?: CategoryNode[];
  draggable?: boolean;
  droppable?: boolean;
}

interface Props {
  onCategorySelect?: (path: string | null) => void;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  categorySelect: [path: string | null];
}>();

const treeData = ref<CategoryNode[]>([]);
const loading = ref(false);
const selectedPath = ref<string | null>(null);

// Load categories from backend
async function loadCategories() {
  loading.value = true;
  try {
    const data = await invokeCommand<CategoryNode[]>("load_categories");
    treeData.value = buildCategoryTree(data);
  } catch (error) {
    console.error("Failed to load categories:", error);
  } finally {
    loading.value = false;
  }
}

// Build tree structure from flat list
function buildCategoryTree(flat: CategoryNode[]): CategoryNode[] {
  const map = new Map<number, CategoryNode>();
  const root: CategoryNode[] = [];

  // Create map of all nodes
  flat.forEach((node) => {
    map.set(node.id, {
      ...node,
      children: [],
      draggable: true,
      droppable: true,
    });
  });

  // Build tree structure
  flat.forEach((node) => {
    const current = map.get(node.id)!;
    if (node.parent_id === null) {
      root.push(current);
    } else {
      const parent = map.get(node.parent_id);
      if (parent) {
        parent.children = parent.children || [];
        parent.children.push(current);
      }
    }
  });

  return root;
}

// Handle node selection
function handleNodeClick(data: CategoryNode) {
  selectedPath.value = data.path;
  emit("categorySelect", data.path);
}

// Handle "All Papers" click
function handleAllPapersClick() {
  selectedPath.value = null;
  emit("categorySelect", null);
}

// Get node icon
function getNodeIcon(node: CategoryNode): string {
  if (node.children && node.children.length > 0) {
    return "mdi-folder";
  }
  return "mdi-folder-outline";
}

// Load categories on mount
onMounted(() => {
  loadCategories();
});
</script>

<template>
  <div class="category-tree">
    <!-- All Papers entry -->
    <div
      class="tree-node all-papers"
      :class="{ 'tree-node-selected': selectedPath === null }"
      @click="handleAllPapersClick"
    >
      <v-icon size="small" class="tree-node-icon">mdi-folder-open</v-icon>
      <span class="tree-node-text">All Papers</span>
    </div>

    <v-divider class="my-2" />

    <!-- Tree component -->
    <Draggable
      v-if="!loading && treeData.length > 0"
      v-model="treeData"
      class="he-tree-wrapper"
      :indent="20"
      :default-open="true"
    >
      <template #default="{ node }">
        <div
          class="tree-node-content"
          :class="{ 'tree-node-selected': selectedPath === node.path }"
          @click="handleNodeClick(node)"
        >
          <v-icon size="small" class="tree-node-folder">
            {{ getNodeIcon(node) }}
          </v-icon>
          <span class="tree-node-text">{{ node.name }}</span>
        </div>
      </template>
    </Draggable>

    <!-- Loading state -->
    <div v-if="loading" class="tree-loading">
      <v-progress-circular indeterminate size="24" />
      <span class="ml-2">Loading...</span>
    </div>

    <!-- Empty state -->
    <div v-if="!loading && treeData.length === 0" class="tree-empty">
      <span class="text-caption">No categories</span>
    </div>
  </div>
</template>

<style scoped>
@import "@he-tree/vue/style/default.css";

.category-tree {
  height: 100%;
  overflow-y: auto;
  padding: 8px 0;
}

.he-tree-wrapper {
  width: 100%;
}

.tree-node-content {
  display: flex;
  align-items: center;
  padding: 6px 8px;
  cursor: pointer;
  border-radius: 4px;
  user-select: none;
}

.tree-node-content:hover {
  background-color: rgba(255, 255, 255, 0.08);
}

.tree-node-selected {
  background-color: rgb(var(--v-theme-primary));
  color: rgb(var(--v-theme-on-primary));
}

.tree-node-icon {
  margin-right: 4px;
}

.tree-node-folder {
  margin-right: 8px;
}

.tree-node-text {
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.all-papers {
  display: flex;
  align-items: center;
  padding: 6px 8px;
  cursor: pointer;
  border-radius: 4px;
}

.all-papers:hover {
  background-color: rgba(255, 255, 255, 0.08);
}

.tree-loading,
.tree-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  color: rgba(255, 255, 255, 0.5);
}

/* he-tree overrides */
:deep(.he-tree) {
  background: transparent;
}

:deep(.tree-node) {
  padding: 0;
}

/* Disable transitions */
* {
  transition: none !important;
}
</style>
