<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { t } from "$lib/i18n";
  import { Tags } from "lucide-svelte";

  // Tags state - using Svelte 5 runes
  let allTags = $state<Array<{ name: string; count: number }>>([]);

  // Load labels from backend on mount
  onMount(async () => {
    try {
      console.log('Loading labels from backend...');
      const labels = await invoke<Record<string, any>[]>("get_all_labels");
      console.log('Received labels:', labels);

      allTags = labels.map((label) => ({
        name: label.name,
        count: label.document_count || 0,
      }));

      console.log('Processed tags:', allTags);

      // If no labels returned, try to initialize test data
      if (allTags.length === 0) {
        console.log('No labels found, initializing test data...');
        try {
          await invoke('init_test_labels');
          console.log('Test labels initialized, reloading...');
          // Reload labels
          const labels2 = await invoke<Record<string, any>[]>("get_all_labels");
          allTags = labels2.map((label) => ({
            name: label.name,
            count: label.document_count || 0,
          }));
          console.log('Tags after initialization:', allTags);
        } catch (initError) {
          console.error('Failed to initialize test labels:', initError);
        }
      }
    } catch (error) {
      console.error("Failed to load labels:", error);
      // Use demo data as fallback if running in browser (not Tauri)
      console.log('Using demo data as fallback');
      allTags = [
        { name: "AI", count: 12 },
        { name: "Machine Learning", count: 8 },
        { name: "Deep Learning", count: 6 },
        { name: "NLP", count: 5 },
        { name: "Computer Vision", count: 4 }
      ];
    }
  });
</script>

<div class="tags-section">
  <h3
    class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-2 pb-1 border-b border-gray-200 dark:border-gray-700 flex items-center gap-1"
  >
    <Tags size={14} />
    {$t("navigation.tags")}
  </h3>
  <div
    class="overflow-y-auto pr-1"
    style="max-height: 200px;"
  >
    <div class="flex flex-wrap gap-1.5">
      {#each allTags as tag}
        <span
          class="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium rounded-full bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-accent hover:text-white cursor-pointer transition-colors"
          title="{tag.count} documents"
        >
          {tag.name}
          <span class="text-[10px] opacity-70">({tag.count})</span>
        </span>
      {/each}
    </div>
  </div>
</div>

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
