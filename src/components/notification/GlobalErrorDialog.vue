<!-- src/components/notification/GlobalErrorDialog.vue -->
<script setup lang="ts">
import { useNotificationStore } from '@/stores/useNotificationStore';
import { ref, computed } from 'vue';

const notificationStore = useNotificationStore();

const showDialog = computed(() => notificationStore.showDialog);
const dialogContent = computed(() => notificationStore.dialogContent);

const showDetails = ref(false);

function handleClose() {
  showDetails.value = false;
  notificationStore.closeDialog();
}

async function copyToClipboard() {
  if (dialogContent.value?.details) {
    try {
      await navigator.clipboard.writeText(dialogContent.value.details);
      console.info('Error details copied to clipboard');
    } catch (error) {
      console.error('Failed to copy to clipboard:', error);
    }
  }
}

function getIcon(type: string) {
  switch (type) {
    case 'error':
      return 'mdi-alert-circle';
    case 'warning':
      return 'mdi-alert';
    default:
      return 'mdi-information';
  }
}

function getColor(type: string) {
  switch (type) {
    case 'error':
      return 'error';
    case 'warning':
      return 'warning';
    default:
      return 'info';
  }
}
</script>

<template>
  <v-dialog :model-value="showDialog" @update:model-value="handleClose" max-width="500">
    <v-card v-if="dialogContent">
      <v-card-title class="d-flex align-center">
        <v-icon :icon="getIcon(dialogContent.type)" :color="getColor(dialogContent.type)" class="mr-2" />
        <span>{{ dialogContent.title }}</span>
      </v-card-title>

      <v-card-text>
        <div class="error-message">{{ dialogContent.message }}</div>

        <v-expand-transition>
          <div v-if="dialogContent.details" class="error-details-container">
            <v-btn
              variant="text"
              size="small"
              class="toggle-details-btn"
              @click="showDetails = !showDetails"
            >
              {{ showDetails ? 'Hide' : 'Show' }} details
              <v-icon :icon="showDetails ? 'mdi-chevron-up' : 'mdi-chevron-down'" end />
            </v-btn>

            <v-expand-transition>
              <div v-show="showDetails" class="error-details">
                <pre class="error-stack">{{ dialogContent.details }}</pre>
              </div>
            </v-expand-transition>
          </div>
        </v-expand-transition>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn
          v-if="dialogContent.details"
          variant="text"
          @click="copyToClipboard"
        >
          Copy
        </v-btn>
        <v-btn
          :color="getColor(dialogContent.type)"
          variant="elevated"
          @click="handleClose"
        >
          OK
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
.error-message {
  font-size: 15px;
  line-height: 1.5;
  padding: 8px 0;
}

.error-details-container {
  margin-top: 16px;
}

.toggle-details-btn {
  padding: 0;
}

.error-details {
  margin-top: 8px;
  padding: 12px;
  background-color: rgba(0, 0, 0, 0.05);
  border-radius: 4px;
  max-height: 200px;
  overflow-y: auto;
}

.error-stack {
  margin: 0;
  font-size: 12px;
  font-family: 'Consolas', 'Monaco', monospace;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
