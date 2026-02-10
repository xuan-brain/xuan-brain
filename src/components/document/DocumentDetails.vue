<script setup lang="ts">
import { ref, watch } from "vue";
import { invokeCommand } from "@/lib/tauri";

interface Label {
  id: number;
  name: string;
  color: string;
}

interface PaperDetail {
  id: number;
  title: string;
  authors: string[];
  publication_year?: number;
  journal_name?: string;
  abstract?: string;
  notes?: string;
  read_status: string;
  doi?: string;
  url?: string;
  labels: Label[];
}

interface Props {
  paperId?: number | null;
}

const props = defineProps<Props>();

// State
const details = ref<PaperDetail | null>(null);
const loading = ref(false);
const isEditing = ref(false);
const allLabels = ref<Label[]>([]);

// Edit form state
const editForm = ref({
  title: "",
  publication_year: null as number | null,
  journal_name: "",
  abstract: "",
  notes: "",
  read_status: "unread",
  doi: "",
  url: "",
});

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
      publication_year: details.value.publication_year || null,
      journal_name: details.value.journal_name || "",
      abstract: details.value.abstract || "",
      notes: details.value.notes || "",
      read_status: details.value.read_status || "unread",
      doi: details.value.doi || "",
      url: details.value.url || "",
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

// Save changes
async function saveChanges() {
  if (!details.value) return;

  loading.value = true;
  try {
    await invokeCommand("update_paper_details", {
      id: details.value.id,
      ...editForm.value,
    });
    isEditing.value = false;
    await loadPaperDetails();
  } catch (error) {
    console.error("Failed to save changes:", error);
  } finally {
    loading.value = false;
  }
}

// Add label to paper
async function addLabel(labelId: number) {
  if (!details.value) return;

  try {
    await invokeCommand("add_paper_label", {
      paperId: details.value.id,
      labelId,
    });
    await loadPaperDetails();
  } catch (error) {
    console.error("Failed to add label:", error);
  }
}

// Remove label from paper
async function removeLabel(labelId: number) {
  if (!details.value) return;

  try {
    await invokeCommand("remove_paper_label", {
      paperId: details.value.id,
      labelId,
    });
    await loadPaperDetails();
  } catch (error) {
    console.error("Failed to remove label:", error);
  }
}

// Cancel edit
function cancelEdit() {
  if (details.value) {
    editForm.value = {
      title: details.value.title || "",
      publication_year: details.value.publication_year || null,
      journal_name: details.value.journal_name || "",
      abstract: details.value.abstract || "",
      notes: details.value.notes || "",
      read_status: details.value.read_status || "unread",
      doi: details.value.doi || "",
      url: details.value.url || "",
    };
  }
  isEditing.value = false;
}

// Watch paper ID changes
watch(
  () => props.paperId,
  () => {
    loadPaperDetails();
    loadLabels();
  },
  { immediate: true },
);
</script>

<template>
  <div class="document-details">
    <!-- Loading state -->
    <v-skeleton-loader v-if="loading && !details" type="article" />

    <!-- No paper selected -->
    <v-alert v-else-if="!paperId" type="info">
      Select a document to view details
    </v-alert>

    <!-- Details view -->
    <div v-else-if="details && !isEditing">
      <v-card>
        <v-card-title>{{ details.title }}</v-card-title>
        <v-card-subtitle>
          {{ details.authors?.join(", ") }}
        </v-card-subtitle>

        <v-card-text>
          <!-- Year and Journal -->
          <div class="mb-3">
            <v-chip v-if="details.publication_year" size="small" class="mr-2">
              {{ details.publication_year }}
            </v-chip>
            <v-chip v-if="details.journal_name" size="small">
              {{ details.journal_name }}
            </v-chip>
          </div>

          <!-- Labels -->
          <div class="mb-4">
            <div class="text-caption text-grey mb-1">Labels</div>
            <v-chip
              v-for="label in details.labels"
              :key="label.id"
              size="small"
              :color="label.color"
              closable
              class="mr-1 mb-1"
              @click:close="removeLabel(label.id)"
            >
              {{ label.name }}
            </v-chip>

            <v-menu>
              <template #activator="{ props: menuProps }">
                <v-chip size="small" v-bind="menuProps" class="mb-1">
                  <v-icon start>mdi-plus</v-icon>
                  Add Label
                </v-chip>
              </template>
              <v-list>
                <v-list-item
                  v-for="label in allLabels"
                  :key="label.id"
                  @click="addLabel(label.id)"
                >
                  <v-list-item-title>{{ label.name }}</v-list-item-title>
                </v-list-item>
              </v-list>
            </v-menu>
          </div>

          <!-- Abstract -->
          <div v-if="details.abstract" class="mb-4">
            <div class="text-caption text-grey mb-1">Abstract</div>
            <p class="text-body-2 mb-0">{{ details.abstract }}</p>
          </div>

          <!-- Notes -->
          <div v-if="details.notes">
            <div class="text-caption text-grey mb-1">Notes</div>
            <p class="text-body-2 mb-0">{{ details.notes }}</p>
          </div>
        </v-card-text>

        <v-card-actions>
          <v-btn @click="isEditing = true">Edit</v-btn>
        </v-card-actions>
      </v-card>
    </div>

    <!-- Edit mode -->
    <div v-else-if="details">
      <v-card>
        <v-card-title>Edit Document</v-card-title>

        <v-card-text>
          <v-text-field
            v-model="editForm.title"
            label="Title"
            variant="outlined"
            :disabled="loading"
          />

          <v-row>
            <v-col cols="6">
              <v-text-field
                v-model.number="editForm.publication_year"
                label="Year"
                type="number"
                variant="outlined"
                :disabled="loading"
              />
            </v-col>
            <v-col cols="6">
              <v-text-field
                v-model="editForm.journal_name"
                label="Journal"
                variant="outlined"
                :disabled="loading"
              />
            </v-col>
          </v-row>

          <v-select
            v-model="editForm.read_status"
            label="Read Status"
            :items="['unread', 'reading', 'read']"
            variant="outlined"
            :disabled="loading"
          />

          <v-textarea
            v-model="editForm.abstract"
            label="Abstract"
            rows="4"
            variant="outlined"
            :disabled="loading"
          />

          <v-textarea
            v-model="editForm.notes"
            label="Notes"
            rows="4"
            variant="outlined"
            :disabled="loading"
          />
        </v-card-text>

        <v-card-actions>
          <v-btn @click="cancelEdit" :disabled="loading">Cancel</v-btn>
          <v-btn color="primary" @click="saveChanges" :loading="loading"
            >Save</v-btn
          >
        </v-card-actions>
      </v-card>
    </div>
  </div>
</template>

<style scoped>
.document-details {
  height: 100%;
  overflow-y: auto;
  padding: 8px;
}
</style>
