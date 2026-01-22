<script lang="ts">
  import { Library, Star, Trash2, Tag, Plus } from "lucide-svelte";
  import CategoryTree from "./CategoryTree.svelte";
  import TagsSection from "./TagsSection.svelte";
  import AddCategoryDialog from "./AddCategoryDialog.svelte";
  import AddTagDialog from "./AddTagDialog.svelte";

  // Active navigation item
  let activeItem = $state("library");

  // Dialog state
  let showAddCategoryDialog = $state(false);
  let showAddTagDialog = $state(false);

  // Handle navigation click
  function handleNavClick(itemId: string) {
    activeItem = itemId;
    console.log("Navigated to:", itemId);
    // TODO: Add navigation logic
  }

  // Handle keyboard navigation
  function handleKeydown(event: KeyboardEvent, itemId: string) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      handleNavClick(itemId);
    }
  }

  // Handle add category
  function handleAddCategory() {
    showAddCategoryDialog = true;
  }

  // Handle add tag
  function handleAddTag() {
    showAddTagDialog = true;
  }

  // Dialog close handlers
  function closeAddCategoryDialog() {
    showAddCategoryDialog = false;
  }

  function closeAddTagDialog() {
    showAddTagDialog = false;
  }

  async function onCategoryCreated() {
    closeAddCategoryDialog();
  }

  async function onTagCreated() {
    closeAddTagDialog();
  }
</script>

<div class="flex-1 flex flex-col overflow-hidden">
  <!-- Scrollable content area (Library) -->
  <div class="flex-1 overflow-y-auto" style="padding: 10px;">
    <!-- Library navigation item -->
    <nav>
      <ul class="list-none p-0 m-0 space-y-1">
        <li>
          <div
            class="rounded cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300 flex items-center gap-1.5 text-left py-1.5 px-2 font-semibold"
            class:bg-accent={activeItem === "library"}
            class:text-white={activeItem === "library"}
            class:hover:bg-accent={activeItem === "library"}
            onclick={() => handleNavClick("library")}
            onkeydown={(e) => handleKeydown(e, "library")}
            aria-current={activeItem === "library" ? "page" : undefined}
            role="button"
            tabindex="0"
          >
            <Library size={14} />
            文献库
            <button
              onclick={(e) => {
                e.stopPropagation();
                handleAddCategory();
              }}
              class="ml-auto hover:bg-gray-200 dark:hover:bg-gray-600 rounded p-0.5 transition-colors"
              aria-label="添加分类"
              title="添加分类"
            >
              <Plus size={12} />
            </button>
          </div>

          <!-- Category Tree (always expanded) -->
          <div class="ml-6 mt-1">
            <CategoryTree />
          </div>
        </li>
      </ul>
    </nav>
  </div>

  <!-- Bottom fixed navigation items -->
  <nav
    class="border-t border-gray-200 dark:border-gray-700"
    style="padding: 10px;"
  >
    <ul class="list-none p-0 m-0 space-y-1">
      <!-- Favorites -->
      <li>
        <button
          type="button"
          class="w-full rounded cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300 flex items-center gap-1.5 text-left py-1.5 px-2 font-semibold"
          class:bg-accent={activeItem === "favorites"}
          class:text-white={activeItem === "favorites"}
          class:hover:bg-accent={activeItem === "favorites"}
          onclick={() => handleNavClick("favorites")}
          onkeydown={(e) => handleKeydown(e, "favorites")}
          aria-current={activeItem === "favorites" ? "page" : undefined}
        >
          <Star size={14} />
          收藏
        </button>
      </li>

      <!-- Divider -->
      <li class="border-t border-gray-200 dark:border-gray-700"></li>

      <!-- Tags -->
      <li>
        <div
          class="rounded cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300 flex items-center gap-1.5 text-left py-1.5 px-2 font-semibold"
          class:bg-accent={activeItem === "tags"}
          class:text-white={activeItem === "tags"}
          class:hover:bg-accent={activeItem === "tags"}
          onclick={() => handleNavClick("tags")}
          onkeydown={(e) => handleKeydown(e, "tags")}
          aria-current={activeItem === "tags" ? "page" : undefined}
          role="button"
          tabindex="0"
        >
          <Tag size={14} />
          标签
          <button
            onclick={(e) => {
              e.stopPropagation();
              handleAddTag();
            }}
            class="ml-auto hover:bg-gray-200 dark:hover:bg-gray-600 rounded p-0.5 transition-colors"
            aria-label="添加标签"
            title="添加标签"
          >
            <Plus size={12} />
          </button>
        </div>

        <!-- Tags Section (always expanded) -->
        <div class="ml-6 mt-1">
          <TagsSection />
        </div>
      </li>

      <!-- Divider -->
      <li class="border-t border-gray-200 dark:border-gray-700"></li>

      <!-- Trash -->
      <li>
        <button
          type="button"
          class="w-full rounded cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300 flex items-center gap-1.5 text-left py-1.5 px-2 font-semibold"
          class:bg-accent={activeItem === "trash"}
          class:text-white={activeItem === "trash"}
          class:hover:bg-accent={activeItem === "trash"}
          onclick={() => handleNavClick("trash")}
          onkeydown={(e) => handleKeydown(e, "trash")}
          aria-current={activeItem === "trash" ? "page" : undefined}
        >
          <Trash2 size={14} />
          回收站
        </button>
      </li>
    </ul>
  </nav>
</div>

<!-- Add Category Dialog -->
<AddCategoryDialog
  open={showAddCategoryDialog}
  {onCategoryCreated}
  onClose={closeAddCategoryDialog}
/>

<!-- Add Tag Dialog -->
<AddTagDialog
  open={showAddTagDialog}
  {onTagCreated}
  onClose={closeAddTagDialog}
/>
