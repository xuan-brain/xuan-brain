<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { X } from "lucide-svelte";

  // Props using Svelte 5 runes
  interface Props {
    open: boolean;
    categoryPath: string;
    currentName: string;
    onCategoryUpdated?: () => void;
    onClose?: () => void;
  }

  let { open, categoryPath, currentName, onCategoryUpdated, onClose }: Props =
    $props();

  // Form state - initialize with current name
  let categoryName = $state("");

  // Update form when currentName changes
  $effect(() => {
    categoryName = currentName;
  });

  // Error state
  let errorMessage = $state("");

  // Reset form
  function resetForm() {
    categoryName = currentName;
    errorMessage = "";
  }

  // Close dialog
  function closeDialog() {
    resetForm();
    if (onClose) {
      onClose();
    }
  }

  // Update category
  async function handleUpdateCategory() {
    // Validate
    if (!categoryName.trim()) {
      errorMessage = "请输入分类名称";
      return;
    }

    if (categoryName.trim().length > 50) {
      errorMessage = "分类名称不能超过50个字符";
      return;
    }

    // Check if name changed
    if (categoryName.trim() === currentName.trim()) {
      closeDialog();
      return;
    }

    try {
      await invoke("update_category", {
        path: categoryPath,
        name: categoryName.trim(),
      });

      // Reset and close
      resetForm();
      if (onClose) {
        onClose();
      }

      // Notify parent to refresh categories
      if (onCategoryUpdated) {
        onCategoryUpdated();
      }
    } catch (error) {
      console.error("Failed to update category:", error);
      errorMessage = `更新分类失败: ${error}`;
    }
  }

  // Handle keyboard shortcuts
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      closeDialog();
    } else if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) {
      handleUpdateCategory();
    }
  }
</script>

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
    role="presentation"
    onmousedown={(e) => {
      if (e.target === e.currentTarget) closeDialog();
    }}
  >
    <!-- svelte-ignore a11y_interactive_supports_focus -->
    <div
      class="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-96 max-w-[90vw]"
      onkeydown={handleKeydown}
      role="dialog"
      aria-modal="true"
      aria-labelledby="dialog-title"
    >
      <!-- Header -->
      <div
        class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700"
      >
        <h2
          id="dialog-title"
          class="text-lg font-semibold text-gray-900 dark:text-gray-100"
        >
          编辑分类
        </h2>
        <button
          onclick={closeDialog}
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
          aria-label="关闭"
        >
          <X size={16} />
        </button>
      </div>

      <!-- Body -->
      <div class="p-4 space-y-4">
        <!-- Error message -->
        {#if errorMessage}
          <div
            class="text-sm text-red-600 bg-red-50 dark:bg-red-900/20 dark:text-red-400 rounded-md p-3"
          >
            {errorMessage}
          </div>
        {/if}

        <!-- Category path info -->
        <div class="text-sm text-gray-600 dark:text-gray-400">
          分类路径: <code
            class="px-1.5 py-0.5 bg-gray-100 dark:bg-gray-700 rounded text-xs"
            >{categoryPath}</code
          >
        </div>

        <!-- Category name input -->
        <div>
          <label
            for="edit-category-name"
            class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
          >
            分类名称
          </label>
          <input
            id="edit-category-name"
            type="text"
            bind:value={categoryName}
            placeholder="输入分类名称..."
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
            autocomplete="off"
          />
          <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
            最多 50 个字符
          </p>
        </div>
      </div>

      <!-- Footer -->
      <div
        class="flex justify-end gap-2 p-4 border-t border-gray-200 dark:border-gray-700"
      >
        <button
          onclick={closeDialog}
          class="px-4 py-2 text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
        >
          取消
        </button>
        <button
          onclick={handleUpdateCategory}
          class="px-4 py-2 text-white bg-blue-600 rounded-md hover:bg-blue-700 transition-colors"
        >
          保存
        </button>
      </div>
    </div>
  </div>
{/if}
