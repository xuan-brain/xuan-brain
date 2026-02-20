<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { invokeCommand } from "@/lib/tauri";
import { useI18n } from "@/lib/i18n";

const { t } = useI18n();

interface Label {
  id: number;
  name: string;
  color: string;
}

interface CategoryNode {
  id: number;
  name: string;
  parent_id?: number | null;
  children?: CategoryNode[];
}

interface PaperDetail {
  id: number;
  title: string;
  authors: string[];
  publication_year?: number;
  publication_date?: string;
  journal_name?: string;
  conference_name?: string;
  volume?: string;
  issue?: string;
  pages?: string;
  doi?: string;
  url?: string;
  citation_count?: number;
  abstract_text?: string;
  notes?: string;
  read_status?: string;
  labels: Label[];
  category_id?: number;
  category_name?: string;
}

interface Props {
  paperId?: number | null;
}

const props = defineProps<Props>();

// Emit event for parent updates
const emit = defineEmits<{
  paperUpdated: [paper: PaperDetail];
}>();

// State
const details = ref<PaperDetail | null>(null);
const loading = ref(false);
const isEditing = ref(false);
const allLabels = ref<Label[]>([]);
const categories = ref<CategoryNode[]>([]);
const addingLabel = ref(false);
const addingCategory = ref(false);
const actionLoading = ref(false);

// Edit form state
const editForm = ref<Partial<PaperDetail>>({
  title: "",
  publication_year: undefined,
  journal_name: "",
  conference_name: "",
  volume: "",
  issue: "",
  pages: "",
  doi: "",
  url: "",
  abstract_text: "",
  notes: "",
  read_status: "unread",
  category_id: undefined,
});

// Tree data for category select
const treeCategories = computed(() => buildTreeData(categories.value));

// Build tree data from flat category list
function buildTreeData(flat: CategoryNode[]): CategoryNode[] {
  const nodeMap = new Map<number, CategoryNode>();
  const root: CategoryNode[] = [];

  // First pass: create nodes
  flat.forEach((cat) => {
    nodeMap.set(cat.id, { ...cat, children: [] });
  });

  // Second pass: build hierarchy
  flat.forEach((cat) => {
    const node = nodeMap.get(cat.id)!;
    if (cat.parent_id && nodeMap.has(cat.parent_id)) {
      const parent = nodeMap.get(cat.parent_id)!;
      parent.children!.push(node);
    } else {
      root.push(node);
    }
  });

  return root;
}

// Load paper details
async function loadPaperDetails() {
  if (!props.paperId) return;

  loading.value = true;
  try {
    details.value = await invokeCommand<PaperDetail>("get_paper", {
      id: props.paperId,
    });

    // Initialize edit form
    editForm.value = {
      title: details.value.title || "",
      publication_year: details.value.publication_year,
      journal_name: details.value.journal_name || "",
      conference_name: details.value.conference_name || "",
      volume: details.value.volume || "",
      issue: details.value.issue || "",
      pages: details.value.pages || "",
      doi: details.value.doi || "",
      url: details.value.url || "",
      abstract_text: details.value.abstract_text || "",
      notes: details.value.notes || "",
      read_status: details.value.read_status || "unread",
      category_id: details.value.category_id,
    };
  } catch (error) {
    console.error("Failed to load paper details:", error);
  } finally {
    loading.value = false;
  }
}

// Load all labels
async function loadLabels() {
  try {
    allLabels.value = await invokeCommand<Label[]>("get_all_labels");
  } catch (error) {
    console.error("Failed to load labels:", error);
  }
}

// Load categories
async function loadCategories() {
  try {
    const cats = await invokeCommand<CategoryNode[]>("load_categories");
    categories.value = cats;
  } catch (error) {
    console.error("Failed to load categories:", error);
  }
}

// Notify parent of update
function notifyUpdate(data: PaperDetail) {
  emit("paperUpdated", data);
}

// Set category for paper
async function handleSetCategory(categoryId: number) {
  if (!details.value) return;
  actionLoading.value = true;
  try {
    await invokeCommand("update_paper_category", {
      paperId: details.value.id,
      categoryId: categoryId,
    });
    await loadPaperDetails();
    if (details.value) notifyUpdate(details.value);
    addingCategory.value = false;
  } catch (error) {
    console.error("Failed to update category:", error);
  } finally {
    actionLoading.value = false;
  }
}

// Add label to paper
async function handleAddLabel(labelId: number) {
  if (!details.value) return;
  actionLoading.value = true;
  try {
    await invokeCommand("add_paper_label", {
      paperId: details.value.id,
      labelId: labelId,
    });
    await loadPaperDetails();
    if (details.value) notifyUpdate(details.value);
    addingLabel.value = false;
  } catch (error) {
    console.error("Failed to add label:", error);
  } finally {
    actionLoading.value = false;
  }
}

// Remove label from paper
async function handleRemoveLabel(labelId: number) {
  if (!details.value) return;
  try {
    await invokeCommand("remove_paper_label", {
      paperId: details.value.id,
      labelId: labelId,
    });
    await loadPaperDetails();
    if (details.value) notifyUpdate(details.value);
  } catch (error) {
    console.error("Failed to remove label:", error);
  }
}

// Start edit mode
function startEdit() {
  editForm.value = { ...details.value! };
  isEditing.value = true;
}

// Cancel edit
function cancelEdit() {
  if (details.value) {
    editForm.value = {
      title: details.value.title || "",
      publication_year: details.value.publication_year,
      journal_name: details.value.journal_name || "",
      conference_name: details.value.conference_name || "",
      volume: details.value.volume || "",
      issue: details.value.issue || "",
      pages: details.value.pages || "",
      doi: details.value.doi || "",
      url: details.value.url || "",
      abstract_text: details.value.abstract_text || "",
      notes: details.value.notes || "",
      read_status: details.value.read_status || "unread",
      category_id: details.value.category_id,
    };
  }
  isEditing.value = false;
}

// Save changes
async function saveChanges() {
  if (!details.value) return;

  actionLoading.value = true;
  try {
    // Update basic details
    await invokeCommand("update_paper_details", {
      id: details.value.id,
      title: editForm.value.title,
      publication_year: editForm.value.publication_year,
      journal_name: editForm.value.journal_name,
      conference_name: editForm.value.conference_name,
      volume: editForm.value.volume,
      issue: editForm.value.issue,
      pages: editForm.value.pages,
      doi: editForm.value.doi,
      url: editForm.value.url,
      abstract_text: editForm.value.abstract_text,
      notes: editForm.value.notes,
      read_status: editForm.value.read_status,
    });

    // Update category if changed
    if (editForm.value.category_id !== details.value.category_id) {
      await invokeCommand("update_paper_category", {
        paperId: details.value.id,
        categoryId: editForm.value.category_id || null,
      });
    }

    await loadPaperDetails();
    if (details.value) notifyUpdate(details.value);
    isEditing.value = false;
  } catch (error) {
    console.error("Failed to save changes:", error);
  } finally {
    actionLoading.value = false;
  }
}

// Get available labels (excluding already added)
const availableLabels = computed(() => {
  if (!details.value) return allLabels.value;
  return allLabels.value.filter(
    (l) => !details.value!.labels.some((pl) => pl.id === l.id),
  );
});

// Read status options
const readStatusOptions = [
  { title: "Unread", value: "unread" },
  { title: "Reading", value: "reading" },
  { title: "Read", value: "read" },
];

// Watch paper ID changes
watch(
  () => props.paperId,
  () => {
    if (props.paperId) {
      loadPaperDetails();
      loadLabels();
      loadCategories();
    } else {
      details.value = null;
      isEditing.value = false;
      addingCategory.value = false;
      addingLabel.value = false;
    }
  },
  { immediate: true },
);
</script>

<template>
  <div class="paper-details">
    <!-- Loading state -->
    <div v-if="loading && !details" class="loading-container">
      <v-progress-circular indeterminate size="48" />
    </div>

    <!-- No paper selected - show blank -->
    <div v-else-if="!paperId" class="no-selection"></div>

    <!-- Details view -->
    <div v-else-if="details && !isEditing" class="details-view">
      <!-- Header Actions -->
      <div class="header-actions">
        <v-btn variant="text" size="small" @click="startEdit">
          <v-icon start>mdi-pencil</v-icon>
          Edit
        </v-btn>
      </div>

      <!-- Title -->
      <div class="detail-section">
        <h2 class="paper-title">{{ details.title }}</h2>
      </div>

      <!-- Metadata Tags -->
      <div class="metadata-tags">
        <!-- Category -->
        <template v-if="addingCategory">
          <v-select
            v-model="details.category_id"
            :items="treeCategories"
            item-title="name"
            item-value="id"
            label="Select Category"
            density="compact"
            variant="outlined"
            hide-details
            auto
            @update:model-value="handleSetCategory"
          >
            <template v-if="details.category_id" #append-inner>
              <v-icon @click.stop="addingCategory = false">mdi-close</v-icon>
            </template>
          </v-select>
        </template>
        <v-chip
          v-else-if="details.category_name"
          color="orange"
          class="clickable"
          @click="addingCategory = true"
        >
          <v-icon start size="small">mdi-folder-open</v-icon>
          {{ details.category_name }}
        </v-chip>
        <v-chip v-else class="clickable dashed" @click="addingCategory = true">
          <v-icon start size="small">mdi-plus</v-icon>
          {{ t("dialog.addCategory") || "Add Category" }}
        </v-chip>

        <!-- Year -->
        <v-chip v-if="details.publication_year" size="small">
          {{ details.publication_year }}
        </v-chip>

        <!-- Journal/Conference -->
        <v-chip
          v-if="details.journal_name || details.conference_name"
          color="blue"
          size="small"
        >
          {{ details.journal_name || details.conference_name }}
        </v-chip>

        <!-- Read Status -->
        <v-chip
          v-if="details.read_status"
          :color="details.read_status === 'read' ? 'success' : 'default'"
          size="small"
        >
          {{ details.read_status }}
        </v-chip>
      </div>

      <!-- Authors -->
      <div class="authors">
        <span class="text-body-2 text-grey">{{
          details.authors?.join(", ")
        }}</span>
      </div>

      <!-- Labels -->
      <div class="labels-section">
        <v-chip
          v-for="label in details.labels"
          :key="label.id"
          :color="label.color"
          closable
          size="small"
          class="mr-2 mb-2"
          @click:close="handleRemoveLabel(label.id)"
        >
          {{ label.name }}
        </v-chip>

        <!-- Add Label -->
        <template v-if="addingLabel">
          <v-select
            :items="availableLabels"
            item-title="name"
            item-value="id"
            label="Select label"
            density="compact"
            variant="outlined"
            hide-details
            auto
            style="max-width: 150px"
            @update:model-value="handleAddLabel"
          >
            <template #append-inner>
              <v-icon @click.stop="addingLabel = false">mdi-close</v-icon>
            </template>
          </v-select>
        </template>
        <v-chip
          v-else
          class="clickable dashed"
          @click="
            addingLabel = true;
            loadLabels();
          "
        >
          <v-icon start size="small">mdi-plus</v-icon>
          {{ t("dialog.addTag") || "Add Tag" }}
        </v-chip>
      </div>

      <!-- DOI/URL -->
      <div v-if="details.doi || details.url" class="links-section">
        <div v-if="details.doi" class="link-item">
          <span class="text-caption text-grey">DOI: </span>
          <a
            :href="`https://doi.org/${details.doi}`"
            target="_blank"
            rel="noopener noreferrer"
            class="text-caption"
          >
            {{ details.doi }}
          </a>
        </div>
        <div v-if="details.url" class="link-item">
          <span class="text-caption text-grey">URL: </span>
          <a
            :href="details.url"
            target="_blank"
            rel="noopener noreferrer"
            class="text-caption"
          >
            {{ details.url }}
          </a>
        </div>
      </div>

      <v-divider class="my-3" />

      <!-- Abstract -->
      <div v-if="details.abstract_text" class="detail-section">
        <div class="section-label">Abstract</div>
        <p class="text-body-2 abstract-text">{{ details.abstract_text }}</p>
      </div>

      <!-- Notes -->
      <div v-if="details.notes" class="detail-section">
        <div class="section-label">Notes</div>
        <v-card variant="tonal" density="compact" class="notes-card">
          <p class="text-body-2 mb-0 notes-text">{{ details.notes }}</p>
        </v-card>
      </div>

      <!-- Footer info -->
      <div class="footer-info">
        <span class="text-caption text-grey">
          ID: {{ details.id }} | Citations: {{ details.citation_count || 0 }}
        </span>
      </div>
    </div>

    <!-- Edit mode -->
    <div v-else-if="details" class="edit-view">
      <div class="header-actions">
        <v-btn @click="cancelEdit" :disabled="actionLoading" variant="tonal">
          <v-icon start>mdi-close</v-icon>
          Cancel
        </v-btn>
        <v-btn color="primary" @click="saveChanges" :loading="actionLoading">
          <v-icon start>mdi-content-save</v-icon>
          Save
        </v-btn>
      </div>

      <div class="edit-form">
        <!-- Title -->
        <v-textarea
          v-model="editForm.title"
          label="Title"
          rows="2"
          variant="outlined"
          :disabled="actionLoading"
          auto-grow
          class="title-input"
        />

        <!-- Metadata Row -->
        <v-row class="form-row">
          <v-col cols="6">
            <v-select
              v-model="editForm.category_id"
              :items="treeCategories"
              item-title="name"
              item-value="id"
              label="Category"
              variant="outlined"
              :disabled="actionLoading"
              clearable
            />
          </v-col>
          <v-col cols="3">
            <v-text-field
              v-model.number="editForm.publication_year"
              label="Year"
              type="number"
              variant="outlined"
              :disabled="actionLoading"
            />
          </v-col>
          <v-col cols="3">
            <v-select
              v-model="editForm.read_status"
              label="Read Status"
              :items="readStatusOptions"
              variant="outlined"
              :disabled="actionLoading"
            />
          </v-col>
        </v-row>

        <v-row class="form-row">
          <v-col cols="6">
            <v-text-field
              v-model="editForm.journal_name"
              label="Journal"
              variant="outlined"
              :disabled="actionLoading"
            />
          </v-col>
          <v-col cols="6">
            <v-text-field
              v-model="editForm.conference_name"
              label="Conference"
              variant="outlined"
              :disabled="actionLoading"
            />
          </v-col>
        </v-row>

        <!-- Additional Fields -->
        <div class="additional-fields">
          <v-row>
            <v-col cols="6">
              <v-text-field
                v-model="editForm.doi"
                label="DOI"
                variant="outlined"
                :disabled="actionLoading"
                prepend-inner-icon="mdi-identifier"
              />
            </v-col>
            <v-col cols="6">
              <v-text-field
                v-model="editForm.url"
                label="URL"
                variant="outlined"
                :disabled="actionLoading"
                prepend-inner-icon="mdi-link"
              />
            </v-col>
          </v-row>
          <v-row>
            <v-col cols="4">
              <v-text-field
                v-model="editForm.volume"
                label="Volume"
                variant="outlined"
                :disabled="actionLoading"
              />
            </v-col>
            <v-col cols="4">
              <v-text-field
                v-model="editForm.issue"
                label="Issue"
                variant="outlined"
                :disabled="actionLoading"
              />
            </v-col>
            <v-col cols="4">
              <v-text-field
                v-model="editForm.pages"
                label="Pages"
                variant="outlined"
                :disabled="actionLoading"
              />
            </v-col>
          </v-row>
        </div>

        <!-- Abstract -->
        <v-textarea
          v-model="editForm.abstract_text"
          label="Abstract"
          rows="5"
          variant="outlined"
          :disabled="actionLoading"
          auto-grow
        />

        <!-- Notes -->
        <v-textarea
          v-model="editForm.notes"
          label="Notes"
          rows="4"
          variant="outlined"
          :disabled="actionLoading"
          placeholder="Add notes..."
          auto-grow
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.paper-details {
  height: 100%;
  overflow-y: auto;
  padding: 16px;
}

.loading-container {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.no-selection {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.details-view,
.edit-view {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.header-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-bottom: 16px;
}

.paper-title {
  font-size: 18px;
  font-weight: 600;
  margin: 0 0 16px 0;
  line-height: 1.4;
}

.metadata-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 12px;
}

.metadata-tags .v-chip {
  height: 24px;
}

.clickable {
  cursor: pointer;
}

.dashed {
  border-style: dashed !important;
  background-color: transparent !important;
}

.authors {
  margin-bottom: 16px;
}

.labels-section {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  margin-bottom: 16px;
}

.links-section {
  margin-bottom: 16px;
}

.link-item {
  margin-bottom: 4px;
}

.link-item:last-child {
  margin-bottom: 0;
}

.detail-section {
  margin-bottom: 20px;
}

.section-label {
  font-weight: 500;
  margin-bottom: 8px;
  font-size: 14px;
}

.abstract-text {
  white-space: pre-wrap;
  line-height: 1.6;
}

.notes-card {
  padding: 12px;
}

.notes-text {
  white-space: pre-wrap;
  font-size: 13px;
}

.footer-info {
  margin-top: auto;
  padding-top: 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.12);
}

.edit-form {
  flex: 1;
}

.form-row {
  margin-bottom: 0;
}

.additional-fields {
  margin-bottom: 16px;
  padding: 12px;
  background-color: rgba(var(--v-theme-surface-variant), 0.3);
  border-radius: 8px;
}

.title-input :deep(.v-field__input) {
  font-size: 16px;
  font-weight: 500;
}
</style>
