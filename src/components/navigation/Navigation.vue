<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invokeCommand } from "@/lib/tauri";
import { useI18n } from "@/lib/i18n";
import CategoryTree from "@/components/navigation/CategoryTree.vue";
import AddCategoryDialog from "@/components/dialogs/AddCategoryDialog.vue";
import EditCategoryDialog from "@/components/dialogs/EditCategoryDialog.vue";
import AddTagDialog from "@/components/dialogs/AddTagDialog.vue";

const { t } = useI18n();

interface Label {
  id: number;
  name: string;
  color: string;
}

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

// State
const labels = ref<Label[]>([]);
const loading = ref(false);
const activeNavItem = ref<string>("library");

// Context menu state for tags
const tagContextMenu = ref(false);
const tagContextMenuX = ref(0);
const tagContextMenuY = ref(0);
const selectedTag = ref<Label | null>(null);

// Dialog states
const showAddCategoryDialog = ref(false);
const showEditCategoryDialog = ref(false);
const showAddTagDialog = ref(false);
const showEditTagDialog = ref(false);
const editingTag = ref<Label | null>(null);

// Emit events
const emit = defineEmits<{
  categorySelect: [path: string | null];
  viewChange: [view: "library" | "favorites" | "trash"];
}>();

// Load labels from backend
async function loadLabels() {
  loading.value = true;
  try {
    labels.value = await invokeCommand<Label[]>("get_all_labels");
  } catch (error) {
    console.error("Failed to load labels:", error);
  } finally {
    loading.value = false;
  }
}

// Handle category selection from CategoryTree
function handleCategorySelect(path: string | null) {
  activeNavItem.value = "library";
  emit("categorySelect", path);
  emit("viewChange", "library");
}

// Handle navigation clicks
function handleNavClick(item: "library" | "favorites" | "trash") {
  activeNavItem.value = item;
  if (item === "library") {
    emit("categorySelect", null);
  }
  emit("viewChange", item);
}

// Handle label click
function handleLabelClick(labelId: number) {
  activeNavItem.value = "library";
  // TODO: Filter by label
  emit("viewChange", "library");
}

// Show context menu for tag
function showTagContextMenu(event: MouseEvent, tag: Label) {
  event.preventDefault();
  event.stopPropagation();
  selectedTag.value = tag;
  tagContextMenuX.value = event.clientX;
  tagContextMenuY.value = event.clientY;
  tagContextMenu.value = true;
}

// Hide tag context menu
function hideTagContextMenu() {
  tagContextMenu.value = false;
}

// Handle edit tag
function handleEditTag() {
  if (!selectedTag.value) return;
  editingTag.value = selectedTag.value;
  showEditTagDialog.value = true;
  hideTagContextMenu();
}

// Handle delete tag
async function handleDeleteTag() {
  if (!selectedTag.value) return;

  if (!confirm(`确定要删除标签"${selectedTag.value.name}"吗？`)) {
    return;
  }

  try {
    await invokeCommand("delete_label", { id: selectedTag.value.id });
    await loadLabels();
  } catch (error) {
    console.error("Failed to delete tag:", error);
    alert(`删除标签失败: ${error}`);
  }
  hideTagContextMenu();
}

// Handle update tag color
async function handleUpdateTagColor(colorKey: string) {
  if (!selectedTag.value) return;

  try {
    await invokeCommand("update_label", {
      id: selectedTag.value.id,
      name: selectedTag.value.name,
      color: colorKey,
    });
    await loadLabels();
  } catch (error) {
    console.error("Failed to update tag color:", error);
    alert(`修改标签颜色失败: ${error}`);
  }
  hideTagContextMenu();
}

// Refresh after dialog operations
function refreshCategories() {
  // CategoryTree will handle its own refresh
}

function handleCategoryCreated() {
  showAddCategoryDialog.value = false;
  // Refresh will be handled by CategoryTree
}

function handleTagCreated() {
  showAddTagDialog.value = false;
  loadLabels();
}

function handleTagUpdated() {
  showEditTagDialog.value = false;
  editingTag.value = null;
  loadLabels();
}

// Get color display value from color key
function getColorDisplay(colorKey: string): string {
  return TAG_COLORS[colorKey] || TAG_COLORS.blue;
}

// Initialize on mount
onMounted(() => {
  loadLabels();
});
</script>

<template>
  <div class="navigation">
    <!-- Top Section: Library and Category Tree -->
    <div class="nav-top">
      <!-- Library Header -->
      <div
        class="nav-item library-header"
        :class="{ 'nav-item-active': activeNavItem === 'library' }"
        @click="handleNavClick('library')"
      >
        <v-icon size="small" class="nav-item-icon"> mdi-bookshelf </v-icon>
        <span class="nav-item-text">{{ t("navigation.library") }}</span>
        <v-btn
          icon="mdi-plus"
          size="x-small"
          variant="text"
          @click.stop="showAddCategoryDialog = true"
        />
      </div>

      <!-- Category Tree Component -->
      <CategoryTree @category-select="handleCategorySelect" />
    </div>

    <!-- Bottom Section: Tags, Favorites, Trash -->
    <div class="nav-bottom">
      <!-- Tags Section -->
      <div class="bottom-section tags-section">
        <div class="section-header">
          <v-icon size="small" class="mr-2">mdi-label</v-icon>
          <span class="text-caption text-grey">{{ t("navigation.tags") }}</span>
          <v-spacer />
          <v-btn
            icon="mdi-plus"
            size="x-small"
            variant="text"
            @click="showAddTagDialog = true"
          />
        </div>

        <!-- Tags as chips in a flex wrap layout -->
        <div class="tags-chips">
          <v-chip
            v-for="label in labels"
            :key="label.id"
            :color="label.color"
            size="small"
            variant="elevated"
            class="tag-chip"
            @click="handleLabelClick(label.id)"
            @contextmenu="showTagContextMenu($event, label)"
          >
            {{ label.name }}
          </v-chip>
        </div>
      </div>

      <v-divider />

      <!-- Favorites -->
      <div
        class="nav-item"
        :class="{ 'nav-item-active': activeNavItem === 'favorites' }"
        @click="handleNavClick('favorites')"
      >
        <v-icon size="small" class="nav-item-icon">mdi-star</v-icon>
        <span class="nav-item-text">{{ t("navigation.favorites") }}</span>
      </div>

      <v-divider />

      <!-- Trash -->
      <div
        class="nav-item"
        :class="{ 'nav-item-active': activeNavItem === 'trash' }"
        @click="handleNavClick('trash')"
      >
        <v-icon size="small" class="nav-item-icon">mdi-delete</v-icon>
        <span class="nav-item-text">{{ t("navigation.trash") }}</span>
      </div>
    </div>

    <!-- Tag Context Menu -->
    <v-menu
      v-model="tagContextMenu"
      :close-on-content-click="false"
      :style="{ top: tagContextMenuY + 'px', left: tagContextMenuX + 'px' }"
      absolute
    >
      <v-list density="compact">
        <v-list-item @click="handleEditTag">
          <template #prepend>
            <v-icon>mdi-pencil</v-icon>
          </template>
          <v-list-item-title>{{ t("dialog.editTag") }}</v-list-item-title>
        </v-list-item>
        <v-list-item @click="handleDeleteTag">
          <template #prepend>
            <v-icon color="error">mdi-delete</v-icon>
          </template>
          <v-list-item-title>{{ t("dialog.deleteTag") }}</v-list-item-title>
        </v-list-item>
        <v-divider />
        <v-list-subheader>{{ t("dialog.selectColor") }}</v-list-subheader>
        <v-list-item>
          <div class="color-palette">
            <div
              v-for="(color, key) in TAG_COLORS"
              :key="key"
              class="color-swatch"
              :class="{ 'color-swatch-active': selectedTag?.color === key }"
              :style="{ backgroundColor: color }"
              :title="key"
              @click="handleUpdateTagColor(key)"
            />
          </div>
        </v-list-item>
      </v-list>
    </v-menu>

    <!-- Add Category Dialog -->
    <AddCategoryDialog
      v-model="showAddCategoryDialog"
      @category-created="handleCategoryCreated"
    />

    <!-- Edit Category Dialog -->
    <EditCategoryDialog
      v-model="showEditCategoryDialog"
      @category-updated="refreshCategories"
    />

    <!-- Add Tag Dialog -->
    <AddTagDialog v-model="showAddTagDialog" @tag-created="handleTagCreated" />

    <!-- Edit Tag Dialog (reuse AddTagDialog for editing) -->
    <AddTagDialog
      v-if="showEditTagDialog && editingTag"
      v-model="showEditTagDialog"
      :tag-id="editingTag.id"
      :tag-name="editingTag.name"
      :tag-color="editingTag.color"
      @tag-created="handleTagUpdated"
    />
  </div>
</template>

<style scoped>
.navigation {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.nav-top {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.nav-bottom {
  flex: 0 0 auto;
}

.nav-item {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  cursor: pointer;
  user-select: none;
  transition: background-color 150ms;
}

.nav-item:hover {
  background-color: rgba(255, 255, 255, 0.08);
}

.nav-item-active {
  background-color: rgb(var(--v-theme-primary));
  color: rgb(var(--v-theme-on-primary));
}

.nav-item-active:hover {
  background-color: rgba(var(--v-theme-primary), 0.8);
}

.library-header {
  border-radius: 4px;
  margin: 4px 8px;
}

.nav-item-icon {
  margin-right: 8px;
}

.nav-item-text {
  flex: 1;
  font-size: 14px;
}

.bottom-section {
  padding: 8px 0;
}

.tags-section {
  padding-bottom: 8px;
}

.section-header {
  display: flex;
  align-items: center;
  padding: 4px 16px 8px;
}

.section-header .text-caption {
  flex: 1;
}

.tags-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 0 12px;
}

.tag-chip {
  cursor: pointer;
}

.tag-chip:hover {
  opacity: 0.8;
}

.color-palette {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 4px 0;
}

.color-swatch {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  cursor: pointer;
  border: 2px solid transparent;
  transition: transform 0.15s;
}

.color-swatch:hover {
  transform: scale(1.1);
}

.color-swatch-active {
  border-color: rgb(var(--v-theme-on-surface-variant));
}
</style>
