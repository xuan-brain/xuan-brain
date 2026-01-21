<script lang="ts">
  import ThemeSwitcher from "$lib/components/ThemeSwitcher.svelte";
  import TagsSection from "$lib/components/TagsSection.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import { onDestroy } from "svelte";
  import { t } from "$lib/i18n";
  import { Library, FolderTree, Tags, Star, Trash2 } from "lucide-svelte";

  // Load saved column widths from localStorage, use defaults if not present
  const STORAGE_KEY = "xuan-brain-layout-widths";

  // Minimum width (percentage)
  const MIN_WIDTH_PERCENT = 10;
  const MAX_WIDTH_PERCENT = 40;

  function loadWidths(): { left: number; right: number } {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        const widths = JSON.parse(saved);
        return {
          left: Math.max(
            MIN_WIDTH_PERCENT,
            Math.min(MAX_WIDTH_PERCENT, widths.left),
          ),
          right: Math.max(
            MIN_WIDTH_PERCENT,
            Math.min(MAX_WIDTH_PERCENT, widths.right),
          ),
        };
      }
    } catch (e) {
      console.error("Failed to load layout widths:", e);
    }
    return { left: 15, right: 15 };
  }

  const savedWidths = loadWidths();

  // Column width states (percentage) - using Svelte 5 runes
  let leftWidth = $state(savedWidths.left);
  let rightWidth = $state(savedWidths.right);
  let middleWidth = $derived(100 - leftWidth - rightWidth);

  // Drag states - using Svelte 5 runes
  let isDraggingLeft = $state(false);
  let isDraggingRight = $state(false);
  let startX = $state(0);
  let startLeftWidth = $state(0);
  let startRightWidth = $state(0);

  // Save column widths to localStorage when they change
  $effect(() => {
    try {
      const widthsToSave = JSON.stringify({
        left: leftWidth,
        right: rightWidth,
      });
      localStorage.setItem(STORAGE_KEY, widthsToSave);
    } catch (e) {
      console.error("Failed to save layout widths:", e);
    }
  });

  // Left drag handle
  function handleLeftResizerMouseDown(event: MouseEvent) {
    isDraggingLeft = true;
    startX = event.clientX;
    startLeftWidth = leftWidth;

    // Add global event listeners
    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);

    // Prevent text selection
    event.preventDefault();
  }

  // Right drag handle
  function handleRightResizerMouseDown(event: MouseEvent) {
    isDraggingRight = true;
    startX = event.clientX;
    startRightWidth = rightWidth;

    // Add global event listeners
    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);

    // Prevent text selection
    event.preventDefault();
  }

  // Mouse move
  function handleMouseMove(event: MouseEvent) {
    if (!isDraggingLeft && !isDraggingRight) return;

    const containerWidth = window.innerWidth;
    const deltaX = event.clientX - startX;
    const deltaPercent = (deltaX / containerWidth) * 100;

    if (isDraggingLeft) {
      const newWidth = startLeftWidth + deltaPercent;
      leftWidth = Math.max(
        MIN_WIDTH_PERCENT,
        Math.min(MAX_WIDTH_PERCENT, newWidth),
      );
    }

    if (isDraggingRight) {
      const newWidth = startRightWidth - deltaPercent;
      rightWidth = Math.max(
        MIN_WIDTH_PERCENT,
        Math.min(MAX_WIDTH_PERCENT, newWidth),
      );
    }
  }

  // Mouse up
  function handleMouseUp() {
    isDraggingLeft = false;
    isDraggingRight = false;

    // Remove global event listeners
    document.removeEventListener("mousemove", handleMouseMove);
    document.removeEventListener("mouseup", handleMouseUp);
  }
</script>

<svelte:window />

<div
  class="h-[calc(100vh-32px)] w-screen overflow-hidden bg-gray-200 dark:bg-gray-800 flex"
>
  <!-- Left sidebar -->
  <aside
    class="bg-white dark:bg-gray-800 overflow-hidden min-w-37.5 border-r border-gray-200 dark:border-gray-700 shrink-0 flex flex-col"
    style="width: {leftWidth}%;"
  >
    <div class="flex-1 overflow-y-auto" style="padding: 10px;">
      <h2
        class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-2 pb-1 border-b border-gray-200 dark:border-gray-700"
      >
        {$t("navigation.title")}
      </h2>
      <nav>
        <ul class="list-none p-0 m-0">
          <li
            class="mb-0.5 rounded cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300 text-sm flex items-center gap-1.5"
            style="padding: 5px 8px; --hover-bg: var(--accent-color);"
            class:hover:bg-accent={true}
          >
            <Library size={14} class="text-accent" />
            {$t("navigation.library")}
          </li>
          <li
            class="mb-0.5 rounded cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300 text-sm flex items-center gap-1.5"
            style="padding: 5px 8px;"
          >
            <FolderTree size={14} />
            {$t("navigation.categories")}
          </li>
          <li
            class="mb-0.5 rounded cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300 text-sm flex items-center gap-1.5"
            style="padding: 5px 8px;"
          >
            <Tags size={14} />
            {$t("navigation.tags")}
          </li>
          <li
            class="mb-0.5 rounded cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300 text-sm flex items-center gap-1.5"
            style="padding: 5px 8px;"
          >
            <Star size={14} />
            {$t("navigation.favorites")}
          </li>
          <li
            class="mb-0.5 rounded cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300 text-sm flex items-center gap-1.5"
            style="padding: 5px 8px;"
          >
            <Trash2 size={14} />
            {$t("navigation.trash")}
          </li>
        </ul>
      </nav>
    </div>

    <!-- Tags Section at bottom -->
    <div
      class="border-t border-gray-200 dark:border-gray-700"
      style="padding: 10px;"
    >
      <TagsSection />
    </div>

    <!-- Theme switcher at bottom of left sidebar -->
    <div
      class="border-t border-gray-200 dark:border-gray-700"
      style="padding: 5px;"
    >
      <ThemeSwitcher />
    </div>
  </aside>

  <!-- Left drag handle -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    role="separator"
    aria-orientation="vertical"
    aria-valuenow={leftWidth}
    aria-valuemin={MIN_WIDTH_PERCENT}
    aria-valuemax={MAX_WIDTH_PERCENT}
    aria-label="Adjust left sidebar width"
    class="w-0.5 bg-gray-300 dark:bg-gray-600 cursor-col-resize shrink-0 transition-colors duration-150 z-10 hover:bg-accent"
    class:bg-accent={isDraggingLeft}
    onmousedown={handleLeftResizerMouseDown}
  ></div>

  <!-- Main content area -->
  <main
    class="bg-gray-50 dark:bg-gray-900 overflow-y-auto flex flex-col flex-1 min-w-0"
  >
    <div
      class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 flex justify-between items-center shrink-0"
      style="padding: 10px;"
    >
      <h1 class="text-lg font-semibold text-gray-900 dark:text-gray-100 m-0">
        {$t("main.title")}
      </h1>
      <div class="flex gap-1.5">
        <button
          class="text-sm font-medium bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded border border-gray-200 dark:border-gray-600 cursor-pointer transition-colors hover:text-white active:scale-95 hover:bg-accent"
          style="padding: 5px 8px;"
        >
          {$t("main.importDocuments")}
        </button>
        <button
          class="text-sm font-medium bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded border border-gray-200 dark:border-gray-600 cursor-pointer transition-colors hover:text-white active:scale-95 hover:bg-accent"
          style="padding: 5px 8px;"
        >
          {$t("main.search")}
        </button>
      </div>
    </div>
    <div class="flex-1 overflow-y-auto min-h-0" style="padding: 10px;">
      <!-- Document list will be rendered here -->
      <p class="text-gray-400 dark:text-gray-600 text-center italic mt-10">
        {$t("main.noDocuments")}
      </p>
    </div>
  </main>

  <!-- Right drag handle -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    role="separator"
    aria-orientation="vertical"
    aria-valuenow={rightWidth}
    aria-valuemin={MIN_WIDTH_PERCENT}
    aria-valuemax={MAX_WIDTH_PERCENT}
    aria-label="Adjust right sidebar width"
    class="w-0.5 bg-gray-300 dark:bg-gray-600 cursor-col-resize shrink-0 transition-colors duration-150 z-10 hover:bg-accent"
    class:bg-accent={isDraggingRight}
    onmousedown={handleRightResizerMouseDown}
  ></div>

  <!-- Right sidebar -->
  <aside
    class="bg-white dark:bg-gray-800 overflow-y-auto min-w-37.5 border-l border-gray-200 dark:border-gray-700 shrink-0"
    style="width: {rightWidth}%;"
  >
    <div style="padding: 10px;">
      <h2
        class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-2 pb-1 border-b border-gray-200 dark:border-gray-700"
      >
        {$t("details.title")}
      </h2>
      <div class="detail-panel">
        <p class="text-gray-400 dark:text-gray-600 text-center italic mt-10">
          {$t("details.noSelection")}
        </p>
      </div>
    </div>
  </aside>
</div>

<!-- Status bar -->
<StatusBar />

<style>
  /* Prevent text selection during dragging */
  :global(.cursor-col-resize) {
    user-select: none;
    -webkit-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
  }

  /* Custom scrollbar for tags section */
  .overflow-y-auto::-webkit-scrollbar {
    width: 6px;
  }

  .overflow-y-auto::-webkit-scrollbar-track {
    background: transparent;
  }

  .overflow-y-auto::-webkit-scrollbar-thumb {
    background: #d1d5db;
    border-radius: 3px;
  }

  .overflow-y-auto::-webkit-scrollbar-thumb:hover {
    background: #9ca3af;
  }

  /* Dark mode scrollbar */
  @media (prefers-color-scheme: dark) {
    .overflow-y-auto::-webkit-scrollbar-thumb {
      background: #4b5563;
    }

    .overflow-y-auto::-webkit-scrollbar-thumb:hover {
      background: #6b7280;
    }
  }

  /* Dark mode specific scrollbar */
  :global(.dark) .overflow-y-auto::-webkit-scrollbar-thumb {
    background: #4b5563;
  }

  :global(.dark) .overflow-y-auto::-webkit-scrollbar-thumb:hover {
    background: #6b7280;
  }
</style>
