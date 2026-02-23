<script setup lang="ts">
  import type { Comment } from '@/lib/api/clips';
  import { addClipComment, deleteClipComment, updateClipComment } from '@/lib/api/clips';
  import { ref } from 'vue';

  interface Props {
    clipId: string;
    comments: Comment[];
  }

  const props = defineProps<Props>();

  const emit = defineEmits<{
    commentsUpdated: [comments: Comment[]];
  }>();

  // State
  const newComment = ref('');
  const editingComment = ref<Comment | null>(null);
  const editContent = ref('');
  const loading = ref(false);
  const deleteDialogOpen = ref(false);
  const commentToDelete = ref<Comment | null>(null);

  // Format date for display
  function formatDate(dateString: string): string {
    try {
      const date = new Date(dateString);
      return date.toLocaleDateString(undefined, {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return dateString;
    }
  }

  // Add a new comment
  async function handleAddComment() {
    if (!newComment.value.trim() || loading.value) return;

    loading.value = true;
    try {
      const addedComment = await addClipComment(props.clipId, newComment.value.trim());
      const updatedComments = [...props.comments, addedComment];
      emit('commentsUpdated', updatedComments);
      newComment.value = '';
    } catch (error) {
      console.error('Failed to add comment:', error);
    } finally {
      loading.value = false;
    }
  }

  // Start editing a comment
  function startEdit(comment: Comment) {
    editingComment.value = comment;
    editContent.value = comment.content;
  }

  // Cancel editing
  function cancelEdit() {
    editingComment.value = null;
    editContent.value = '';
  }

  // Save edited comment
  async function saveEdit() {
    if (!editingComment.value || !editContent.value.trim() || loading.value) return;

    loading.value = true;
    try {
      const updated = await updateClipComment(
        props.clipId,
        editingComment.value.id,
        editContent.value.trim()
      );
      const updatedComments = props.comments.map((c) =>
        c.id === updated.id ? updated : c
      );
      emit('commentsUpdated', updatedComments);
      cancelEdit();
    } catch (error) {
      console.error('Failed to update comment:', error);
    } finally {
      loading.value = false;
    }
  }

  // Open delete confirmation dialog
  function confirmDelete(comment: Comment) {
    commentToDelete.value = comment;
    deleteDialogOpen.value = true;
  }

  // Delete comment
  async function handleDelete() {
    if (!commentToDelete.value || loading.value) return;

    loading.value = true;
    try {
      await deleteClipComment(props.clipId, commentToDelete.value.id);
      const updatedComments = props.comments.filter(
        (c) => c.id !== commentToDelete.value!.id
      );
      emit('commentsUpdated', updatedComments);
      deleteDialogOpen.value = false;
      commentToDelete.value = null;
    } catch (error) {
      console.error('Failed to delete comment:', error);
    } finally {
      loading.value = false;
    }
  }
</script>

<template>
  <div class="clip-comments">
    <!-- Header -->
    <div class="comments-header">
      <v-icon size="small" start>mdi-comment-multiple</v-icon>
      <span class="section-title">Comments</span>
      <v-chip size="x-small" color="primary" variant="flat" class="ml-2">
        {{ comments.length }}
      </v-chip>
    </div>

    <!-- Add Comment Form -->
    <div class="add-comment-form">
      <v-textarea
        v-model="newComment"
        placeholder="Add a comment..."
        rows="2"
        auto-grow
        density="compact"
        variant="outlined"
        hide-details
        :disabled="loading"
      />
      <div class="add-comment-actions">
        <v-spacer />
        <v-btn
          size="small"
          color="primary"
          variant="flat"
          :disabled="!newComment.trim() || loading"
          :loading="loading"
          @click="handleAddComment"
        >
          Add Comment
        </v-btn>
      </div>
    </div>

    <!-- Comments List -->
    <div v-if="comments.length > 0" class="comments-list">
      <v-card
        v-for="comment in comments"
        :key="comment.id"
        variant="outlined"
        class="comment-card"
      >
        <v-card-text class="pa-3">
          <!-- View Mode -->
          <div v-if="editingComment?.id !== comment.id">
            <div class="comment-content">{{ comment.content }}</div>
            <div class="comment-meta">
              <span class="text-caption text-grey">
                {{ formatDate(comment.created_at) }}
              </span>
              <span v-if="comment.updated_at !== comment.created_at" class="text-caption text-grey ml-2">
                (edited)
              </span>
              <v-spacer />
              <v-btn
                size="x-small"
                variant="text"
                density="compact"
                icon="mdi-pencil"
                @click="startEdit(comment)"
              />
              <v-btn
                size="x-small"
                variant="text"
                density="compact"
                icon="mdi-delete"
                color="error"
                @click="confirmDelete(comment)"
              />
            </div>
          </div>

          <!-- Edit Mode -->
          <div v-else>
            <v-textarea
              v-model="editContent"
              rows="2"
              auto-grow
              density="compact"
              variant="outlined"
              hide-details
              :disabled="loading"
            />
            <div class="edit-actions">
              <v-btn
                size="x-small"
                variant="text"
                :disabled="loading"
                @click="cancelEdit"
              >
                Cancel
              </v-btn>
              <v-btn
                size="x-small"
                color="primary"
                variant="flat"
                :disabled="!editContent.trim() || loading"
                :loading="loading"
                @click="saveEdit"
              >
                Save
              </v-btn>
            </div>
          </div>
        </v-card-text>
      </v-card>
    </div>

    <!-- Empty State -->
    <div v-else class="empty-comments">
      <v-icon size="32" color="grey">mdi-comment-outline</v-icon>
      <p class="text-caption text-grey mt-2">No comments yet</p>
    </div>

    <!-- Delete Confirmation Dialog -->
    <v-dialog v-model="deleteDialogOpen" max-width="400">
      <v-card>
        <v-card-title class="text-h6">Delete Comment</v-card-title>
        <v-card-text>Are you sure you want to delete this comment?</v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="deleteDialogOpen = false">Cancel</v-btn>
          <v-btn color="error" variant="flat" :loading="loading" @click="handleDelete">
            Delete
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<style scoped>
  .clip-comments {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .comments-header {
    display: flex;
    align-items: center;
    font-weight: 500;
    font-size: 14px;
  }

  .section-title {
    font-weight: 600;
  }

  .add-comment-form {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .add-comment-actions {
    display: flex;
    align-items: center;
  }

  .comments-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .comment-card {
    border-radius: 8px;
  }

  .comment-content {
    font-size: 14px;
    line-height: 1.5;
    white-space: pre-wrap;
    margin-bottom: 8px;
  }

  .comment-meta {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .edit-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 8px;
  }

  .empty-comments {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 24px;
    text-align: center;
  }
</style>
