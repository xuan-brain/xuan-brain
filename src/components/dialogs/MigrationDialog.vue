<template>
  <v-dialog v-model="isOpen" max-width="600">
    <v-card>
      <v-card-title class="d-flex align-center justify-space-between">
        <span>Data Migration</span>
        <v-btn icon="mdi-close" variant="text" @click="close" />
      </v-card-title>

      <v-card-text>
        <!-- Migration Status -->
        <div v-if="!isMigrating && !migrationReport">
          <v-alert
            v-if="status"
            :type="status.canMigrate && status.sqlitePapersCount > status.surrealPapersCount ? 'info' : 'success'"
            class="mb-4"
          >
            {{ status.message }}
          </v-alert>

          <v-list v-if="status" density="compact">
            <v-list-item>
              <template v-slot:prepend>
                <v-icon icon="mdi-database" />
              </template>
              <v-list-item-title>SQLite Papers</v-list-item-title>
              <v-list-item-subtitle>{{ status.sqlitePapersCount }} records</v-list-item-subtitle>
            </v-list-item>
            <v-list-item>
              <template v-slot:prepend>
                <v-icon icon="mdi-database-outline" />
              </template>
              <v-list-item-title>SurrealDB Papers</v-list-item-title>
              <v-list-item-subtitle>{{ status.surrealPapersCount }} records</v-list-item-subtitle>
            </v-list-item>
          </v-list>

          <v-divider class="my-4" />

          <p class="text-body-2 text-medium-emphasis">
            This will migrate all data from SQLite to SurrealDB, including:
          </p>
          <v-chip-group class="mt-2">
            <v-chip size="small">Papers</v-chip>
            <v-chip size="small">Authors</v-chip>
            <v-chip size="small">Keywords</v-chip>
            <v-chip size="small">Labels</v-chip>
            <v-chip size="small">Categories</v-chip>
            <v-chip size="small">Attachments</v-chip>
          </v-chip-group>
        </div>

        <!-- Migration Progress -->
        <div v-if="isMigrating" class="text-center py-8">
          <v-progress-circular indeterminate size="64" class="mb-4" />
          <p class="text-h6">Migrating data...</p>
          <p class="text-body-2 text-medium-emphasis">Please wait while we transfer your data</p>
        </div>

        <!-- Migration Report -->
        <div v-if="migrationReport && !isMigrating">
          <v-alert type="success" class="mb-4">
            Migration completed successfully!
          </v-alert>

          <v-list density="compact">
            <v-list-item v-if="migrationReport.papers_migrated">
              <v-list-item-title>Papers</v-list-item-title>
              <template v-slot:append>
                <v-chip size="small">{{ migrationReport.papers_migrated }}</v-chip>
              </template>
            </v-list-item>
            <v-list-item v-if="migrationReport.authors_migrated">
              <v-list-item-title>Authors</v-list-item-title>
              <template v-slot:append>
                <v-chip size="small">{{ migrationReport.authors_migrated }}</v-chip>
              </template>
            </v-list-item>
            <v-list-item v-if="migrationReport.keywords_migrated">
              <v-list-item-title>Keywords</v-list-item-title>
              <template v-slot:append>
                <v-chip size="small">{{ migrationReport.keywords_migrated }}</v-chip>
              </template>
            </v-list-item>
            <v-list-item v-if="migrationReport.labels_migrated">
              <v-list-item-title>Labels</v-list-item-title>
              <template v-slot:append>
                <v-chip size="small">{{ migrationReport.labels_migrated }}</v-chip>
              </template>
            </v-list-item>
            <v-list-item v-if="migrationReport.categories_migrated">
              <v-list-item-title>Categories</v-list-item-title>
              <template v-slot:append>
                <v-chip size="small">{{ migrationReport.categories_migrated }}</v-chip>
              </template>
            </v-list-item>
            <v-list-item v-if="migrationReport.attachments_migrated">
              <v-list-item-title>Attachments</v-list-item-title>
              <template v-slot:append>
                <v-chip size="small">{{ migrationReport.attachments_migrated }}</v-chip>
              </template>
            </v-list-item>
            <v-list-item v-if="migrationReport.paper_author_relations">
              <v-list-item-title>Paper-Author Relations</v-list-item-title>
              <template v-slot:append>
                <v-chip size="small">{{ migrationReport.paper_author_relations }}</v-chip>
              </template>
            </v-list-item>
            <v-list-item v-if="migrationReport.paper_label_relations">
              <v-list-item-title>Paper-Label Relations</v-list-item-title>
              <template v-slot:append>
                <v-chip size="small">{{ migrationReport.paper_label_relations }}</v-chip>
              </template>
            </v-list-item>
            <v-list-item v-if="migrationReport.paper_category_relations">
              <v-list-item-title>Paper-Category Relations</v-list-item-title>
              <template v-slot:append>
                <v-chip size="small">{{ migrationReport.paper_category_relations }}</v-chip>
              </template>
            </v-list-item>
          </v-list>

          <v-list-item v-if="migrationReport.duration_ms">
            <v-list-item-title>Duration</v-list-item-title>
            <template v-slot:append>
              <v-chip size="small">{{ (migrationReport.duration_ms / 1000).toFixed(2) }}s</v-chip>
            </template>
          </v-list-item>

          <v-alert
            v-if="migrationReport.errors && migrationReport.errors.length > 0"
            type="warning"
            class="mt-4"
          >
            <p class="font-weight-bold mb-2">Some errors occurred:</p>
            <v-list density="compact" bg-color="transparent">
              <v-list-item v-for="(error, i) in migrationReport.errors" :key="i">
                <v-list-item-title>{{ error }}</v-list-item-title>
              </v-list-item>
            </v-list>
          </v-alert>
        </div>

        <!-- Error State -->
        <v-alert v-if="error" type="error" class="mt-4">
          {{ error }}
        </v-alert>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn variant="text" @click="close">
          {{ migrationReport ? 'Close' : 'Cancel' }}
        </v-btn>
        <v-btn
          v-if="!isMigrating && !migrationReport && status?.canMigrate"
          color="primary"
          variant="flat"
          @click="startMigration"
        >
          Start Migration
        </v-btn>
        <v-btn
          v-if="migrationReport"
          color="primary"
          variant="flat"
          @click="resetAndClose"
        >
          Done
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

interface MigrationStatus {
  can_migrate: boolean
  sqlite_papers_count: number
  surreal_papers_count: number
  message: string
}

interface MigrationReport {
  papers_migrated: number
  authors_migrated: number
  keywords_migrated: number
  labels_migrated: number
  categories_migrated: number
  attachments_migrated: number
  paper_author_relations: number
  paper_label_relations: number
  paper_category_relations: number
  errors: string[]
  duration_ms: number
}

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  (e: 'update:open', value: boolean): void
}>()

const isOpen = ref(props.open)
const status = ref<MigrationStatus | null>(null)
const migrationReport = ref<MigrationReport | null>(null)
const isMigrating = ref(false)
const error = ref<string | null>(null)

// Helper function for lazy loading Tauri API
async function invokeCommand<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core')
  return invoke<T>(cmd, args)
}

watch(
  () => props.open,
  async (newVal) => {
    isOpen.value = newVal
    if (newVal) {
      await loadStatus()
    }
  }
)

watch(isOpen, (newVal) => {
  emit('update:open', newVal)
})

async function loadStatus() {
  error.value = null
  try {
    status.value = await invokeCommand<MigrationStatus>('get_migration_status')
    console.info('Migration status:', status.value)
  } catch (e) {
    error.value = String(e)
    console.error('Failed to get migration status:', e)
  }
}

async function startMigration() {
  isMigrating.value = true
  error.value = null
  migrationReport.value = null

  try {
    migrationReport.value = await invokeCommand<MigrationReport>('run_migration')
    console.info('Migration report:', migrationReport.value)
  } catch (e) {
    error.value = String(e)
    console.error('Migration failed:', e)
  } finally {
    isMigrating.value = false
  }
}

function close() {
  isOpen.value = false
}

function resetAndClose() {
  migrationReport.value = null
  status.value = null
  error.value = null
  close()
}
</script>
