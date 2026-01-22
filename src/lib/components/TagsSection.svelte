<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { t } from "$lib/i18n";
  import { Tags, Plus, Trash2 } from "lucide-svelte";
  import AddTagDialog from "./AddTagDialog.svelte";

  // Predefined color palette for tags
  const TAG_COLORS: Record<string, string> = {
    red: "#ef4444",
    orange: "#f97316",
    amber: "#f59e0b",
    yellow: "#eab308",
    lime: "#84cc16",
    green: "#22c55e",
    emerald: "#10b981",
    teal: "#14b8a6",
    cyan: "#06b6d4",
    sky: "#0ea5e9",
    blue: "#3b82f6",
    indigo: "#6366f1",
    violet: "#8b5cf6",
    purple: "#a855f7",
    fuchsia: "#d946ef",
    pink: "#ec4899",
    rose: "#f43f5e",
  };

  // Tags state - using Svelte 5 runes
  let allTags = $state<
    Array<{ id: number; name: string; count: number; color: string }>
  >([]);

  // Dialog state
  let showDialog = $state(false);

  // Context menu state
  let contextMenu = $state<{
    visible: boolean;
    x: number;
    y: number;
    tagId: number | null;
    tagName: string;
  }>({
    visible: false,
    x: 0,
    y: 0,
    tagId: null,
    tagName: "",
  });

  // Close dialog handler
  function closeDialog() {
    showDialog = false;
  }

  // Load labels from backend on mount
  async function loadTags() {
    try {
      console.log("Loading labels from backend...");
      const labels = await invoke<Record<string, any>[]>("get_all_labels");
      console.log("Received labels:", labels);

      allTags = labels.map((label) => ({
        id: label.id,
        name: label.name,
        count: label.document_count || 0,
        // Use the color name from database to get the color code
        color: TAG_COLORS[label.color] || TAG_COLORS["blue"],
      }));

      console.log("Processed tags count:", allTags.length);
    } catch (error) {
      console.error("Failed to load labels:", error);
      // Use demo data as fallback if running in browser (not Tauri)
      console.log("Using demo data as fallback");
      allTags = [
        { id: 1, name: "AI", count: 12, color: TAG_COLORS["blue"] },
        {
          id: 2,
          name: "Machine Learning",
          count: 8,
          color: TAG_COLORS["indigo"],
        },
        { id: 3, name: "Deep Learning", count: 6, color: TAG_COLORS["purple"] },
        { id: 4, name: "NLP", count: 5, color: TAG_COLORS["red"] },
        {
          id: 5,
          name: "Computer Vision",
          count: 4,
          color: TAG_COLORS["orange"],
        },
      ];
    }
  }

  // Load labels from backend on mount
  onMount(() => {
    loadTags();

    // Close context menu on click outside
    const handleClickOutside = () => {
      if (contextMenu.visible) {
        contextMenu.visible = false;
      }
    };

    document.addEventListener("click", handleClickOutside);
    return () => {
      document.removeEventListener("click", handleClickOutside);
    };
  });

  // Handle right-click on tag
  function handleTagContextMenu(
    event: MouseEvent,
    tag: { id: number; name: string },
  ) {
    event.preventDefault();
    event.stopPropagation();

    contextMenu = {
      visible: true,
      x: event.clientX,
      y: event.clientY,
      tagId: tag.id,
      tagName: tag.name,
    };
  }

  // Delete tag
  async function handleDeleteTag() {
    if (contextMenu.tagId === null || contextMenu.tagId === undefined) return;

    const tagId = contextMenu.tagId;
    console.log("Deleting tag with ID:", tagId);

    try {
      console.log("Invoking delete_label command with id:", tagId);
      await invoke("delete_label", { id: tagId });
      console.log("Tag deleted successfully");

      // Close context menu
      contextMenu.visible = false;

      // Reload tags
      await loadTags();
    } catch (error) {
      console.error("Failed to delete tag:", error);
      alert(`删除标签失败: ${error}`);
    }
  }
</script>

<div class="tags-section">
  <h3
    class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-2 pb-1 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between"
  >
    <div class="flex items-center gap-1">
      <Tags size={14} />
      {$t("navigation.tags")}
    </div>
    <button
      onclick={() => (showDialog = true)}
      class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 transition-colors p-0.5 rounded hover:bg-gray-100 dark:hover:bg-gray-700"
      aria-label="添加标签"
      title="添加标签"
    >
      <Plus size={14} />
    </button>
  </h3>
  <div class="overflow-y-auto pr-1" style="max-height: 200px;">
    <div class="flex flex-wrap gap-1.5">
      {#each allTags as tag}
        <span
          class="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium rounded-full text-white hover:opacity-80 cursor-pointer transition-opacity"
          style="background-color: {tag.color};"
          title="{tag.count} documents"
          oncontextmenu={(e) => handleTagContextMenu(e, tag)}
          role="button"
          tabindex="0"
          aria-label="{tag.name} - {tag.count} documents - right-click to delete"
        >
          {tag.name}
          <span class="text-[10px] opacity-70">({tag.count})</span>
        </span>
      {/each}
    </div>
  </div>
</div>

<!-- Context Menu -->
{#if contextMenu.visible}
  <div
    class="fixed z-50 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 min-w-37.5"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
    onmousedown={(e) => e.stopPropagation()}
    role="menu"
    tabindex="-1"
    aria-label="Tag context menu"
  >
    <div
      class="px-3 py-2 text-xs text-gray-500 dark:text-gray-400 border-b border-gray-200 dark:border-gray-700"
    >
      {contextMenu.tagName}
    </div>
    <button
      onclick={handleDeleteTag}
      class="w-full px-3 py-2 text-sm text-left text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 flex items-center gap-2 transition-colors"
    >
      <Trash2 size={14} />
      删除标签
    </button>
  </div>
{/if}

<!-- Add Tag Dialog -->
<AddTagDialog open={showDialog} onTagCreated={loadTags} onClose={closeDialog} />

<style>
  /* Custom scrollbar for tags section */
  .tags-section .overflow-y-auto::-webkit-scrollbar {
    width: 6px;
  }

  .tags-section .overflow-y-auto::-webkit-scrollbar-track {
    background: transparent;
  }

  .tags-section .overflow-y-auto::-webkit-scrollbar-thumb {
    background: #d1d5db;
    border-radius: 3px;
  }

  .tags-section .overflow-y-auto::-webkit-scrollbar-thumb:hover {
    background: #9ca3af;
  }

  /* Dark mode scrollbar */
  @media (prefers-color-scheme: dark) {
    .tags-section .overflow-y-auto::-webkit-scrollbar-thumb {
      background: #4b5563;
    }

    .tags-section .overflow-y-auto::-webkit-scrollbar-thumb:hover {
      background: #6b7280;
    }
  }

  /* Dark mode specific scrollbar */
  :global(.dark) .tags-section .overflow-y-auto::-webkit-scrollbar-thumb {
    background: #4b5563;
  }

  :global(.dark) .tags-section .overflow-y-auto::-webkit-scrollbar-thumb:hover {
    background: #6b7280;
  }
</style>
