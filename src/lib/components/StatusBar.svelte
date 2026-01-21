<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { t } from "$lib/i18n";

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
  onMount(() => {
    intervalId = setInterval(() => {
      currentTime = new Date();
      // Simulate memory usage (in actual projects, use performance.memory or Tauri API)
      const memory = Math.floor(Math.random() * 100 + 50);
      memoryUsage = `${memory} MB`;
    }, 1000);
  });

  // Cleanup on unmount
  onDestroy(() => {
    clearInterval(intervalId);
  });

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

<footer
  class="h-8 bg-gray-800 dark:bg-gray-950 border-t border-gray-700 dark:border-gray-800 flex items-center justify-between px-2 text-xs text-gray-300 dark:text-gray-400 select-none"
>
  <div class="flex items-center gap-2">
    <button
      class="flex items-center gap-1 hover:text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      class:animate-pulse={isSyncing}
      disabled={isSyncing}
      onclick={handleSync}
    >
      <span
        class="w-1.5 h-1.5 rounded-full"
        class:bg-green-500={syncStatus === "Synced"}
        class:bg-yellow-500={syncStatus === "Syncing..."}
        class:bg-gray-500={syncStatus === "Unsynced"}
      ></span>
      {syncStatus}
    </button>
    <span class="flex items-center gap-0.5">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="w-2.5 h-2.5"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          d="M9 4.804A7.968 7.968 0 005.5 4c-1.255 0-2.443.29-3.5.804v10A7.969 7.969 0 015.5 14c1.669 0 3.218.51 4.5 1.385A7.962 7.962 0 0114.5 14c1.255 0 2.443.29 3.5.804v-10A7.968 7.968 0 0014.5 4c-1.255 0-2.443.29-3.5.804V12a1 1 0 11-2 0V4.804z"
        />
      </svg>
      {$t("status.documents")}: {documentCount}
    </span>
    <span class="hidden md:flex items-center gap-0.5">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="w-2.5 h-2.5"
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
    <span class="hidden lg:flex items-center gap-0.5">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="w-2.5 h-2.5"
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
  <div class="flex items-center gap-2">
    <span class="hidden sm:inline">{$t("status.version")} 0.1.0</span>
    <span
      class="font-mono bg-gray-700 dark:bg-gray-900 px-1.5 py-0.5 rounded text-[10px]"
    >
      {currentTime.toLocaleTimeString()}
    </span>
  </div>
</footer>
