<script setup lang="ts">
import { ref, onMounted } from "vue";
import { Draggable } from "@he-tree/vue";
import "@he-tree/vue/style/default.css";
import { invokeCommand } from "@/lib/tauri";
import { useI18n } from "@/lib/i18n";
import AddCategoryDialog from "@/components/dialogs/AddCategoryDialog.vue";
import EditCategoryDialog from "@/components/dialogs/EditCategoryDialog.vue";

interface CategoryDto {
  id: number;
  path: string;
  name: string;
  parent_id: number | null;
  sort_order: number;
}

interface CategoryNode {
  id: number;
  path: string;
  name: string;
  parent_id: number | null;
  sort_order: number;
  children?: CategoryNode[];
}

interface Props {
  onCategorySelect?: (path: string | null) => void;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  categorySelect: [path: string | null];
}>();

const { t } = useI18n();

const treeData = ref<CategoryNode[]>([]);
const loading = ref(false);
const selectedPath = ref<string | null>(null);
const errorMsg = ref<string | null>(null);
const rawCount = ref(0);

// Context menu state
const contextMenu = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);
const selectedNode = ref<CategoryNode | null>(null);

// Dialog states
const showAddCategoryDialog = ref(false);
const showEditCategoryDialog = ref(false);
const editingNodePath = ref("");
const editingNodeName = ref("");

// Load categories from backend
async function loadCategories() {
  loading.value = true;
  errorMsg.value = null;
  try {
    const data = await invokeCommand<CategoryDto[]>("load_categories");
    console.log("Loaded categories:", data);
    rawCount.value = data.length;
    treeData.value = buildCategoryTree(data);
    console.log("Built tree:", treeData.value);
  } catch (error) {
    console.error("Failed to load categories:", error);
    errorMsg.value = String(error);
  } finally {
    loading.value = false;
  }
}

// Build tree structure from flat list using ltree path
function buildCategoryTree(flat: CategoryDto[]): CategoryNode[] {
  if (flat.length === 0) return [];

  // Sort by path to ensure parents come before children
  const sorted = [...flat].sort((a, b) => a.path.localeCompare(b.path));

  // Map to store all nodes
  const nodeMap = new Map<string, CategoryNode>();

  // First pass: create all nodes
  sorted.forEach((dto) => {
    nodeMap.set(dto.path, {
      id: dto.id,
      path: dto.path,
      name: dto.name,
      parent_id: dto.parent_id,
      sort_order: dto.sort_order,
      children: [],
    });
  });

  // Second pass: build tree structure
  const root: CategoryNode[] = [];
  sorted.forEach((dto) => {
    const node = nodeMap.get(dto.path)!;

    // Check if this is a root node (path doesn't contain dots or only one level)
    const pathParts = dto.path.split(".");
    if (pathParts.length === 1) {
      // Root node
      root.push(node);
    } else {
      // Child node - find parent
      const parentPath = pathParts.slice(0, -1).join(".");
      const parent = nodeMap.get(parentPath);
      if (parent) {
        parent.children = parent.children || [];
        parent.children.push(node);
      } else {
        // Parent not found, treat as root
        console.warn(`Parent not found for path: ${dto.path}`);
        root.push(node);
      }
    }
  });

  return root;
}

// Handle node selection
function handleNodeClick(node: CategoryNode) {
  selectedPath.value = node.path;
  emit("categorySelect", node.path);
}

// Handle "All Papers" click
function handleAllPapersClick() {
  selectedPath.value = null;
  emit("categorySelect", null);
}

// Get node icon
function getNodeIcon(node: CategoryNode): string {
  const hasChildren = node.children && node.children.length > 0;
  return hasChildren ? "mdi-folder" : "mdi-folder-outline";
}

// Handle right-click on node
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
  editingNodePath.value = selectedNode.value.path;
  editingNodeName.value = selectedNode.value.name;
  showEditCategoryDialog.value = true;
  hideContextMenu();
}

async function handleDeleteCategory() {
  if (!selectedNode.value) return;

  if (
    !confirm(
      `确定要删除分类"${selectedNode.value.name}"及其所有子分类吗？此操作不可恢复。`,
    )
  ) {
    return;
  }

  try {
    await invokeCommand("delete_category", { path: selectedNode.value.path });
    await loadCategories();
  } catch (error) {
    console.error("Failed to delete category:", error);
    alert(`删除失败: ${error}`);
  }
  hideContextMenu();
}

// Add root category
function handleAddRootCategory() {
  selectedNode.value = null;
  showAddCategoryDialog.value = true;
}

// Handle dialog operations
async function handleCategoryCreated() {
  showAddCategoryDialog.value = false;
  await loadCategories();
}

async function handleCategoryUpdated() {
  showEditCategoryDialog.value = false;
  await loadCategories();
}

// Convert CategoryNode to TreeNodeDto (for backend)
function convertToTreeNode(node: CategoryNode): any {
  return {
    id: node.id,
    name: node.name,
    children:
      node.children && node.children.length > 0
        ? node.children.map(convertToTreeNode)
        : undefined,
  };
}

// Handle drag end - save to database
async function handleAfterDrop() {
  console.log("=== Drag After Drop Started ===");

  // After drag, he-tree has already updated treeData
  // We just need to send the new structure to backend
  const treeStructure = treeData.value.map(convertToTreeNode);
  console.log("Sending tree structure to backend:", treeStructure);

  try {
    await invokeCommand("reorder_tree", {
      treeData: treeStructure,
    });
    console.info("✓ Tree reordered successfully in database");

    // Reload to ensure sync with database
    await loadCategories();

    console.log("=== Drag After Drop Completed Successfully ===");
  } catch (error) {
    console.error("✗ Failed to reorder tree in database:", error);
    console.log("Reloading to revert local changes...");
    await loadCategories();
    console.log("=== Drag After Drop Failed ===");
  }
}

// Handle drag end
function handleDragEnd() {
  console.log("=== Drag End ===");
}

// Load categories on mount
onMounted(() => {
  loadCategories();
});

// Expose loadCategories function for parent component refresh
defineExpose({
  loadCategories,
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

    <!-- Error state -->
    <div v-if="errorMsg" class="tree-error">
      <v-alert type="error" density="compact">
        {{ errorMsg }}
        <template #append>
          <v-btn size="small" @click="loadCategories">重试</v-btn>
        </template>
      </v-alert>
    </div>

    <!-- Empty state with add button -->
    <div
      v-if="!loading && treeData.length === 0 && !errorMsg"
      class="tree-empty"
    >
      <span class="text-caption text-grey mb-2">暂无分类</span>
      <v-btn
        size="small"
        variant="outlined"
        block
        @click="handleAddRootCategory"
      >
        <v-icon size="small" start>mdi-plus</v-icon>
        新建分类
      </v-btn>
    </div>

    <!-- Tree component -->
    <Draggable
      v-if="!loading && treeData.length > 0"
      v-model="treeData"
      class="he-tree-wrapper"
      :indent="20"
      :default-open="true"
      draggable
      @drag-end="handleDragEnd"
      @after-drop="handleAfterDrop"
    >
      <template #default="{ node }">
        <div
          class="tree-node-content"
          :class="{ 'tree-node-selected': selectedPath === node.path }"
          @click="handleNodeClick(node)"
          @contextmenu="showContextMenu($event, node)"
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
          <v-list-item-title>{{
            t("dialog.addSubcategory")
          }}</v-list-item-title>
        </v-list-item>
        <v-list-item @click="handleEditCategory">
          <template #prepend>
            <v-icon>mdi-pencil</v-icon>
          </template>
          <v-list-item-title>{{ t("dialog.rename") }}</v-list-item-title>
        </v-list-item>
        <v-divider />
        <v-list-item @click="handleDeleteCategory">
          <template #prepend>
            <v-icon color="error">mdi-delete</v-icon>
          </template>
          <v-list-item-title>{{ t("dialog.delete") }}</v-list-item-title>
        </v-list-item>
      </v-list>
    </v-menu>

    <!-- Add Category Dialog -->
    <AddCategoryDialog
      v-model="showAddCategoryDialog"
      :parent-path="selectedNode?.path"
      @category-created="handleCategoryCreated"
    />

    <!-- Edit Category Dialog -->
    <EditCategoryDialog
      v-model="showEditCategoryDialog"
      :category-path="editingNodePath"
      :category-name="editingNodeName"
      @category-updated="handleCategoryUpdated"
    />
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
  margin-bottom: 8px;
}

.all-papers:hover {
  background-color: rgba(255, 255, 255, 0.08);
}

.tree-loading,
.tree-empty,
.tree-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 20px;
  color: rgba(255, 255, 255, 0.5);
  gap: 8px;
}

/* he-tree overrides */
:deep(.he-tree) {
  background: transparent;
}

:deep(.tree-node) {
  padding: 0;
}

/* Disable transitions for non-drag elements */
.tree-node-content,
.all-papers {
  transition: background-color 150ms ease;
}
</style>
