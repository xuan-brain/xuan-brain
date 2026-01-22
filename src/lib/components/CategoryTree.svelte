<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Tree } from "@keenmate/svelte-treeview";
  import {
    Folder,
    FolderOpen,
    Plus,
    SquarePen,
    Trash2,
    Pencil,
  } from "lucide-svelte";
  import AddCategoryDialog from "./AddCategoryDialog.svelte";
  import EditCategoryDialog from "./EditCategoryDialog.svelte";

  // Category data structure
  interface CategoryNode {
    path: string;
    name: string;
    parent_id?: number;
    sort_order: number;
    isDraggable?: boolean;
  }

  // State using Svelte 5 runes
  let categories = $state<CategoryNode[]>([]);
  let selectedPath = $state<string | null>(null);
  let contextMenuNode = $state<{
    visible: boolean;
    x: number;
    y: number;
    node: CategoryNode | null;
  }>({
    visible: false,
    x: 0,
    y: 0,
    node: null,
  });

  // Dialog states
  let showAddDialog = $state(false);
  let showEditDialog = $state(false);
  let addDialogParentPath = $state<string | undefined>(undefined);
  let editDialogNode = $state<{ path: string; name: string } | null>(null);

  // Load categories from backend
  async function loadCategories() {
    try {
      console.log("Loading categories from backend...");
      const data = await invoke<CategoryNode[]>("load_categories");
      console.log("Received categories:", data);

      // Enable drag for all categories
      categories = data.map((cat) => ({
        ...cat,
        isDraggable: true,
      }));
    } catch (error) {
      console.error("Failed to load categories:", error);
      // Use demo data as fallback
      categories = [
        {
          path: "1",
          name: "计算机科学",
          parent_id: undefined,
          sort_order: 0,
          isDraggable: true,
        },
        {
          path: "1.1",
          name: "人工智能",
          parent_id: 1,
          sort_order: 0,
          isDraggable: true,
        },
        {
          path: "1.1.1",
          name: "机器学习",
          parent_id: 2,
          sort_order: 0,
          isDraggable: true,
        },
        {
          path: "1.1.2",
          name: "深度学习",
          parent_id: 2,
          sort_order: 0,
          isDraggable: true,
        },
        {
          path: "1.2",
          name: "数据库",
          parent_id: 1,
          sort_order: 0,
          isDraggable: true,
        },
        {
          path: "2",
          name: "物理学",
          parent_id: undefined,
          sort_order: 0,
          isDraggable: true,
        },
        {
          path: "2.1",
          name: "量子力学",
          parent_id: 6,
          sort_order: 0,
          isDraggable: true,
        },
      ];
    }
  }

  // Load on mount
  onMount(() => {
    loadCategories();

    // Close context menu on click outside
    const handleClickOutside = () => {
      if (contextMenuNode.visible) {
        contextMenuNode.visible = false;
      }
    };

    document.addEventListener("click", handleClickOutside);
    return () => {
      document.removeEventListener("click", handleClickOutside);
    };
  });

  // Handle node click
  function handleNodeClicked(node: any) {
    if (!node.data) return;
    selectedPath = node.data.path;
    console.log("Selected category:", $state.snapshot(node.data));
    // TODO: Navigate to category page
  }

  // Handle context menu
  function handleContextMenu(event: MouseEvent, node: { data: CategoryNode }) {
    event.preventDefault();
    event.stopPropagation();

    contextMenuNode = {
      visible: true,
      x: event.clientX,
      y: event.clientY,
      node: node.data,
    };
  }

  // Context menu actions
  function handleAddSubCategory(path: string) {
    addDialogParentPath = path;
    showAddDialog = true;
    contextMenuNode.visible = false;
  }

  function handleEditCategory(node: CategoryNode) {
    editDialogNode = { path: node.path, name: node.name };
    showEditDialog = true;
    contextMenuNode.visible = false;
  }

  async function handleDeleteCategory(path: string) {
    if (!confirm("确定要删除此分类吗？子分类也将被删除。")) {
      return;
    }

    try {
      await invoke("delete_category", { path });
      contextMenuNode.visible = false;
      await loadCategories();
    } catch (error) {
      console.error("Failed to delete category:", error);
      alert(`删除分类失败: ${error}`);
    }
  }

  // Dialog close handlers
  function closeAddDialog() {
    showAddDialog = false;
    addDialogParentPath = undefined;
  }

  function closeEditDialog() {
    showEditDialog = false;
    editDialogNode = null;
  }

  // Dialog success handlers
  async function onCategoryCreated() {
    closeAddDialog();
    await loadCategories();
  }

  async function onCategoryUpdated() {
    closeEditDialog();
    await loadCategories();
  }

  // Public method to refresh categories (can be called from parent)
  export async function refreshCategories() {
    await loadCategories();
  }

  // Handle add root category button
  function handleAddRootCategory() {
    addDialogParentPath = undefined;
    showAddDialog = true;
  }

  // Template-level drag handlers
  let draggedPath = $state<string | null>(null);

  function handleTemplateDragStart(path: string, event: DragEvent) {
    console.log("Template dragstart:", path);
    draggedPath = path;
    if (event.dataTransfer) {
      event.dataTransfer.setData("text/plain", path);
      event.dataTransfer.effectAllowed = "move";
    }
  }

  function handleTemplateDragOver(event: DragEvent) {
    console.log("Template dragover");
    event.preventDefault();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = "move";
    }
  }

  async function handleTemplateDrop(targetPath: string, event: DragEvent) {
    console.log("Template drop:", { draggedPath, targetPath });
    event.preventDefault();
    event.stopPropagation();

    if (!draggedPath) {
      console.warn("No dragged path");
      return;
    }

    if (draggedPath === targetPath) {
      console.warn("Cannot drop on itself");
      return;
    }

    try {
      await invoke("move_category", {
        draggedPath,
        targetPath,
        position: "child",
      });

      await loadCategories();
      console.log("✅ Category moved successfully");
    } catch (error) {
      console.error("❌ Failed to move category:", error);
      alert(`移动分类失败: ${error}`);
    } finally {
      draggedPath = null;
    }
  }

  // Sort callback for tree ordering
  function sortCallback(items: any[]) {
    return items.sort((a, b) => {
      // First, sort by level (shallower levels first)
      const aLevel = a.data.path.split(".").length;
      const bLevel = b.data.path.split(".").length;
      if (aLevel !== bLevel) {
        return aLevel - bLevel;
      }

      // Then sort by sort_order
      return a.data.sort_order - b.data.sort_order;
    });
  }
</script>

<div class="category-tree">
  <!-- Tree view -->
  {#if categories.length > 0}
    <div class="tree-container overflow-y-auto" style="max-height: 300px;">
      <Tree
        data={categories}
        idMember="id"
        pathMember="path"
        displayValueMember="name"
        selectedNodeClass="ltree-selected-bold"
        dragOverNodeClass="ltree-dragover-highlight"
      >
        <!-- {#snippet nodeTemplate(node: any)}
          <div
            class="flex items-center gap-1 py-0.5 px-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 cursor-move"
            aria-label={node.data.name}
          >
            <Folder size={14} class="text-gray-500 dark:text-gray-400" />
            <span class="text-sm text-gray-700 dark:text-gray-300">
              {node.data.name}
            </span>
          </div>
        {/snippet} -->
      </Tree>
    </div>
  {:else}
    <div class="text-sm text-gray-500 dark:text-gray-400 py-2">
      暂无分类，点击 + 添加
    </div>
  {/if}
</div>

<!-- Add Category Dialog -->
<AddCategoryDialog
  open={showAddDialog}
  parentPath={addDialogParentPath}
  {onCategoryCreated}
  onClose={closeAddDialog}
/>

<!-- Edit Category Dialog -->
{#if editDialogNode}
  <EditCategoryDialog
    open={showEditDialog}
    categoryPath={editDialogNode.path}
    currentName={editDialogNode.name}
    {onCategoryUpdated}
    onClose={closeEditDialog}
  />
{/if}

<style>
  /* Custom scrollbar for tree container */
  .tree-container::-webkit-scrollbar {
    width: 6px;
  }

  .tree-container::-webkit-scrollbar-track {
    background: transparent;
  }

  .tree-container::-webkit-scrollbar-thumb {
    background: #d1d5db;
    border-radius: 3px;
  }

  .tree-container::-webkit-scrollbar-thumb:hover {
    background: #9ca3af;
  }

  /* Dark mode scrollbar */
  @media (prefers-color-scheme: dark) {
    .tree-container::-webkit-scrollbar-thumb {
      background: #4b5563;
    }

    .tree-container::-webkit-scrollbar-thumb:hover {
      background: #6b7280;
    }
  }

  :global(.dark) .tree-container::-webkit-scrollbar-thumb {
    background: #4b5563;
  }

  :global(.dark) .tree-container::-webkit-scrollbar-thumb:hover {
    background: #6b7280;
  }

  /* Selected node style */
  :global(.ltree-selected-bold) {
    font-weight: 600;
    color: rgb(var(--color-accent-500)) !important;
  }

  /* Drag over style */
  :global(.ltree-dragover-highlight) {
    border: 2px dashed rgb(var(--color-accent-500));
    background-color: rgba(var(--color-accent-500), 0.1);
  }
</style>
