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

// State
const labels = ref<Label[]>([]);
const loading = ref(false);
const activeNavItem = ref<string>("library");

// Dialog states
const showAddCategoryDialog = ref(false);
const showEditCategoryDialog = ref(false);
const showAddTagDialog = ref(false);

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
      <v-divider />

      <!-- Tags Section -->
      <div class="bottom-section">
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

        <v-list density="compact">
          <v-list-item
            v-for="label in labels"
            :key="label.id"
            @click="handleLabelClick(label.id)"
          >
            <template #prepend>
              <v-icon :color="label.color" size="small">mdi-label</v-icon>
            </template>
            <v-list-item-title class="text-body-2">{{
              label.name
            }}</v-list-item-title>
          </v-list-item>
        </v-list>
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

.section-header {
  display: flex;
  align-items: center;
  padding: 4px 16px 8px;
}

.section-header .text-caption {
  flex: 1;
}
</style>
