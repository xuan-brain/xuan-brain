<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useI18n } from "@/lib/i18n";
import { invokeCommand } from "@/lib/tauri";
import type { UnlistenFn } from "@tauri-apps/api/event";

const { t } = useI18n();

// Types
interface DataFolderInfo {
  current_path: string;
  config_path: string;
  files_path: string;
  cache_path: string;
  logs_path: string;
  is_custom: boolean;
  default_path: string;
  total_size: number;
}

interface ValidationResult {
  valid: boolean;
  warnings: string[];
  error?: string;
}

interface MigrationStatus {
  phase: string;
  current_file: string | null;
  total_files: number;
  processed_files: number;
  error: string | null;
}

// Database migration types
interface DbMigrationStatus {
  can_migrate: boolean;
  sqlite_papers_count: number;
  surreal_papers_count: number;
  message: string;
}

interface DbMigrationReport {
  papers_migrated: number;
  authors_migrated: number;
  keywords_migrated: number;
  labels_migrated: number;
  categories_migrated: number;
  attachments_migrated: number;
  paper_author_relations: number;
  paper_label_relations: number;
  paper_category_relations: number;
  errors: string[];
  duration_ms: number;
}

// State
const dataFolderInfo = ref<DataFolderInfo | null>(null);
const loading = ref(false);
const selectedPath = ref("");
const validation = ref<ValidationResult | null>(null);
const validating = ref(false);

// Dialogs
const showMigrationDialog = ref(false);
const migrating = ref(false);
const migrationStatus = ref<MigrationStatus | null>(null);

// Database migration state
const dbMigrationStatus = ref<DbMigrationStatus | null>(null);
const dbMigrationReport = ref<DbMigrationReport | null>(null);
const dbMigrating = ref(false);
const showDbMigrationDialog = ref(false);

// Event listener
let unlisten: UnlistenFn | null = null;

// Computed
const formattedDataSize = computed(() => {
  if (!dataFolderInfo.value) return "0 B";
  return formatBytes(dataFolderInfo.value.total_size);
});

const migrationProgress = computed(() => {
  if (!migrationStatus.value) return 0;
  if (migrationStatus.value.total_files === 0) return 0;
  return Math.round(
    (migrationStatus.value.processed_files / migrationStatus.value.total_files) * 100
  );
});

const migrationPhaseText = computed(() => {
  if (!migrationStatus.value) return "";
  const phaseKey = `settings.migrationPhase.${migrationStatus.value.phase}`;
  const translated = t(phaseKey);
  // If translation returns the key itself or undefined, return the phase name
  return translated && translated !== phaseKey ? translated : migrationStatus.value.phase;
});

// Helper functions
function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

// Load data folder info
async function loadDataFolderInfo() {
  loading.value = true;
  try {
    dataFolderInfo.value = await invokeCommand<DataFolderInfo>("get_data_folder_info_command");
    console.info("Data folder info loaded:", dataFolderInfo.value);
  } catch (error) {
    console.error("Failed to load data folder info:", error);
  } finally {
    loading.value = false;
  }
}

// Browse for folder
async function browseFolder() {
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({
      directory: true,
      multiple: false,
      title: t("settings.selectDataFolder"),
    });
    if (selected && typeof selected === "string") {
      selectedPath.value = selected;
      await validatePath();
    }
  } catch (error) {
    console.error("Failed to browse folder:", error);
  }
}

// Validate selected path
async function validatePath() {
  if (!selectedPath.value) return;

  validating.value = true;
  validation.value = null;
  try {
    validation.value = await invokeCommand<ValidationResult>("validate_data_folder_command", {
      path: selectedPath.value,
    });
    console.info("Validation result:", validation.value);
  } catch (error) {
    console.error("Failed to validate path:", error);
    validation.value = {
      valid: false,
      warnings: [],
      error: String(error),
    };
  } finally {
    validating.value = false;
  }
}

// Start migration
async function startMigration() {
  if (!selectedPath.value || !validation.value?.valid) return;

  migrating.value = true;
  showMigrationDialog.value = true;
  migrationStatus.value = {
    phase: "preparing",
    current_file: null,
    total_files: 0,
    processed_files: 0,
    error: null,
  };

  try {
    await invokeCommand("migrate_data_folder_command", {
      newPath: selectedPath.value,
    });
  } catch (error) {
    migrationStatus.value = {
      phase: "failed",
      current_file: null,
      total_files: 0,
      processed_files: 0,
      error: String(error),
    };
  }
}

// Restart app
async function restartApp() {
  try {
    await invokeCommand("restart_app");
  } catch (error) {
    console.error("Failed to restart app:", error);
  }
}

// Close migration dialog
function closeMigrationDialog() {
  showMigrationDialog.value = false;
}

// Database migration functions
async function loadDbMigrationStatus() {
  try {
    dbMigrationStatus.value = await invokeCommand<DbMigrationStatus>("get_migration_status");
    console.info("Database migration status:", dbMigrationStatus.value);
  } catch (error) {
    console.error("Failed to load database migration status:", error);
    dbMigrationStatus.value = null;
  }
}

async function startDbMigration() {
  if (!dbMigrationStatus.value?.can_migrate) return;

  dbMigrating.value = true;
  showDbMigrationDialog.value = true;
  dbMigrationReport.value = null;

  try {
    dbMigrationReport.value = await invokeCommand<DbMigrationReport>("run_migration");
    console.info("Database migration report:", dbMigrationReport.value);
    // Refresh status after migration
    await loadDbMigrationStatus();
  } catch (error) {
    console.error("Database migration failed:", error);
    dbMigrationReport.value = {
      papers_migrated: 0,
      authors_migrated: 0,
      keywords_migrated: 0,
      labels_migrated: 0,
      categories_migrated: 0,
      attachments_migrated: 0,
      paper_author_relations: 0,
      paper_label_relations: 0,
      paper_category_relations: 0,
      errors: [String(error)],
      duration_ms: 0,
    };
  } finally {
    dbMigrating.value = false;
  }
}

function closeDbMigrationDialog() {
  showDbMigrationDialog.value = false;
  if (dbMigrationReport.value && dbMigrationReport.value.errors.length === 0) {
    dbMigrationReport.value = null;
  }
}

// Lifecycle
onMounted(async () => {
  await loadDataFolderInfo();
  await loadDbMigrationStatus();

  // Listen for migration progress events
  try {
    const { listen } = await import("@tauri-apps/api/event");
    unlisten = await listen<MigrationStatus>("data-migration-progress", (event) => {
      migrationStatus.value = event.payload;
    });
  } catch (error) {
    console.error("Failed to listen for migration events:", error);
  }
});

onUnmounted(() => {
  if (unlisten) {
    unlisten();
  }
});
</script>

<template>
  <v-card>
    <v-card-title>
      <v-icon start>mdi-folder-cog</v-icon>
      {{ t("settings.dataFolder") }}
    </v-card-title>

    <v-card-text>
      <!-- Loading state -->
      <div v-if="loading" class="d-flex justify-center pa-4">
        <v-progress-circular indeterminate color="primary" />
      </div>

      <!-- Data folder info -->
      <div v-else-if="dataFolderInfo">
        <!-- Current location -->
        <div class="setting-section">
          <div class="setting-label">
            <v-icon class="mr-2">mdi-folder</v-icon>
            <span>{{ t("settings.currentDataFolder") }}</span>
          </div>
          <div class="mt-2">
            <v-chip :color="dataFolderInfo.is_custom ? 'primary' : 'default'" size="small" class="mb-2">
              {{ dataFolderInfo.is_custom ? t("settings.customLocation") : t("settings.defaultLocation") }}
            </v-chip>
            <div class="text-body-2 text-medium-emphasis">{{ dataFolderInfo.current_path }}</div>
          </div>
        </div>

        <v-divider class="my-4" />

        <!-- Data size -->
        <div class="setting-section">
          <div class="setting-label">
            <v-icon class="mr-2">mdi-database</v-icon>
            <span>{{ t("settings.totalDataSize") }}</span>
          </div>
          <div class="mt-2 text-h6">{{ formattedDataSize }}</div>
        </div>

        <v-divider class="my-4" />

        <!-- Change data folder -->
        <div class="setting-section">
          <div class="setting-label">
            <v-icon class="mr-2">mdi-folder-arrow-left-right</v-icon>
            <span>{{ t("settings.changeDataFolder") }}</span>
          </div>
          <div class="mt-2">
            <p class="text-body-2 text-medium-emphasis mb-3">
              {{ t("settings.dataFolderDescription") }}
            </p>

            <!-- Path input -->
            <div class="d-flex gap-2 align-center">
              <v-text-field
                v-model="selectedPath"
                :placeholder="t('settings.selectDataFolder')"
                readonly
                density="compact"
                variant="outlined"
                hide-details
                class="flex-grow-1"
              />
              <v-btn color="primary" variant="tonal" @click="browseFolder">
                <v-icon start>mdi-folder-open</v-icon>
                {{ t("settings.browseFolder") }}
              </v-btn>
            </div>

            <!-- Validation result -->
            <div v-if="validation" class="mt-3">
              <v-alert v-if="!validation.valid" type="error" density="compact" class="mb-2">
                {{ validation.error }}
              </v-alert>
              <v-alert
                v-if="validation.warnings.length > 0"
                type="warning"
                density="compact"
                class="mb-2"
              >
                <div v-for="warning in validation.warnings" :key="warning" class="text-body-2">
                  {{ warning }}
                </div>
              </v-alert>
            </div>

            <!-- Migration warning -->
            <v-alert v-if="validation?.valid" type="info" density="compact" class="mt-3">
              {{ t("settings.migrationWarning") }}
            </v-alert>

            <!-- Apply button -->
            <v-btn
              v-if="selectedPath && validation?.valid"
              color="primary"
              class="mt-3"
              :loading="migrating"
              @click="startMigration"
            >
              <v-icon start>mdi-folder-arrow-right</v-icon>
              {{ t("settings.applyMigration") }}
            </v-btn>
          </div>
        </div>

        <v-divider class="my-4" />

        <!-- Database Migration (SQLite to SurrealDB) -->
        <div class="setting-section">
          <div class="setting-label">
            <v-icon class="mr-2">mdi-database-sync</v-icon>
            <span>{{ t("settings.dbMigration") }}</span>
          </div>
          <div class="mt-2">
            <p class="text-body-2 text-medium-emphasis mb-3">
              {{ t("settings.dbMigrationDescription") }}
            </p>

            <!-- Migration status -->
            <div v-if="dbMigrationStatus" class="mb-3">
              <v-chip-group>
                <v-chip size="small" color="info">
                  SQLite: {{ dbMigrationStatus.sqlite_papers_count }} papers
                </v-chip>
                <v-chip size="small" color="success">
                  SurrealDB: {{ dbMigrationStatus.surreal_papers_count }} papers
                </v-chip>
              </v-chip-group>

              <v-alert
                v-if="dbMigrationStatus.can_migrate && dbMigrationStatus.sqlite_papers_count > dbMigrationStatus.surreal_papers_count"
                type="info"
                density="compact"
                class="mt-2"
              >
                {{ dbMigrationStatus.message }}
              </v-alert>
              <v-alert
                v-else-if="dbMigrationStatus.sqlite_papers_count === 0"
                type="info"
                density="compact"
                class="mt-2"
              >
                {{ t("settings.noDataToMigrate") }}
              </v-alert>
              <v-alert
                v-else
                type="success"
                density="compact"
                class="mt-2"
              >
                {{ t("settings.migrationCompleted") }}
              </v-alert>
            </div>

            <!-- Migrate button -->
            <v-btn
              v-if="dbMigrationStatus?.can_migrate && dbMigrationStatus.sqlite_papers_count > dbMigrationStatus.surreal_papers_count"
              color="primary"
              :loading="dbMigrating"
              @click="startDbMigration"
            >
              <v-icon start>mdi-database-arrow-right</v-icon>
              {{ t("settings.startDbMigration") }}
            </v-btn>
          </div>
        </div>

        <v-divider class="my-4" />

        <!-- Revert to default -->
        <div v-if="dataFolderInfo.is_custom" class="setting-section">
          <div class="setting-label">
            <v-icon class="mr-2">mdi-undo</v-icon>
            <span>{{ t("settings.revertToDefault") }}</span>
          </div>
          <div class="mt-2">
            <p class="text-body-2 text-medium-emphasis mb-2">
              {{ t("settings.defaultLocation") }}: {{ dataFolderInfo.default_path }}
            </p>
          </div>
        </div>
      </div>
    </v-card-text>

    <!-- Migration Progress Dialog -->
    <v-dialog v-model="showMigrationDialog" max-width="500" persistent>
      <v-card>
        <v-card-title>{{ t("settings.migrationProgress") }}</v-card-title>
        <v-card-text>
          <div class="mb-4">
            <div class="text-body-2 mb-2">{{ migrationPhaseText }}</div>
            <v-progress-linear
              :model-value="migrationProgress"
              color="primary"
              height="10"
              rounded
            />
            <div class="text-caption text-medium-emphasis mt-1">
              {{ migrationStatus?.processed_files || 0 }} / {{ migrationStatus?.total_files || 0 }}
            </div>
          </div>

          <div v-if="migrationStatus?.current_file" class="text-caption text-medium-emphasis">
            {{ migrationStatus.current_file }}
          </div>

          <v-alert v-if="migrationStatus?.error" type="error" density="compact" class="mt-3">
            {{ migrationStatus.error }}
          </v-alert>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <!-- Restart button when completed -->
          <v-btn
            v-if="migrationStatus?.phase === 'completed'"
            color="primary"
            @click="restartApp"
          >
            <v-icon start>mdi-restart</v-icon>
            {{ t("settings.restartNow") }}
          </v-btn>
          <!-- Close button when failed -->
          <v-btn
            v-if="migrationStatus?.phase === 'failed'"
            @click="closeMigrationDialog"
          >
            {{ t("dialog.close") }}
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Database Migration Dialog -->
    <v-dialog v-model="showDbMigrationDialog" max-width="600" persistent>
      <v-card>
        <v-card-title class="d-flex align-center justify-space-between">
          <span>{{ t("settings.dbMigrationProgress") }}</span>
          <v-btn v-if="!dbMigrating" icon="mdi-close" variant="text" @click="closeDbMigrationDialog" />
        </v-card-title>

        <v-card-text>
          <!-- Migration in progress -->
          <div v-if="dbMigrating" class="text-center py-8">
            <v-progress-circular indeterminate size="64" class="mb-4" />
            <p class="text-h6">{{ t("settings.migratingData") }}</p>
            <p class="text-body-2 text-medium-emphasis">{{ t("settings.pleaseWait") }}</p>
          </div>

          <!-- Migration report -->
          <div v-else-if="dbMigrationReport">
            <v-alert
              :type="dbMigrationReport.errors.length > 0 ? 'warning' : 'success'"
              class="mb-4"
            >
              {{ dbMigrationReport.errors.length > 0 ? t("settings.migrationCompletedWithErrors") : t("settings.migrationCompletedSuccess") }}
            </v-alert>

            <v-list density="compact">
              <v-list-item v-if="dbMigrationReport.papers_migrated > 0">
                <v-list-item-title>{{ t("settings.papers") }}</v-list-item-title>
                <template v-slot:append>
                  <v-chip size="small">{{ dbMigrationReport.papers_migrated }}</v-chip>
                </template>
              </v-list-item>
              <v-list-item v-if="dbMigrationReport.authors_migrated > 0">
                <v-list-item-title>{{ t("settings.authors") }}</v-list-item-title>
                <template v-slot:append>
                  <v-chip size="small">{{ dbMigrationReport.authors_migrated }}</v-chip>
                </template>
              </v-list-item>
              <v-list-item v-if="dbMigrationReport.keywords_migrated > 0">
                <v-list-item-title>{{ t("settings.keywords") }}</v-list-item-title>
                <template v-slot:append>
                  <v-chip size="small">{{ dbMigrationReport.keywords_migrated }}</v-chip>
                </template>
              </v-list-item>
              <v-list-item v-if="dbMigrationReport.labels_migrated > 0">
                <v-list-item-title>{{ t("settings.labels") }}</v-list-item-title>
                <template v-slot:append>
                  <v-chip size="small">{{ dbMigrationReport.labels_migrated }}</v-chip>
                </template>
              </v-list-item>
              <v-list-item v-if="dbMigrationReport.categories_migrated > 0">
                <v-list-item-title>{{ t("settings.categories") }}</v-list-item-title>
                <template v-slot:append>
                  <v-chip size="small">{{ dbMigrationReport.categories_migrated }}</v-chip>
                </template>
              </v-list-item>
              <v-list-item v-if="dbMigrationReport.attachments_migrated > 0">
                <v-list-item-title>{{ t("settings.attachments") }}</v-list-item-title>
                <template v-slot:append>
                  <v-chip size="small">{{ dbMigrationReport.attachments_migrated }}</v-chip>
                </template>
              </v-list-item>
              <v-list-item v-if="dbMigrationReport.duration_ms > 0">
                <v-list-item-title>{{ t("settings.duration") }}</v-list-item-title>
                <template v-slot:append>
                  <v-chip size="small">{{ (dbMigrationReport.duration_ms / 1000).toFixed(2) }}s</v-chip>
                </template>
              </v-list-item>
            </v-list>

            <v-alert
              v-if="dbMigrationReport.errors && dbMigrationReport.errors.length > 0"
              type="warning"
              class="mt-4"
            >
              <p class="font-weight-bold mb-2">{{ t("settings.migrationErrors") }}</p>
              <v-list density="compact" bg-color="transparent">
                <v-list-item v-for="(error, i) in dbMigrationReport.errors" :key="i">
                  <v-list-item-title>{{ error }}</v-list-item-title>
                </v-list-item>
              </v-list>
            </v-alert>
          </div>
        </v-card-text>

        <v-card-actions v-if="!dbMigrating && dbMigrationReport">
          <v-spacer />
          <v-btn color="primary" variant="flat" @click="closeDbMigrationDialog">
            {{ t("dialog.close") }}
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

  </v-card>
</template>

<style scoped>
.setting-section {
  padding: 8px 0;
}

.setting-label {
  display: flex;
  align-items: center;
  font-weight: 500;
  margin-bottom: 8px;
}

.gap-2 {
  gap: 8px;
}
</style>
