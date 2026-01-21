<script lang="ts">
  import { t } from "$lib/i18n";
  import { Library, Star, Trash2 } from "lucide-svelte";

  // Active navigation item
  let activeItem = $state("library");

  // Navigation items configuration
  const navItems: Array<{
    id: string;
    getIcon: () => any;
    label: string;
  }> = [
    { id: "library", getIcon: () => Library, label: "navigation.library" },
    { id: "favorites", getIcon: () => Star, label: "navigation.favorites" },
    { id: "trash", getIcon: () => Trash2, label: "navigation.trash" },
  ];

  // Handle navigation item click
  function handleNavClick(itemId: string) {
    activeItem = itemId;
    // TODO: Add navigation logic here
  }

  // Handle keyboard navigation
  function handleKeydown(event: KeyboardEvent, itemId: string) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      handleNavClick(itemId);
    }
  }
</script>

<div class="flex-1 overflow-y-auto" style="padding: 10px;">
  <h2
    class="section-title text-gray-900 dark:text-gray-100 border-b border-gray-200 dark:border-gray-700"
  >
    {$t("navigation.title")}
  </h2>
  <nav>
    <ul class="list-none p-0 m-0">
      {#each navItems as item (item.id)}
        <li>
          <button
            type="button"
            class="nav-item w-full rounded cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300 flex items-center gap-1.5 text-left"
            class:bg-accent={activeItem === item.id}
            class:text-white={activeItem === item.id}
            class:hover:bg-accent={activeItem === item.id}
            onclick={() => handleNavClick(item.id)}
            onkeydown={(e) => handleKeydown(e, item.id)}
            aria-current={activeItem === item.id ? "page" : undefined}
          >
            {#if item.id === "library"}
              <Library size={14} />
            {:else if item.id === "favorites"}
              <Star size={14} />
            {:else if item.id === "trash"}
              <Trash2 size={14} />
            {/if}
            {$t(item.label)}
          </button>
        </li>
      {/each}
    </ul>
  </nav>
</div>
