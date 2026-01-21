<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { t } from "$lib/i18n";
  import { Tags } from "lucide-svelte";

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
  let allTags = $state<Array<{ name: string; count: number; color: string }>>(
    [],
  );

  // Load labels from backend on mount
  onMount(async () => {
    try {
      console.log("Loading labels from backend...");
      const labels = await invoke<Record<string, any>[]>("get_all_labels");
      console.log("Received labels:", labels);

      allTags = labels.map((label) => ({
        name: label.name,
        count: label.document_count || 0,
        // Use the color name from database to get the color code
        color: TAG_COLORS[label.color] || TAG_COLORS["blue"],
      }));

      console.log("Processed tags:", allTags);
    } catch (error) {
      console.error("Failed to load labels:", error);
      // Use demo data as fallback if running in browser (not Tauri)
      console.log("Using demo data as fallback");
      allTags = [
        { name: "AI", count: 12, color: TAG_COLORS["blue"] },
        { name: "Machine Learning", count: 8, color: TAG_COLORS["indigo"] },
        { name: "Deep Learning", count: 6, color: TAG_COLORS["purple"] },
        { name: "NLP", count: 5, color: TAG_COLORS["red"] },
        { name: "Computer Vision", count: 4, color: TAG_COLORS["orange"] },
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
  <div class="overflow-y-auto pr-1" style="max-height: 200px;">
    <div class="flex flex-wrap gap-1.5">
      {#each allTags as tag}
        <span
          class="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium rounded-full text-white hover:opacity-80 cursor-pointer transition-opacity"
          style="background-color: {tag.color};"
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
