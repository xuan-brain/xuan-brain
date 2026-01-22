<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { X } from "lucide-svelte";

  // Props using Svelte 5 runes
  interface Props {
    open: boolean;
    onTagCreated?: () => void;
    onClose?: () => void;
  }

  let { open, onTagCreated, onClose }: Props = $props();

  // Predefined color palette (must match TagsSection)
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

  // Form state
  let tagName = $state("");
  let selectedColor = $state("blue"); // Default color

  // Error state
  let errorMessage = $state("");

  // Reset form
  function resetForm() {
    tagName = "";
    selectedColor = "blue";
    errorMessage = "";
  }

  // Close dialog
  function closeDialog() {
    resetForm();
    if (onClose) {
      onClose();
    }
  }

  // Create tag
  async function handleCreateTag() {
    // Validate
    if (!tagName.trim()) {
      errorMessage = "请输入标签名称";
      return;
    }

    if (tagName.trim().length > 50) {
      errorMessage = "标签名称不能超过50个字符";
      return;
    }

    try {
      await invoke("create_label", {
        name: tagName.trim(),
        color: selectedColor,
      });

      // Reset and close
      resetForm();
      if (onClose) {
        onClose();
      }

      // Notify parent to refresh tags
      if (onTagCreated) {
        onTagCreated();
      }
    } catch (error) {
      console.error("Failed to create tag:", error);
      errorMessage = `创建标签失败: ${error}`;
    }
  }

  // Handle keyboard shortcuts
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      closeDialog();
    } else if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) {
      handleCreateTag();
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
        class="flex items-center justify-between dialog-header-padding border-b border-gray-200 dark:border-gray-700"
      >
        <h2
          id="dialog-title"
          class="dialog-title text-gray-900 dark:text-gray-100"
        >
          添加标签
        </h2>
        <button
          onclick={closeDialog}
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors dialog-close-button"
          aria-label="关闭"
        >
          <X size={16} />
        </button>
      </div>

      <!-- Body -->
      <div class="dialog-padding dialog-section-gap">
        <!-- Error message -->
        {#if errorMessage}
          <div
            class="dialog-body-text text-red-600 bg-red-50 dark:bg-red-900/20 dark:text-red-400 rounded-md"
          >
            {errorMessage}
          </div>
        {/if}

        <!-- Tag name input -->
        <div>
          <label
            for="tag-name"
            class="block dialog-label text-gray-700 dark:text-gray-300"
          >
            标签名称
          </label>
          <input
            id="tag-name"
            type="text"
            bind:value={tagName}
            placeholder="输入标签名称..."
            class="w-full dialog-input border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
            autocomplete="off"
          />
        </div>

        <!-- Color picker -->
        <div>
          <fieldset>
            <legend class="block dialog-label text-gray-700 dark:text-gray-300">
              选择颜色
            </legend>
            <div class="flex flex-wrap gap-2">
              {#each Object.entries(TAG_COLORS) as [colorName, colorHex]}
                <button
                  type="button"
                  class="w-8 h-8 rounded-full border-2 transition-all hover:scale-110 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                  class:border-gray-300={selectedColor !== colorName}
                  class:border-gray-900={selectedColor === colorName}
                  style="background-color: {colorHex};"
                  onclick={() => (selectedColor = colorName)}
                  aria-label="选择 {colorName} 颜色"
                  title={colorName}
                >
                  {#if selectedColor === colorName}
                    <svg
                      class="w-5 h-5 text-white drop-shadow-md"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="3"
                        d="M5 13l4 4L19 7"
                      />
                    </svg>
                  {/if}
                </button>
              {/each}
            </div>
          </fieldset>
        </div>
      </div>

      <!-- Footer -->
      <div
        class="flex justify-end dialog-gap dialog-padding border-t border-gray-200 dark:border-gray-700"
      >
        <button
          onclick={closeDialog}
          class="dialog-button text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
        >
          取消
        </button>
        <button
          onclick={handleCreateTag}
          class="dialog-button text-white bg-blue-600 rounded-md hover:bg-blue-700 transition-colors"
        >
          创建
        </button>
      </div>
    </div>
  </div>
{/if}
