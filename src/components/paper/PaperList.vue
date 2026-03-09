<script setup lang="ts">
  import { useI18n } from '@/lib/i18n';
  import { invokeCommand } from '@/lib/tauri';
  import { listen } from '@tauri-apps/api/event';
  import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
  import { open } from '@tauri-apps/plugin-dialog';
  import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue';
  import type { VxeTablePropTypes } from 'vxe-table';
  // 显式导入 vxe-pager 组件
  import VxePager from 'vxe-pc-ui/lib/pager';
  import PaperToolbar from './PaperToolbar.vue';

  const { t } = useI18n();

  interface Label {
    id: string;
    name: string;
    color: string;
  }

  interface Attachment {
    id: string;
    paper_id: string;
    file_name: string | null;
    file_type: string | null;
    created_at: string | null;
  }

  interface PaperDto {
    id: string;
    title: string;
    authors: string[];
    publication_year?: number;
    journal_name?: string;
    conference_name?: string;
    labels: Label[];
    attachment_count?: number;
    attachments?: Attachment[];
    publisher?: string | null;
    issn?: string | null;
    language?: string | null;
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
  }

  interface Props {
    categoryId?: string | null;
    currentView?: 'library' | 'favorites' | 'trash';
  }

  const props = withDefaults(defineProps<Props>(), {
    categoryId: null,
    currentView: 'library',
  });

  const emit = defineEmits<{
    paperSelect: [paperId: string];
    paperUpdated: [paperId: string, detail: PaperDetail];
  }>();

  // State
  const loading = ref(false);
  const papers = ref<PaperDto[]>([]);

  // 分页配置
  const pageConfig = reactive({
    currentPage: 1,
    pageSize: 50,
    total: 0,
  });

  const pageSizes = [20, 50, 100, 200]; // 可选的每页条数

  // Table ref

  // Expand configuration
  const expandRowIds = ref<string[]>([]);

  const expandConfig = computed<VxeTablePropTypes.ExpandConfig>(() => ({
    showIcon: true,
    trigger: 'row',
    expandRowKeys: expandRowIds.value,
    accordion: true, // 手风琴模式：展开一行时其他行自动折叠
    visibleMethod: ({ row }) => {
      // 只对有附件的行显示展开图标
      const paper = row as PaperDto;
      return (paper.attachment_count ?? 0) > 0;
    },
    toggleMethod({ row }) {
      const paper = row as PaperDto;
      if ((paper.attachment_count ?? 0) === 0) {
        return false; // 没有附件，禁止展开
      }
      return true;
    },
  }));

  // Sort configuration
  const sortConfig = reactive({
    defaultSort: {
      field: '' as string,
      order: '' as 'asc' | 'desc',
    },
  });

  // Context menu configuration
  const contextMenuConfig = computed(() => {
    // Build menu items based on current view (must be 2D array)
    if (props.currentView === 'trash') {
      // Trash view menu
      return {
        body: {
          options: [
            [
              {
                code: 'restore',
                name: t('dialog.restore'),
                prefixConfig: { icon: 'vxe-icon-undo' },
              },
              {
                code: 'permanently_delete',
                name: t('dialog.permanentlyDelete'),
                prefixConfig: { icon: 'vxe-icon-delete' },
              },
            ],
          ],
        },
      };
    } else {
      // Normal view menu
      return {
        body: {
          options: [
            [
              {
                code: 'add_attachment',
                name: '添加附件',
                prefixConfig: { icon: 'vxe-icon-file' },
              },
              {
                code: 'open_folder',
                name: '打开附件文件夹',
                prefixConfig: { icon: 'vxe-icon-folder-open' },
              },
            ],
            [
              {
                code: 'delete',
                name: t('dialog.delete'),
                prefixConfig: { icon: 'vxe-icon-folder-open' },
              },
            ],
          ],
        },
      };
    }
  });

  // Handle context menu click
  async function handleContextMenuClick({ menu, row }: any) {
    const paper = row as PaperDto;
    console.info('Context menu clicked:', menu.code, paper);

    switch (menu.code) {
      case 'add_attachment':
        await handleAddAttachment(paper);
        break;
      case 'open_folder':
        await handleOpenFolder(paper);
        break;
      case 'delete':
        await handleDeletePaper(paper);
        break;
      case 'restore':
        await handleRestorePaper(paper);
        break;
      case 'permanently_delete':
        await handlePermanentlyDeletePaper(paper);
        break;
    }
  }

  // Handle context menu shown (for debugging)
  function handleContextMenuVisible({ type, row, column }: any) {
    console.info('Context menu shown:', { type, row, column });
  }

  // Add attachment to paper
  async function handleAddAttachment(paper: PaperDto) {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
      });
      if (selected) {
        const filePath = Array.isArray(selected) ? selected[0] : selected;
        if (filePath) {
          await invokeCommand('add_attachment', {
            paperId: paper.id,
            filePath: filePath,
          });
          // Reload papers to show updated attachment count
          await loadPapers();
          console.info('Attachment added successfully');
        }
      }
    } catch (error) {
      console.error('Failed to add attachment:', error);
    }
  }

  // Open paper folder
  async function handleOpenFolder(paper: PaperDto) {
    try {
      await invokeCommand('open_paper_folder', { paperId: paper.id });
    } catch (error) {
      console.error('Failed to open folder:', error);
    }
  }

  // Delete paper (move to trash)
  async function handleDeletePaper(paper: PaperDto) {
    try {
      await invokeCommand('delete_paper', { id: paper.id });
      await loadPapers();
    } catch (error) {
      console.error('Failed to delete paper:', error);
    }
  }

  // Restore paper from trash
  async function handleRestorePaper(paper: PaperDto) {
    try {
      await invokeCommand('restore_paper', { id: paper.id });
      await loadPapers();
    } catch (error) {
      console.error('Failed to restore paper:', error);
    }
  }

  // Permanently delete paper
  async function handlePermanentlyDeletePaper(paper: PaperDto) {
    try {
      await invokeCommand('permanently_delete_paper', { id: paper.id });
      await loadPapers();
    } catch (error) {
      console.error('Failed to permanently delete paper:', error);
    }
  }

  // Table columns definition

  // Load papers from backend based on current view
  async function loadPapers() {
    const perfStart = performance.now();
    console.info('[PERF] Starting loadPapers (pagination mode)');
    loading.value = true;

    // Clear existing papers
    papers.value = [];

    try {
      if (props.currentView === 'trash') {
        // Load deleted papers (no pagination for trash)
        const data = await invokeCommand<PaperDto[]>('get_deleted_papers');
        papers.value = data;
        pageConfig.total = data.length;
        loading.value = false;
      } else if (props.categoryId) {
        // Load papers for specific category (no pagination for category)
        const data = await invokeCommand<PaperDto[]>('get_papers_by_category', {
          categoryId: props.categoryId,
        });
        papers.value = data;
        pageConfig.total = data.length;
        loading.value = false;
      } else {
        // Load all papers with pagination
        const offset = (pageConfig.currentPage - 1) * pageConfig.pageSize;

        const data = await invokeCommand<{
          papers: PaperDto[];
          total: number;
          offset: number;
          limit: number;
          has_more: boolean;
        }>('get_papers_paginated', {
          offset: offset,
          limit: pageConfig.pageSize,
        });

        papers.value = data.papers;
        pageConfig.total = data.total;
        loading.value = false;

        console.info(
          `[PERF] Pagination loaded: ${data.papers.length} papers, total=${data.total}, page=${pageConfig.currentPage}`
        );
      }

      const totalEnd = performance.now();
      console.info(`[PERF] loadPapers completed: ${(totalEnd - perfStart).toFixed(2)}ms`);
    } catch (error) {
      console.error('Failed to load papers:', error);
      loading.value = false;
    }
  }

  // 分页事件处理：页码或每页条数变化
  function handlePageChange(params: { currentPage: number; pageSize: number }) {
    // v-model 会自动更新值，这里只需要触发数据加载
    loadPapers();
  }

  // // Handle row click - emit paper selection
  // function handleCellClick({ row }: { row: PaperDto }) {
  //   emit("paperSelect", row.id);
  // }

  // // Handle row double click - navigate to paper detail
  // function handleRowDblclick({ row }: { row: PaperDto }) {
  //   router.push(`/papers/${row.id}`);
  // }

  // Handle sort change
  function handleSortChange(params: any) {
    const { sortList } = params;
    if (sortList && sortList.length > 0) {
      const { field, order } = sortList[0];
      sortConfig.defaultSort.field = field;
      sortConfig.defaultSort.order = order as 'asc' | 'desc';

      // Sort papers locally
      papers.value.sort((a, b) => {
        const aVal = (a as any)[field];
        const bVal = (b as any)[field];

        if (aVal === undefined || aVal === null) return 1;
        if (bVal === undefined || bVal === null) return -1;
        if (aVal < bVal) return order === 'asc' ? -1 : 1;
        if (aVal > bVal) return order === 'asc' ? 1 : -1;
        return 0;
      });
    }
  }

  // Handle cell click event
  function cellClickEvent({ row, column }: { row: any; column: any }) {
    console.log(`Single clicked row: ${row.id}, column: ${column.field}`);
    emit('paperSelect', row.id);
  }

  // Handle cell double click event
  async function cellDblclickEvent({ row, column }: { row: any; column: any }) {
    console.log(`Double clicked row: ${row.id}, column: ${column.field}`);

    const paper = row as PaperDto;

    // Check if paper has attachments
    if ((paper.attachment_count ?? 0) === 0) {
      console.info(`Paper ${paper.id} has no attachments`);
      return;
    }

    try {
      // Create PDF viewer window with unique label
      const windowLabel = `pdf-viewer-${paper.id}`;

      // Check if PDF viewer window already exists
      const existingPdfWindow = await WebviewWindow.getByLabel(windowLabel);

      if (existingPdfWindow) {
        // Focus existing window instead of creating a new one
        await existingPdfWindow.setFocus();
        await existingPdfWindow.unminimize();
        console.info(`Focused existing PDF viewer window for paper ${paper.id}`);
        return;
      }

      // Create new PDF viewer window
      const webview = new WebviewWindow(windowLabel, {
        title: paper.title,
        url: '/pdf-viewer.html',
        width: 1200,
        height: 800,
        center: true,
        resizable: true,
        decorations: true,
        transparent: false,
        alwaysOnTop: false,
      });

      // Wait for window to be created
      webview.once('tauri://created', async () => {
        console.info(`PDF viewer window created for paper ${paper.id}`);
        await webview.show();
        await webview.setFocus();
      });

      webview.once('tauri://error', (error) => {
        console.error('Failed to create PDF viewer window:', error);
      });
    } catch (error) {
      console.error('Failed to open PDF viewer:', error);
    }
  }

  // Store for loaded attachments (lazy loading)
  const loadedAttachments = ref<Map<string, Attachment[]>>(new Map());
  const loadingAttachments = ref<Set<string>>(new Set());

  // Handle expand row toggle - load attachments on demand
  async function handleToggleExpandChange({ row }: { row: any }) {
    const paper = row as PaperDto;
    const index = expandRowIds.value.indexOf(paper.id);

    console.info(
      'Toggle expand clicked for row:',
      paper.id,
      'current expanded rows:',
      expandRowIds.value
    );

    if (index > -1) {
      // Collapse
      expandRowIds.value.splice(index, 1);
      console.info('Collapsed row:', paper.id);
    } else {
      // Expand - load attachments on demand if not already loaded
      if ((paper.attachment_count ?? 0) > 0 && !loadedAttachments.value.has(paper.id)) {
        loadingAttachments.value.add(paper.id);
        try {
          const attachments = await invokeCommand<Attachment[]>('get_attachments', {
            paperId: paper.id,
          });
          loadedAttachments.value.set(paper.id, attachments);
          paper.attachments = attachments;
          console.info('Loaded attachments for paper:', paper.id, 'count:', attachments.length);
        } catch (error) {
          console.error('Failed to load attachments:', error);
          paper.attachments = [];
        } finally {
          loadingAttachments.value.delete(paper.id);
        }
      } else {
        // Already loaded, just expand
        expandRowIds.value.push(paper.id);
        console.info('Expanded row:', paper.id, 'attachment_count:', paper.attachment_count);
      }
    }
  }

  // Handle expand row change event (fallback)

  // Check if row is expanded

  // Get file icon based on file type
  function getFileIcon(fileType: string | null): string {
    if (!fileType) return 'mdi-file';
    const type = fileType.toLowerCase();
    if (type.includes('pdf')) return 'mdi-file-pdf-box';
    if (type.includes('doc') || type.includes('word')) return 'mdi-file-word-box';
    if (type.includes('xls') || type.includes('excel') || type.includes('spreadsheet'))
      return 'mdi-file-excel-box';
    if (type.includes('ppt') || type.includes('powerpoint')) return 'mdi-file-powerpoint-box';
    if (type.includes('txt')) return 'mdi-file-document-box';
    if (type.includes('zip') || type.includes('rar') || type.includes('archive'))
      return 'mdi-file-zip-box';
    return 'mdi-file';
  }

  // Watch category ID changes - reset pagination to first page
  watch(
    () => props.categoryId,
    () => {
      pageConfig.currentPage = 1;
      loadPapers();
    }
  );

  // Watch current view changes - reset pagination to first page
  watch(
    () => props.currentView,
    () => {
      // Clear expanded rows when view changes
      expandRowIds.value = [];
      pageConfig.currentPage = 1;
      loadPapers();
    }
  );

  // Listen for paper import events from API
  let unlistenPaperImported: (() => void) | null = null;

  onMounted(async () => {
    loadPapers();

    // Listen for paper:imported event from backend
    unlistenPaperImported = await listen('paper:imported', () => {
      console.info('Received paper:imported event, refreshing paper list');
      loadPapers();
    });
  });

  onUnmounted(() => {
    if (unlistenPaperImported) {
      unlistenPaperImported();
    }
  });

  // Expose load function for parent component refresh
  defineExpose({
    loadPapers,
  });
</script>

<template>
  <div class="paper-list">
    <!-- View indicator -->
    <div v-if="currentView === 'trash'" class="view-indicator">
      <v-icon size="small" color="warning" class="mr-2">mdi-delete</v-icon>
      <span class="text-caption">{{ $t('navigation.trash') }}</span>
    </div>

    <!-- Toolbar -->
    <PaperToolbar :on-refresh="loadPapers" :selected-category-id="categoryId" />

    <div class="table-container">
      <!-- Loading overlay -->
      <div v-if="loading" class="loading-overlay">
        <v-progress-circular indeterminate size="48" />
      </div>

      <div class="table-wrapper">
        <vxe-table
          ref="tableRef"
          id="paper-list-table"
          :data="papers"
          :expand-config="expandConfig"
          :column-config="{ resizable: true }"
          :custom-config="{ enabled: true, storage: true }"
          :menu-config="contextMenuConfig"
          :sort-config="{
            trigger: 'cell',
            defaultSort: sortConfig.defaultSort.field
              ? {
                  field: sortConfig.defaultSort.field,
                  order: sortConfig.defaultSort.order,
                }
              : undefined,
          }"
          :row-config="{ isCurrent: true, isHover: true, keyField: 'id' }"
          :cell-config="{ height: 32 }"
          :scroll-y="{ enabled: false }"
          :scroll-x="{ enabled: false }"
          :style="{
            '--vxe-ui-table-row-current-background-color': 'rgba(var(--v-theme-primary), 0.2)',
            '--vxe-ui-table-row-hover-current-background-color':
              'rgba(var(--v-theme-primary), 0.3)',
          }"
          height="100%"
          stripe
          border
          size="mini"
          @cell-click="cellClickEvent"
          @cell-dblclick="cellDblclickEvent"
          @sort-change="handleSortChange"
          @toggle-expand-change="handleToggleExpandChange"
          @menu-click="handleContextMenuClick"
          @menu-visible="handleContextMenuVisible"
        >
          <!-- Expand column (only for papers with attachments) -->
          <vxe-column type="expand" width="40" fixed="left">
            <template #content="{ row }">
              <div class="expand-row-content">
                <!-- Loading state -->
                <div
                  v-if="loadingAttachments.has(row.id) && !loadedAttachments.has(row.id)"
                  class="loading-attachments"
                >
                  <v-progress-circular indeterminate size="small" class="mr-2" />
                  <span class="text-caption">Loading attachments...</span>
                </div>
                <!-- Loaded attachments -->
                <div
                  v-else-if="row.attachments && row.attachments.length > 0"
                  class="attachments-list"
                >
                  <div class="attachments-header">
                    <v-icon size="small" class="mr-2">mdi-paperclip</v-icon>
                    <span class="text-subtitle-2">Attachments ({{ row.attachments.length }})</span>
                  </div>
                  <v-list density="compact" class="attachments-list-items">
                    <v-list-item
                      v-for="attachment in row.attachments"
                      :key="attachment.id"
                      class="attachment-item"
                    >
                      <template #prepend>
                        <v-icon :icon="getFileIcon(attachment.file_type)" size="small" />
                      </template>
                      <v-list-item-title>
                        {{ attachment.file_name || 'Unnamed file' }}
                      </v-list-item-title>
                      <v-list-item-subtitle v-if="attachment.file_type">
                        {{ attachment.file_type }}
                        <span v-if="attachment.created_at">
                          •
                          {{ new Date(attachment.created_at).toLocaleDateString() }}
                        </span>
                      </v-list-item-subtitle>
                    </v-list-item>
                  </v-list>
                </div>
                <!-- No attachments -->
                <div v-else class="no-attachments">
                  <v-icon size="small" class="mr-2">mdi-information</v-icon>
                  <span class="text-caption">No attachments</span>
                </div>
              </div>
            </template>
          </vxe-column>

          <!-- Title column -->
          <vxe-column field="title" title="Title" min-width="200" sortable show-overflow>
            <template #default="{ row }">
              <span class="text-truncate">{{ row.title }}</span>
            </template>
          </vxe-column>

          <!-- Authors column -->
          <vxe-column field="authors" title="Authors" width="15%" show-overflow>
            <template #default="{ row }">
              <v-chip v-if="row.authors && row.authors.length > 0" size="x-small" class="mr-1">
                {{ row.authors[0] }}
              </v-chip>
              <v-chip v-if="row.authors && row.authors.length > 1" size="x-small">
                +{{ row.authors.length - 1 }}
              </v-chip>
            </template>
          </vxe-column>

          <!-- Year column -->
          <vxe-column field="publication_year" title="Year" width="80" sortable />

          <!-- Journal column -->
          <vxe-column field="journal_name" title="Journal" width="35%" sortable show-overflow />
        </vxe-table>
      </div>

      <!-- 分页组件 -->
      <div class="pagination-container">
        <VxePager
          v-model:current-page="pageConfig.currentPage"
          v-model:page-size="pageConfig.pageSize"
          :total="pageConfig.total"
          :page-sizes="pageSizes"
          :layouts="['PrevPage', 'Number', 'NextPage', 'Sizes', 'Total']"
          @page-change="handlePageChange"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
  .paper-list {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .view-indicator {
    display: flex;
    align-items: center;
    padding: 8px 16px;
    background-color: rgba(255, 152, 0, 0.1);
    border-bottom: 1px solid rgba(255, 255, 255, 0.12);
    flex-shrink: 0;
  }

  .table-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
  }

  .table-wrapper {
    flex: 1;
    overflow: hidden;
  }

  .loading-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: rgba(0, 0, 0, 0.5);
    z-index: 100;
  }

  /* 分页样式 */
  .pagination-container {
    display: flex;
    justify-content: flex-end;
    padding: 12px 16px;
    background: rgb(var(--v-theme-surface));
    border-top: 1px solid rgba(255, 255, 255, 0.12);
    flex-shrink: 0;
  }

  .text-truncate {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  :deep(.vxe-table) {
    background-color: transparent;
  }

  :deep(.vxe-table--body .vxe-body--row.row--checked) {
    background-color: rgba(var(--v-theme-primary), 0.1);
  }

  :deep(.vxe-table--body .vxe-body--row:hover) {
    cursor: pointer;
  }

  :deep(.vxe-table .vxe-body--column) {
    padding: 4px 8px;
  }

  :deep(.vxe-table .vxe-header--column) {
    padding: 6px 8px;
    font-weight: 600;
    font-size: 13px;
  }

  :deep(.vxe-table--render-default .vxe-body--column) {
    font-size: 13px;
  }

  :deep(.vxe-table--render-default.size--mini .vxe-body--column) {
    padding: 2px 8px;
  }

  :deep(.vxe-table--render-default.size--mini .vxe-header--column) {
    padding: 4px 8px;
    font-size: 12px;
  }

  /* Expand row styles */
  .expand-row-content {
    padding: 12px 16px;
    background-color: var(--vxe-table-body-background-color);
  }

  .attachments-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .attachments-header {
    display: flex;
    align-items: center;
    font-weight: 600;
    padding-bottom: 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.12);
  }

  .attachments-list-items {
    background-color: transparent;
    padding: 0;
  }

  .attachment-item {
    padding: 4px 8px;
    min-height: auto;
  }

  .attachment-item :deep(.v-list-item-title) {
    font-size: 13px;
  }

  .attachment-item :deep(.v-list-item-subtitle) {
    font-size: 11px;
    opacity: 0.7;
  }

  .no-attachments {
    display: flex;
    align-items: center;
    opacity: 0.6;
    font-style: italic;
  }

  /* Expand icon rotation */
  .rotate-90 {
    transform: rotate(90deg);
  }

  /* Expand row styling */
  :deep(.vxe-table--expand-icon) {
    transition: transform 0.2s ease;
  }

  :deep(.vxe-body--row.row--expanded .vxe-body--expand-row) {
    background-color: var(--vxe-table-body-background-color);
  }

  /* Disable text selection on double click */
  :deep(.vxe-table) {
    user-select: none;
    -webkit-user-select: none;
  }

  :deep(.vxe-table .vxe-body--column) {
    user-select: text;
    -webkit-user-select: text;
  }
</style>
