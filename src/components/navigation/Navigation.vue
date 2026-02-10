<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invokeCommand } from "@/lib/tauri";
import { useI18n } from "@/lib/i18n";
import AddCategoryDialog from "@/components/dialogs/AddCategoryDialog.vue";
import EditCategoryDialog from "@/components/dialogs/EditCategoryDialog.vue";

const { t } = useI18n();

interface CategoryNode {
  id: number;
  name: string;
  path: string;
  parent_id: number | null;
  children?: CategoryNode[];
}

interface Label {
  id: number;
  name: string;
  color: string;
}

// State
const categories = ref<CategoryNode[]>([]);
const labels = ref<Label[]>([]);
const loading = ref(false);
const selectedCategory = ref<string | null>(null);
const expandedCategories = ref<string[]>([]);

// Context menu state
const contextMenu = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);
const selectedNode = ref<CategoryNode | null>(null);

// Dialog states
const showAddCategoryDialog = ref(false);
const showEditCategoryDialog = ref(false);
const editingCategoryId = ref<number | null>(null);
const editingCategoryName = ref<string>("");

// Emit events
const emit = defineEmits<{
  categorySelect: [path: string | null];
}>();

// Load categories from backend
async function loadCategories() {
  loading.value = true;
  try {
    const data = await invokeCommand<CategoryNode[]>("load_categories");
    categories.value = buildCategoryTree(data);
    // Expand all categories by default
    expandedCategories.value = getAllNodePaths(categories.value);
  } catch (error) {
    console.error("Failed to load categories:", error);
  } finally {
    loading.value = false;
  }
}

// Load labels from backend
async function loadLabels() {
  try {
    labels.value = await invokeCommand<Label[]>("get_all_labels");
  } catch (error) {
    console.error("Failed to load labels:", error);
  }
}

// Build tree structure from flat array
function buildCategoryTree(flat: CategoryNode[]): CategoryNode[] {
  const map = new Map<number, CategoryNode>();

  // Create map of all nodes
  flat.forEach((node) => {
    map.set(node.id, { ...node, children: [] });
  });

  // Build tree structure
  const root: CategoryNode[] = [];
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

// Get all node paths for default expansion
function getAllNodePaths(nodes: CategoryNode[]): string[] {
  const paths: string[] = [];

  function traverse(node: CategoryNode) {
    paths.push(node.path);
    if (node.children) {
      node.children.forEach(traverse);
    }
  }

  nodes.forEach(traverse);
  return paths;
}

// Handle category selection
function handleCategorySelect(path: string | null) {
  selectedCategory.value = path;
  emit("categorySelect", path);
}

// Handle right-click on category
function showContextMenu(event: MouseEvent, node: CategoryNode) {
  event.preventDefault();
  event.stopPropagation();
  selectedNode.value = node;
  contextMenuX.value = event.clientX;
  contextMenuY.value = event.clientY;
  contextMenu.value = true;
}

// Hide context menu
function hideContextMenu() {
  contextMenu.value = false;
}

// Context menu actions
function handleAddSubcategory() {
  if (!selectedNode.value) return;
  showAddCategoryDialog.value = true;
  hideContextMenu();
}

function handleEditCategory() {
  if (!selectedNode.value) return;
  editingCategoryId.value = selectedNode.value.id;
  editingCategoryName.value = selectedNode.value.name;
  showEditCategoryDialog.value = true;
  hideContextMenu();
}

async function handleDeleteCategory() {
  if (!selectedNode.value) return;

  try {
    await invokeCommand("delete_category", { id: selectedNode.value.id });
    await loadCategories();
  } catch (error) {
    console.error("Failed to delete category:", error);
  }
  hideContextMenu();
}

// Refresh after dialog operations
async function refreshCategories() {
  await loadCategories();
}

// Initialize on mount
onMounted(() => {
  loadCategories();
  loadLabels();
});

// Close context menu when clicking elsewhere
function handleGlobalClick() {
  hideContextMenu();
}
</script>

<template>
  <div class="navigation" @click="handleGlobalClick">
    <!-- Categories Section -->
    <div class="nav-section">
      <div class="section-header">
        <span class="text-caption text-grey">{{ t("main.categories") }}</span>
      </div>

      <!-- All Papers Entry -->
      <v-list-item
        :class="{ 'v-list-item--active': selectedCategory === null }"
        @click="handleCategorySelect(null)"
      >
        <template #prepend>
          <v-icon>mdi-folder-open</v-icon>
        </template>
        <v-list-item-title>All Papers</v-list-item-title>
      </v-list-item>

      <v-divider />

      <!-- Category Tree with context menu -->
      <v-treeview
        v-model:opened="expandedCategories"
        :items="categories"
        item-value="path"
        item-children="children"
        item-title="name"
        density="compact"
        hover
        class="category-tree"
        @update:activated="
          (value: unknown) => {
            const items = Array.isArray(value) ? value : [value];
            if (items.length > 0) handleCategorySelect(items[0] as string);
          }
        "
      >
        <template #prepend="{ item }">
          <div @contextmenu="(e: MouseEvent) => showContextMenu(e, item)">
            <v-icon>mdi-folder</v-icon>
          </div>
        </template>

        <template #title="{ item }">
          <div @contextmenu="(e: MouseEvent) => showContextMenu(e, item)">
            {{ item.name }}
          </div>
        </template>
      </v-treeview>
    </div>

    <!-- Labels Section -->
    <div class="nav-section">
      <v-divider />
      <div class="section-header">
        <span class="text-caption text-grey">Labels</span>
      </div>

      <v-list density="compact">
        <v-list-item
          v-for="label in labels"
          :key="label.id"
          @click="handleCategorySelect(null)"
        >
          <template #prepend>
            <v-icon :color="label.color">mdi-label</v-icon>
          </template>
          <v-list-item-title>{{ label.name }}</v-list-item-title>
        </v-list-item>
      </v-list>
    </div>

    <!-- Context Menu -->
    <v-menu
      v-model="contextMenu"
      :close-on-content-click="false"
      :style="{ top: contextMenuY + 'px', left: contextMenuX + 'px' }"
      absolute
    >
      <v-list density="compact">
        <v-list-item @click="handleAddSubcategory">
          <template #prepend>
            <v-icon>mdi-plus</v-icon>
          </template>
          <v-list-item-title>Add Subcategory</v-list-item-title>
        </v-list-item>
        <v-list-item @click="handleEditCategory">
          <template #prepend>
            <v-icon>mdi-pencil</v-icon>
          </template>
          <v-list-item-title>Edit</v-list-item-title>
        </v-list-item>
        <v-list-item @click="handleDeleteCategory">
          <template #prepend>
            <v-icon color="error">mdi-delete</v-icon>
          </template>
          <v-list-item-title>Delete</v-list-item-title>
        </v-list-item>
      </v-list>
    </v-menu>

    <!-- Add Category Dialog -->
    <AddCategoryDialog
      v-model="showAddCategoryDialog"
      :parent-path="selectedNode?.path"
      @category-created="refreshCategories"
    />

    <!-- Edit Category Dialog -->
    <EditCategoryDialog
      v-model="showEditCategoryDialog"
      :category-id="editingCategoryId"
      :category-name="editingCategoryName"
      @category-updated="refreshCategories"
    />
  </div>
</template>

<style scoped>
.navigation {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.nav-section {
  flex: 0 0 auto;
}

.section-header {
  padding: 8px 16px;
}

.category-tree {
  max-height: 400px;
  overflow-y: auto;
}

:deep(.v-treeview-item--active) {
  background-color: rgb(var(--v-theme-primary));
  color: rgb(var(--v-theme-on-primary));
}
</style>
