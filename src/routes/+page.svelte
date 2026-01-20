<script lang="ts">
  import ThemeSwitcher from "$lib/components/ThemeSwitcher.svelte";
  import { onDestroy } from "svelte";

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

  // Status bar states - using Svelte 5 runes
  let currentTime = $state(new Date());
  let documentCount = $state(0);
  let syncStatus = $state("Synced");
  let isSyncing = $state(false);
  let searchStatus = $state("Ready");
  let memoryUsage = $state("0 MB");

  // Update time and memory usage every second
  let intervalId: ReturnType<typeof setInterval>;

  // Initialize timer on mount
  intervalId = setInterval(() => {
    currentTime = new Date();
    // Simulate memory usage (in actual projects, use performance.memory or Tauri API)
    const memory = Math.floor(Math.random() * 100 + 50);
    memoryUsage = `${memory} MB`;
  }, 1000);

  // Cleanup on unmount
  onDestroy(() => {
    clearInterval(intervalId);
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

  // Handle sync button click
  function handleSync() {
    isSyncing = true;
    syncStatus = "Syncing...";
    setTimeout(() => {
      isSyncing = false;
      syncStatus = "Synced";
    }, 2000);
  }
</script>

<svelte:window />

<div
  class="h-[calc(100vh-36px)] w-screen overflow-hidden bg-gray-200 dark:bg-gray-800 flex"
>
  <!-- Left sidebar -->
  <aside
    class="bg-white dark:bg-gray-800 overflow-y-auto min-w-37.5 border-r border-gray-200 dark:border-gray-700 shrink-0 flex flex-col"
    style="width: {leftWidth}%;"
  >
    <div class="p-5 flex-1">
      <h2
        class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4 pb-2 border-b-2 border-gray-200 dark:border-gray-700"
      >
        Navigation
      </h2>
      <nav>
        <ul class="list-none p-0 m-0">
          <li
            class="px-3 py-2.5 mb-1 rounded-lg cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300"
          >
            Library
          </li>
          <li
            class="px-3 py-2.5 mb-1 rounded-lg cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300"
          >
            Categories
          </li>
          <li
            class="px-3 py-2.5 mb-1 rounded-lg cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300"
          >
            Tags
          </li>
          <li
            class="px-3 py-2.5 mb-1 rounded-lg cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300"
          >
            Favorites
          </li>
          <li
            class="px-3 py-2.5 mb-1 rounded-lg cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300"
          >
            Trash
          </li>
        </ul>
      </nav>
    </div>

    <!-- Theme switcher at bottom of left sidebar -->
    <div class="p-5 border-t border-gray-200 dark:border-gray-700">
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
    class="w-0.5 bg-gray-300 dark:bg-gray-600 hover:bg-blue-500 dark:hover:bg-blue-500 cursor-col-resize shrink-0 transition-colors duration-150 z-10"
    class:bg-blue-500={isDraggingLeft}
    class:dark:bg-blue-500={isDraggingLeft}
    onmousedown={handleLeftResizerMouseDown}
  ></div>

  <!-- Main content area -->
  <main
    class="bg-gray-50 dark:bg-gray-900 overflow-y-auto flex flex-col flex-1 min-w-0"
  >
    <div
      class="bg-white dark:bg-gray-800 p-5 border-b border-gray-200 dark:border-gray-700 flex justify-between items-center shrink-0"
    >
      <h1 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 m-0">
        Library
      </h1>
      <div class="flex gap-2.5">
        <button
          class="px-4 py-2 text-sm font-medium bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-lg border border-transparent cursor-pointer transition-all duration-200 shadow-[0_2px_4px_rgba(0,0,0,0.1)] hover:bg-blue-500 hover:text-white hover:-translate-y-0.5 hover:shadow-[0_4px_8px_rgba(0,0,0,0.15)] active:translate-y-0 active:shadow-[0_2px_4px_rgba(0,0,0,0.1)]"
        >
          Import Documents
        </button>
        <button
          class="px-4 py-2 text-sm font-medium bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-lg border border-transparent cursor-pointer transition-all duration-200 shadow-[0_2px_4px_rgba(0,0,0,0.1)] hover:bg-blue-500 hover:text-white hover:-translate-y-0.5 hover:shadow-[0_4px_8px_rgba(0,0,0,0.15)] active:translate-y-0 active:shadow-[0_2px_4px_rgba(0,0,0,0.1)]"
        >
          Search
        </button>
      </div>
    </div>
    <div class="flex-1 p-5 overflow-y-auto min-h-0">
      <!-- Document list will be rendered here -->
      <p class="text-gray-400 dark:text-gray-600 text-center italic mt-10">
        No documents yet
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
    class="w-0.5 bg-gray-300 dark:bg-gray-600 hover:bg-blue-500 dark:hover:bg-blue-500 cursor-col-resize shrink-0 transition-colors duration-150 z-10"
    class:bg-blue-500={isDraggingRight}
    class:dark:bg-blue-500={isDraggingRight}
    onmousedown={handleRightResizerMouseDown}
  ></div>

  <!-- Right sidebar -->
  <aside
    class="bg-white dark:bg-gray-800 overflow-y-auto min-w-37.5 border-l border-gray-200 dark:border-gray-700 shrink-0"
    style="width: {rightWidth}%;"
  >
    <div class="p-5">
      <h2
        class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4 pb-2 border-b-2 border-gray-200 dark:border-gray-700"
      >
        Details
      </h2>
      <div class="detail-panel">
        <p class="text-gray-400 dark:text-gray-600 text-center italic mt-10">
          Select a document to view details
        </p>
      </div>
    </div>
  </aside>
</div>

<!-- Status bar -->
<footer
  class="h-9 bg-gray-800 dark:bg-gray-950 border-t border-gray-700 dark:border-gray-800 flex items-center justify-between px-4 text-xs text-gray-300 dark:text-gray-400 select-none"
>
  <div class="flex items-center gap-4">
    <button
      class="flex items-center gap-1.5 hover:text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      class:animate-pulse={isSyncing}
      disabled={isSyncing}
      onclick={handleSync}
    >
      <span
        class="w-2 h-2 rounded-full"
        class:bg-green-500={syncStatus === "Synced"}
        class:bg-yellow-500={syncStatus === "Syncing..."}
        class:bg-gray-500={syncStatus === "Unsynced"}
      ></span>
      {syncStatus}
    </button>
    <span class="flex items-center gap-1">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="w-3.5 h-3.5"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          d="M9 4.804A7.968 7.968 0 005.5 4c-1.255 0-2.443.29-3.5.804v10A7.969 7.969 0 015.5 14c1.669 0 3.218.51 4.5 1.385A7.962 7.962 0 0114.5 14c1.255 0 2.443.29 3.5.804v-10A7.968 7.968 0 0014.5 4c-1.255 0-2.443.29-3.5.804V12a1 1 0 11-2 0V4.804z"
        />
      </svg>
      Documents: {documentCount}
    </span>
    <span class="hidden md:flex items-center gap-1">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="w-3.5 h-3.5"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          fill-rule="evenodd"
          d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z"
          clip-rule="evenodd"
        />
      </svg>
      {searchStatus}
    </span>
    <span class="hidden lg:flex items-center gap-1">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="w-3.5 h-3.5"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          fill-rule="evenodd"
          d="M11.3 1.046A1 1 0 0112 2v5h4a1 1 0 01.82 1.573l-7 10A1 1 0 018 18v-5H4a1 1 0 01-.82-1.573l7-10a1 1 0 011.12-.38z"
          clip-rule="evenodd"
        />
      </svg>
      {memoryUsage}
    </span>
  </div>
  <div class="flex items-center gap-4">
    <span class="hidden sm:inline">Version 0.1.0</span>
    <span class="font-mono bg-gray-700 dark:bg-gray-900 px-2 py-0.5 rounded">
      {currentTime.toLocaleTimeString()}
    </span>
  </div>
</footer>

<style>
  /* Prevent text selection during dragging */
  :global(.cursor-col-resize) {
    user-select: none;
    -webkit-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
  }
</style>
