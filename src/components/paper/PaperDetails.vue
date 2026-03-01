<script setup lang="ts">
  import { useI18n } from '@/lib/i18n';
  import { invokeCommand } from '@/lib/tauri';
  import { computed, ref, watch } from 'vue';

  const { t } = useI18n();

  interface Label {
    id: string;
    name: string;
    color: string;
  }

  interface Attachment {
    id: string;
    paper_id: string;
    file_name?: string;
    file_type?: string;
    created_at?: string;
  }

  interface CategoryNode {
    id: string;
    name: string;
    parent_id?: string | null;
    children?: CategoryNode[];
  }

  interface PaperDetail {
    id: string;
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
    category_id?: string;
    category_name?: string;
    attachments: Attachment[];
    attachment_count: number;
    created_at?: string;
    updated_at?: string;
  }

  interface Props {
    paperId?: string | null;
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
    title: '',
    publication_year: undefined,
    journal_name: '',
    conference_name: '',
    volume: '',
    issue: '',
    pages: '',
    doi: '',
    url: '',
    abstract_text: '',
    notes: '',
    read_status: 'unread',
    category_id: undefined,
  });

  // Tree data for category select
  const treeCategories = computed(() => buildTreeData(categories.value));

  // Build tree data from flat category list
  function buildTreeData(flat: CategoryNode[]): CategoryNode[] {
    const nodeMap = new Map<string, CategoryNode>();
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
      details.value = await invokeCommand<PaperDetail>('get_paper', {
        id: props.paperId,
      });

      // Initialize edit form
      editForm.value = {
        title: details.value.title || '',
        publication_year: details.value.publication_year,
        journal_name: details.value.journal_name || '',
        conference_name: details.value.conference_name || '',
        volume: details.value.volume || '',
        issue: details.value.issue || '',
        pages: details.value.pages || '',
        doi: details.value.doi || '',
        url: details.value.url || '',
        abstract_text: details.value.abstract_text || '',
        notes: details.value.notes || '',
        read_status: details.value.read_status || 'unread',
        category_id: details.value.category_id,
      };
    } catch (error) {
      console.error('Failed to load paper details:', error);
    } finally {
      loading.value = false;
    }
  }

  // Load all labels
  async function loadLabels() {
    try {
      allLabels.value = await invokeCommand<Label[]>('get_all_labels');
    } catch (error) {
      console.error('Failed to load labels:', error);
    }
  }

  // Load categories
  async function loadCategories() {
    try {
      const cats = await invokeCommand<CategoryNode[]>('load_categories');
      categories.value = cats;
    } catch (error) {
      console.error('Failed to load categories:', error);
    }
  }

  // Notify parent of update
  function notifyUpdate(data: PaperDetail) {
    emit('paperUpdated', data);
  }

  // Set category for paper
  async function handleSetCategory(categoryId: string) {
    if (!details.value) return;
    actionLoading.value = true;
    try {
      await invokeCommand('update_paper_category', {
        paperId: details.value.id,
        categoryId: categoryId,
      });
      await loadPaperDetails();
      if (details.value) notifyUpdate(details.value);
      addingCategory.value = false;
    } catch (error) {
      console.error('Failed to update category:', error);
    } finally {
      actionLoading.value = false;
    }
  }

  // Add label to paper
  async function handleAddLabel(labelId: string) {
    if (!details.value) return;
    actionLoading.value = true;
    try {
      await invokeCommand('add_paper_label', {
        paperId: details.value.id,
        labelId: labelId,
      });
      await loadPaperDetails();
      if (details.value) notifyUpdate(details.value);
      addingLabel.value = false;
    } catch (error) {
      console.error('Failed to add label:', error);
    } finally {
      actionLoading.value = false;
    }
  }

  // Remove label from paper
  async function handleRemoveLabel(labelId: string) {
    if (!details.value) return;
    try {
      await invokeCommand('remove_paper_label', {
        paperId: details.value.id,
        labelId: labelId,
      });
      await loadPaperDetails();
      if (details.value) notifyUpdate(details.value);
    } catch (error) {
      console.error('Failed to remove label:', error);
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
        title: details.value.title || '',
        publication_year: details.value.publication_year,
        journal_name: details.value.journal_name || '',
        conference_name: details.value.conference_name || '',
        volume: details.value.volume || '',
        issue: details.value.issue || '',
        pages: details.value.pages || '',
        doi: details.value.doi || '',
        url: details.value.url || '',
        abstract_text: details.value.abstract_text || '',
        notes: details.value.notes || '',
        read_status: details.value.read_status || 'unread',
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
      await invokeCommand('update_paper_details', {
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
        await invokeCommand('update_paper_category', {
          paperId: details.value.id,
          categoryId: editForm.value.category_id || null,
        });
      }

      await loadPaperDetails();
      if (details.value) notifyUpdate(details.value);
      isEditing.value = false;
    } catch (error) {
      console.error('Failed to save changes:', error);
    } finally {
      actionLoading.value = false;
    }
  }

  // Get available labels (excluding already added)
  const availableLabels = computed(() => {
    if (!details.value) return allLabels.value;
    return allLabels.value.filter((l) => !details.value!.labels.some((pl) => pl.id === l.id));
  });

  // Read status options
  const readStatusOptions = [
    { title: 'Unread', value: 'unread' },
    { title: 'Reading', value: 'reading' },
    { title: 'Read', value: 'read' },
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
    { immediate: true }
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
      <h2 class="paper-title">{{ details.title }}</h2>

      <!-- Tags Row -->
      <div class="tags-row">
        <!-- Category -->
        <template v-if="addingCategory">
          <v-select
            v-model="details.category_id"
            :items="treeCategories"
            item-title="name"
            item-value="id"
            density="compact"
            variant="outlined"
            hide-details
            style="max-width: 150px"
            @update:model-value="handleSetCategory"
          />
        </template>
        <v-chip
          v-else-if="details.category_name"
          size="small"
          color="orange"
          class="clickable"
          @click="addingCategory = true"
        >
          <v-icon start size="x-small">mdi-folder</v-icon>
          {{ details.category_name }}
        </v-chip>
        <v-chip v-else size="small" variant="outlined" class="clickable" @click="addingCategory = true">
          <v-icon start size="x-small">mdi-plus</v-icon>
          Category
        </v-chip>

        <!-- Read Status -->
        <v-chip
          v-if="details.read_status"
          size="small"
          :color="details.read_status === 'read' ? 'success' : details.read_status === 'reading' ? 'primary' : 'default'"
        >
          {{ details.read_status }}
        </v-chip>

        <!-- Labels -->
        <v-chip
          v-for="label in details.labels"
          :key="label.id"
          :color="label.color"
          size="small"
          closable
          class="mr-1"
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
            density="compact"
            variant="outlined"
            hide-details
            style="max-width: 120px"
            @update:model-value="handleAddLabel"
          />
        </template>
        <v-chip v-else size="small" variant="outlined" class="clickable" @click="addingLabel = true; loadLabels()">
          <v-icon start size="x-small">mdi-tag-plus</v-icon>
          Tag
        </v-chip>
      </div>

      <!-- Properties Table -->
      <table class="props-table">
        <tbody>
          <tr v-if="details.authors?.length">
            <td class="prop-label">Authors</td>
            <td class="prop-value">
              <div v-for="(author, i) in details.authors" :key="i" class="author-line">
                {{ author }}
              </div>
            </td>
          </tr>
          <tr v-if="details.journal_name">
            <td class="prop-label">Journal</td>
            <td class="prop-value">{{ details.journal_name }}</td>
          </tr>
          <tr v-if="details.conference_name">
            <td class="prop-label">Conference</td>
            <td class="prop-value">{{ details.conference_name }}</td>
          </tr>
          <tr v-if="details.publication_year">
            <td class="prop-label">Year</td>
            <td class="prop-value">{{ details.publication_year }}</td>
          </tr>
          <tr v-if="details.publication_date">
            <td class="prop-label">Date</td>
            <td class="prop-value">{{ details.publication_date }}</td>
          </tr>
          <tr v-if="details.volume || details.issue || details.pages">
            <td class="prop-label">Volume</td>
            <td class="prop-value">
              <span v-if="details.volume">{{ details.volume }}</span>
              <span v-if="details.issue"> ({{ details.issue }})</span>
              <span v-if="details.pages">, pp.{{ details.pages }}</span>
            </td>
          </tr>
          <tr v-if="details.doi">
            <td class="prop-label">DOI</td>
            <td class="prop-value">
              <a :href="`https://doi.org/${details.doi}`" target="_blank" class="link">{{ details.doi }}</a>
            </td>
          </tr>
          <tr v-if="details.url">
            <td class="prop-label">URL</td>
            <td class="prop-value">
              <a :href="details.url" target="_blank" class="link">{{ details.url }}</a>
            </td>
          </tr>
          <tr v-if="details.attachments?.length">
            <td class="prop-label">Files</td>
            <td class="prop-value">
              <div v-for="att in details.attachments" :key="att.id" class="file-item">
                <v-icon size="small" :color="att.file_type === 'pdf' ? 'error' : 'grey'">
                  {{ att.file_type === 'pdf' ? 'mdi-file-pdf-box' : 'mdi-file-document-outline' }}
                </v-icon>
                <span>{{ att.file_name }}</span>
              </div>
            </td>
          </tr>
          <tr>
            <td class="prop-label">Citations</td>
            <td class="prop-value">{{ details.citation_count || 0 }}</td>
          </tr>
        </tbody>
      </table>

      <!-- Abstract -->
      <div v-if="details.abstract_text" class="content-section">
        <div class="section-title">Abstract</div>
        <p class="section-text">{{ details.abstract_text }}</p>
      </div>

      <!-- Notes -->
      <div v-if="details.notes" class="content-section">
        <div class="section-title">Notes</div>
        <p class="section-text notes">{{ details.notes }}</p>
      </div>

      <!-- Footer -->
      <div class="footer">
        <span>ID: {{ details.id }}</span>
        <span v-if="details.created_at">Created: {{ new Date(details.created_at).toLocaleDateString() }}</span>
        <span v-if="details.updated_at">Updated: {{ new Date(details.updated_at).toLocaleDateString() }}</span>
      </div>
    </div>

    <!-- Edit mode -->
    <div v-else-if="details" class="edit-view">
      <div class="header-actions">
        <v-btn @click="cancelEdit" :disabled="actionLoading" variant="tonal" size="small">
          <v-icon start>mdi-close</v-icon>
          Cancel
        </v-btn>
        <v-btn color="primary" @click="saveChanges" :loading="actionLoading" size="small">
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
          class="mb-3"
        />

        <!-- Basic Info -->
        <v-row dense>
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
              density="compact"
            />
          </v-col>
          <v-col cols="3">
            <v-text-field
              v-model.number="editForm.publication_year"
              label="Year"
              type="number"
              variant="outlined"
              :disabled="actionLoading"
              density="compact"
            />
          </v-col>
          <v-col cols="3">
            <v-select
              v-model="editForm.read_status"
              label="Status"
              :items="readStatusOptions"
              variant="outlined"
              :disabled="actionLoading"
              density="compact"
            />
          </v-col>
        </v-row>

        <v-row dense>
          <v-col cols="6">
            <v-text-field
              v-model="editForm.journal_name"
              label="Journal"
              variant="outlined"
              :disabled="actionLoading"
              density="compact"
            />
          </v-col>
          <v-col cols="6">
            <v-text-field
              v-model="editForm.conference_name"
              label="Conference"
              variant="outlined"
              :disabled="actionLoading"
              density="compact"
            />
          </v-col>
        </v-row>

        <v-row dense>
          <v-col cols="4">
            <v-text-field
              v-model="editForm.volume"
              label="Volume"
              variant="outlined"
              :disabled="actionLoading"
              density="compact"
            />
          </v-col>
          <v-col cols="4">
            <v-text-field
              v-model="editForm.issue"
              label="Issue"
              variant="outlined"
              :disabled="actionLoading"
              density="compact"
            />
          </v-col>
          <v-col cols="4">
            <v-text-field
              v-model="editForm.pages"
              label="Pages"
              variant="outlined"
              :disabled="actionLoading"
              density="compact"
            />
          </v-col>
        </v-row>

        <v-row dense>
          <v-col cols="6">
            <v-text-field
              v-model="editForm.doi"
              label="DOI"
              variant="outlined"
              :disabled="actionLoading"
              density="compact"
            />
          </v-col>
          <v-col cols="6">
            <v-text-field
              v-model="editForm.url"
              label="URL"
              variant="outlined"
              :disabled="actionLoading"
              density="compact"
            />
          </v-col>
        </v-row>

        <!-- Abstract -->
        <v-textarea
          v-model="editForm.abstract_text"
          label="Abstract"
          rows="4"
          variant="outlined"
          :disabled="actionLoading"
          auto-grow
          class="mt-2"
        />

        <!-- Notes -->
        <v-textarea
          v-model="editForm.notes"
          label="Notes"
          rows="3"
          variant="outlined"
          :disabled="actionLoading"
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
  }

  .details-view,
  .edit-view {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .header-actions {
    display: flex;
    justify-content: flex-end;
    margin-bottom: 8px;
  }

  .paper-title {
    font-size: 16px;
    font-weight: 600;
    line-height: 1.4;
    margin: 0 0 12px 0;
    color: rgba(0, 0, 0, 0.87);
  }

  :global([data-theme='dark']) .paper-title {
    color: rgba(255, 255, 255, 0.95);
  }

  .tags-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    margin-bottom: 16px;
  }

  .clickable {
    cursor: pointer;
  }

  /* Properties Table */
  .props-table {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: 16px;
    font-size: 14px;
  }

  .props-table td {
    padding: 8px 0;
    vertical-align: top;
    border-bottom: 1px solid rgba(128, 128, 128, 0.15);
  }

  .props-table tr:last-child td {
    border-bottom: none;
  }

  .prop-label {
    width: 80px;
    color: rgba(0, 0, 0, 0.87);
    font-weight: 700;
    white-space: nowrap;
    font-size: 13px;
  }

  :global([data-theme='dark']) .prop-label {
    color: rgba(255, 255, 255, 0.9);
  }

  .prop-value {
    color: rgba(0, 0, 0, 0.87);
    word-break: break-word;
    font-weight: 500;
  }

  :global([data-theme='dark']) .prop-value {
    color: rgba(255, 255, 255, 0.9);
  }

  .author-line {
    line-height: 1.8;
  }

  .prop-sub {
    color: rgba(0, 0, 0, 0.5);
    font-size: 13px;
  }

  :global([data-theme='dark']) .prop-sub {
    color: rgba(255, 255, 255, 0.5);
  }

  .link {
    color: #1976d2;
    text-decoration: none;
  }

  :global([data-theme='dark']) .link {
    color: #64b5f6;
  }

  .link:hover {
    text-decoration: underline;
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 0;
  }

  /* Content Sections */
  .content-section {
    margin-bottom: 16px;
  }

  .section-title {
    font-size: 12px;
    font-weight: 600;
    color: rgba(0, 0, 0, 0.6);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 8px;
  }

  :global([data-theme='dark']) .section-title {
    color: rgba(255, 255, 255, 0.6);
  }

  .section-text {
    font-size: 14px;
    line-height: 1.7;
    margin: 0;
    white-space: pre-wrap;
    color: rgba(0, 0, 0, 0.8);
  }

  :global([data-theme='dark']) .section-text {
    color: rgba(255, 255, 255, 0.85);
  }

  .section-text.notes {
    padding: 10px 12px;
    background: rgba(25, 118, 210, 0.08);
    border-radius: 6px;
    border-left: 3px solid #1976d2;
  }

  :global([data-theme='dark']) .section-text.notes {
    background: rgba(100, 181, 246, 0.1);
    border-left-color: #64b5f6;
  }

  /* Footer */
  .footer {
    margin-top: auto;
    padding-top: 12px;
    display: flex;
    gap: 16px;
    font-size: 11px;
    color: rgba(0, 0, 0, 0.5);
    border-top: 1px solid rgba(128, 128, 128, 0.15);
  }

  :global([data-theme='dark']) .footer {
    color: rgba(255, 255, 255, 0.5);
  }

  /* Edit Form */
  .edit-form {
    flex: 1;
    overflow-y: auto;
  }
</style>
