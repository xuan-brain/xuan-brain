<script setup lang="ts">
  import ImportByArxivDialog from '@/components/dialogs/ImportByArxivDialog.vue';
  import ImportByDoiDialog from '@/components/dialogs/ImportByDoiDialog.vue';
  import ImportByPubmedDialog from '@/components/dialogs/ImportByPubmedDialog.vue';
  import ImportPdfDialog from '@/components/dialogs/ImportPdfDialog.vue';
  import ImportZoteroDialog from '@/components/dialogs/ImportZoteroDialog.vue';
  import { useI18n } from '@/lib/i18n';
  import { ref } from 'vue';

  const { t } = useI18n();

  // Dialog states
  const showDoiDialog = ref(false);
  const showArxivDialog = ref(false);
  const showPubmedDialog = ref(false);
  const showPdfDialog = ref(false);
  const showZoteroDialog = ref(false);

  // Import options
  const importOptions = [
    {
      id: 'doi',
      icon: 'mdi-link-variant',
      title: 'toolbar.importByDoi',
      description: 'toolbar.doiDescription',
      color: 'primary',
    },
    {
      id: 'arxiv',
      icon: 'mdi-school',
      title: 'toolbar.importByArxiv',
      description: 'toolbar.arxivDescription',
      color: 'secondary',
    },
    {
      id: 'pubmed',
      icon: 'mdi-dna',
      title: 'toolbar.importByPubmed',
      description: 'toolbar.pubmedDescription',
      color: 'success',
    },
    {
      id: 'pdf',
      icon: 'mdi-file-pdf-box',
      title: 'toolbar.importPdf',
      description: 'toolbar.pdfDescription',
      color: 'error',
    },
    {
      id: 'zotero',
      icon: 'mdi-database-import',
      title: 'toolbar.importFromZotero',
      description: 'toolbar.zoteroDescription',
      color: 'indigo',
    },
  ];

  function handleImportClick(id: string) {
    switch (id) {
      case 'doi':
        showDoiDialog.value = true;
        break;
      case 'arxiv':
        showArxivDialog.value = true;
        break;
      case 'pubmed':
        showPubmedDialog.value = true;
        break;
      case 'pdf':
        showPdfDialog.value = true;
        break;
      case 'zotero':
        showZoteroDialog.value = true;
        break;
    }
  }

  function handleImportSuccess() {
    // Refresh could be handled here if needed
    console.info('Import completed successfully');
  }
</script>

<template>
  <div class="import-page">
    <div class="import-header">
      <h1 class="text-h4 mb-2">{{ t('main.importDocuments') }}</h1>
      <p class="text-body-1 text-medium-emphasis">
        Choose an import method to add papers to your library
      </p>
    </div>

    <div class="import-options">
      <v-card
        v-for="option in importOptions"
        :key="option.id"
        class="import-card"
        :prepend-icon="option.icon"
        hover
        @click="handleImportClick(option.id)"
      >
        <template #title>
          <span class="font-weight-medium">{{ t(option.title) }}</span>
        </template>
        <v-card-text>
          {{ t(option.description) }}
        </v-card-text>
      </v-card>
    </div>

    <!-- Import Dialogs -->
    <ImportByDoiDialog v-model="showDoiDialog" @success="handleImportSuccess" />
    <ImportByArxivDialog v-model="showArxivDialog" @success="handleImportSuccess" />
    <ImportByPubmedDialog v-model="showPubmedDialog" @success="handleImportSuccess" />
    <ImportPdfDialog v-model="showPdfDialog" @success="handleImportSuccess" />
    <ImportZoteroDialog v-model="showZoteroDialog" @success="handleImportSuccess" />
  </div>
</template>

<style scoped>
  .import-page {
    height: 100%;
    padding: 32px;
    overflow-y: auto;
  }

  .import-header {
    max-width: 800px;
    margin: 0 auto 32px;
    text-align: center;
  }

  .import-options {
    max-width: 800px;
    margin: 0 auto;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(340px, 1fr));
    gap: 16px;
  }

  .import-card {
    cursor: pointer;
    transition:
      transform 0.2s ease,
      box-shadow 0.2s ease;
  }

  .import-card:hover {
    transform: translateY(-2px);
  }
</style>
