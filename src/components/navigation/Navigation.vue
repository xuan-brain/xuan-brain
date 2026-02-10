<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invokeCommand } from "@/lib/tauri";
import { useI18n } from "@/lib/i18n";
import CategoryTree from "@/components/navigation/CategoryTree.vue";
import AddCategoryDialog from "@/components/dialogs/AddCategoryDialog.vue";
import EditCategoryDialog from "@/components/dialogs/EditCategoryDialog.vue";

const { t } = useI18n();

interface Label {
  id: number;
  name: string;
  color: string;
}

// State
const labels = ref<Label[]>([]);
const loading = ref(false);

// Dialog states
const showAddCategoryDialog = ref(false);
const showEditCategoryDialog = ref(false);

// Emit events
const emit = defineEmits<{
  categorySelect: [path: string | null];
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
  emit("categorySelect", path);
}

// Refresh after dialog operations
function refreshCategories() {
  // CategoryTree will handle its own refresh
}

// Initialize on mount
onMounted(() => {
  loadLabels();
});
</script>

<template>
  <div class="navigation">
    <!-- Categories Section -->
    <div class="nav-section">
      <div class="section-header">
        <span class="text-caption text-grey">{{ t("main.categories") }}</span>
      </div>

      <!-- Category Tree Component -->
      <CategoryTree @category-select="handleCategorySelect" />
    </div>

    <!-- Labels Section -->
    <div class="nav-section">
      <v-divider />
      <div class="section-header">
        <span class="text-caption text-grey">Labels</span>
      </div>

      <v-list density="compact">
        <v-list-item
          v-for="label in labels"
          :key="label.id"
          @click="handleCategorySelect(null)"
        >
          <template #prepend>
            <v-icon :color="label.color">mdi-label</v-icon>
          </template>
          <v-list-item-title>{{ label.name }}</v-list-item-title>
        </v-list-item>
      </v-list>
    </div>

    <!-- Add Category Dialog -->
    <AddCategoryDialog
      v-model="showAddCategoryDialog"
      @category-created="refreshCategories"
    />

    <!-- Edit Category Dialog -->
    <EditCategoryDialog
      v-model="showEditCategoryDialog"
      @category-updated="refreshCategories"
    />
  </div>
</template>

<style scoped>
.navigation {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.nav-section {
  flex: 0 0 auto;
}

.section-header {
  padding: 8px 16px;
}
</style>
