<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Tree } from "@keenmate/svelte-treeview";
  import { Folder, FolderOpen, Plus, Edit, Trash2 } from "lucide-svelte";
  import AddCategoryDialog from "./AddCategoryDialog.svelte";
  import EditCategoryDialog from "./EditCategoryDialog.svelte";

  // Category data structure
  interface CategoryNode {
    path: string;
    name: string;
    parent_id?: number;
    sort_order: number;
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
      categories = data;
    } catch (error) {
      console.error("Failed to load categories:", error);
      // Use demo data as fallback
      categories = [
        { path: "1", name: "计算机科学", parent_id: undefined, sort_order: 0 },
        { path: "1.1", name: "人工智能", parent_id: 1, sort_order: 0 },
        { path: "1.1.1", name: "机器学习", parent_id: 2, sort_order: 0 },
        { path: "1.1.2", name: "深度学习", parent_id: 2, sort_order: 0 },
        { path: "1.2", name: "数据库", parent_id: 1, sort_order: 0 },
        { path: "2", name: "物理学", parent_id: undefined, sort_order: 0 },
        { path: "2.1", name: "量子力学", parent_id: 6, sort_order: 0 },
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
  function handleNodeClicked(node: { data: CategoryNode }) {
    selectedPath = node.data.path;
    console.log("Selected category:", node.data);
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

  // Handle add root category button
  function handleAddRootCategory() {
    addDialogParentPath = undefined;
    showAddDialog = true;
  }
</script>

<div class="category-tree">
  <!-- Tree view -->
  {#if categories.length > 0}
    <div class="tree-container overflow-y-auto" style="max-height: 300px;">
      <Tree
        data={categories}
        idMember="path"
        pathMember="path"
        displayValueMember="name"
        selectedNodeClass="ltree-selected-bold"
        onNodeClicked={handleNodeClicked}
      >
        {#snippet nodeTemplate(node)}
          <div
            class="flex items-center gap-1 py-0.5 px-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer"
            oncontextmenu={(e) => handleContextMenu(e, node)}
            role="button"
            tabindex="0"
            aria-label={node.data.name}
          >
            <Folder size={14} class="text-gray-500 dark:text-gray-400" />
            <span class="text-sm text-gray-700 dark:text-gray-300">
              {node.data.name}
            </span>
          </div>
        {/snippet}

        {#snippet contextMenu(node, closeMenu)}
          <div
            class="fixed z-50 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 min-w-[150px]"
            style="left: {contextMenuNode.x}px; top: {contextMenuNode.y}px;"
            onmousedown={(e) => e.stopPropagation()}
            role="menu"
            tabindex="-1"
            aria-label="Category context menu"
          >
            <button
              onclick={() => handleAddSubCategory(node.data.path)}
              class="w-full px-3 py-2 text-sm text-left text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-2 transition-colors"
            >
              <Plus size={14} />
              添加子分类
            </button>
            <button
              onclick={() => handleEditCategory(node.data)}
              class="w-full px-3 py-2 text-sm text-left text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-2 transition-colors"
            >
              <Edit size={14} />
              编辑
            </button>
            <button
              onclick={() => handleDeleteCategory(node.data.path)}
              class="w-full px-3 py-2 text-sm text-left text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 flex items-center gap-2 transition-colors"
            >
              <Trash2 size={14} />
              删除
            </button>
          </div>
        {/snippet}
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
