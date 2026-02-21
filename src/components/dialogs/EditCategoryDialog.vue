<script setup lang="ts">
  import { useI18n } from '@/lib/i18n';
  import { invokeCommand } from '@/lib/tauri';
  import { ref, watch } from 'vue';

  const { t } = useI18n();

  interface Props {
    modelValue: boolean;
    categoryId?: string;
    categoryName?: string;
  }

  const props = defineProps<Props>();

  const emit = defineEmits<{
    'update:modelValue': [value: boolean];
    categoryUpdated: [];
  }>();

  // State
  const name = ref('');
  const error = ref('');
  const loading = ref(false);

  // Reset form when dialog opens
  watch(
    () => props.modelValue,
    (isOpen) => {
      if (isOpen && props.categoryName) {
        name.value = props.categoryName;
        error.value = '';
      }
    }
  );

  // Close dialog
  function handleClose() {
    name.value = '';
    error.value = '';
    emit('update:modelValue', false);
  }

  // Submit form
  async function handleSubmit() {
    if (!props.categoryId) {
      error.value = 'Category ID is required';
      return;
    }

    if (!name.value.trim()) {
      error.value = t('dialog.categoryNameRequired');
      return;
    }

    if (name.value.length > 50) {
      error.value = t('dialog.categoryNameMaxLength');
      return;
    }

    loading.value = true;
    try {
      await invokeCommand('update_category', {
        id: props.categoryId,
        name: name.value.trim(),
      });
      console.info('Category updated successfully:', name.value.trim());
      name.value = '';
      error.value = '';
      emit('categoryUpdated');
      emit('update:modelValue', false);
    } catch (err) {
      error.value = err as string;
    } finally {
      loading.value = false;
    }
  }

  // Handle Enter key
  function handleKeyPress(event: KeyboardEvent) {
    if (event.key === 'Enter' && !loading.value && name.value.trim() && name.value.length <= 50) {
      handleSubmit();
    }
  }
</script>

<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="emit('update:modelValue', $event)"
    max-width="480"
  >
    <v-card>
      <v-card-title>
        <v-icon start>mdi-folder-edit</v-icon>
        {{ t('dialog.editCategory') }}
      </v-card-title>

      <v-card-text>
        <v-alert v-if="error" type="error" :text="error" class="mb-4" />

        <v-text-field
          v-model="name"
          autofocus
          :label="t('dialog.enterCategoryName')"
          variant="outlined"
          :error-messages="error ? [error] : []"
          :disabled="loading"
          @keyup="handleKeyPress"
        />

        <v-alert type="info" density="compact" class="mt-4">
          {{ t('dialog.categoryNameRules') }}
        </v-alert>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn @click="handleClose" :disabled="loading">
          {{ t('dialog.cancel') }}
        </v-btn>
        <v-btn
          color="primary"
          @click="handleSubmit"
          :loading="loading"
          :disabled="!name.trim() || name.length > 50"
        >
          {{ t('dialog.save') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>
