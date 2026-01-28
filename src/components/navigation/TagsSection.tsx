import { useState, useEffect, useCallback } from "react";
import { Box, Typography, Chip, Menu, MenuItem, Divider } from "@mui/material";
import { Delete } from "@mui/icons-material";
import AddTagDialog from "../dialogs/AddTagDialog";

// Lazy load invoke helper - works in both Tauri and browser
async function invokeCommand<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

// Predefined color palette for tags
const TAG_COLORS: Record<string, string> = {
  red: "#ef4444",
  orange: "#f97316",
  amber: "#f59e0b",
  yellow: "#eab308",
  lime: "#84cc16",
  green: "#22c55e",
  emerald: "#10b981",
  teal: "#14b8a6",
  cyan: "#06b6d4",
  sky: "#0ea5e9",
  blue: "#3b82f6",
  indigo: "#6366f1",
  violet: "#8b5cf6",
  purple: "#a855f7",
  fuchsia: "#d946ef",
  pink: "#ec4899",
  rose: "#f43f5e",
};

interface Tag {
  id: number;
  name: string;
  count: number;
  color: string;
}

interface TagsSectionProps {
  onAddTag?: () => void;
}

export default function TagsSection(_props: TagsSectionProps) {
  const [tags, setTags] = useState<Tag[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [contextMenu, setContextMenu] = useState<{
    anchorEl: null | HTMLElement;
    tag: Tag | null;
  }>({
    anchorEl: null,
    tag: null,
  });

  // Load tags from backend
  const loadTags = useCallback(async () => {
    setLoading(true);
    try {
      const labels =
        await invokeCommand<Record<string, any>[]>("get_all_labels");

      const processedTags: Tag[] = labels.map((label) => ({
        id: label.id,
        name: label.name,
        count: label.document_count || 0,
        color: TAG_COLORS[label.color] || TAG_COLORS.blue,
      }));

      setTags(processedTags);
    } catch (error) {
      console.error("Failed to load labels:", error);
      // Use demo data as fallback
      setTags([
        { id: 1, name: "AI", count: 12, color: TAG_COLORS.blue },
        { id: 2, name: "Machine Learning", count: 8, color: TAG_COLORS.indigo },
        { id: 3, name: "Deep Learning", count: 6, color: TAG_COLORS.purple },
        { id: 4, name: "NLP", count: 5, color: TAG_COLORS.red },
        { id: 5, name: "Computer Vision", count: 4, color: TAG_COLORS.orange },
      ]);
    } finally {
      setLoading(false);
    }
  }, []);

  // Load on mount
  useEffect(() => {
    loadTags();
  }, [loadTags]);

  // Handle tag created
  const handleTagCreated = async () => {
    setShowAddDialog(false);
    await loadTags();
  };

  // Handle context menu
  const handleContextMenu = (
    event: React.MouseEvent<HTMLElement>,
    tag: Tag,
  ) => {
    event.preventDefault();
    event.stopPropagation();
    setContextMenu({
      anchorEl: event.currentTarget,
      tag,
    });
  };

  // Handle close context menu
  const handleCloseContextMenu = () => {
    setContextMenu({
      anchorEl: null,
      tag: null,
    });
  };

  // Handle delete tag
  const handleDeleteTag = async () => {
    if (!contextMenu.tag) return;

    const tagId = contextMenu.tag.id;
    try {
      await invokeCommand("delete_label", { id: tagId });
      handleCloseContextMenu();
      await loadTags();
    } catch (error) {
      console.error("Failed to delete tag:", error);
      alert(`删除标签失败: ${error}`);
    }
  };

  // Handle update tag color
  const handleUpdateTagColor = async (colorKey: string) => {
    if (!contextMenu.tag) return;

    try {
      await invokeCommand("update_label", {
        id: contextMenu.tag.id,
        color: colorKey,
      });
      handleCloseContextMenu();
      await loadTags();
    } catch (error) {
      console.error("Failed to update tag color:", error);
      alert(`修改标签颜色失败: ${error}`);
    }
  };

  return (
    <Box sx={{ py: 1 }}>
      {/* Header */}
      {/*<Box
        sx={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          px: 1,
          mb: 1,
        }}
      >
        <Typography variant="caption" color="text.secondary">
          标签（{tags.length}）
        </Typography>
        <IconButton
          size="small"
          onClick={() => setShowAddDialog(true)}
          aria-label="添加标签"
        >
          <MoreVert fontSize="small" />
        </IconButton>
      </Box>*/}

      {/* Tags list */}
      {loading ? (
        <Box sx={{ px: 2, py: 1, textAlign: "center" }}>
          <Typography variant="caption" color="text.secondary">
            加载中...
          </Typography>
        </Box>
      ) : tags.length === 0 ? (
        <Box sx={{ px: 2, py: 1, textAlign: "center" }}>
          <Typography variant="caption" color="text.secondary">
            暂无标签
          </Typography>
        </Box>
      ) : (
        <Box sx={{ display: "flex", flexWrap: "wrap", gap: 0.5, px: 1 }}>
          {tags.map((tag) => (
            <Chip
              key={tag.id}
              label={`${tag.name} (${tag.count})`}
              size="small"
              sx={{
                height: 24,
                bgcolor: tag.color,
                color: "white",
                "&:hover": {
                  bgcolor: tag.color,
                  opacity: 0.8,
                },
                cursor: "pointer",
              }}
              onClick={(e) => handleContextMenu(e, tag)}
            />
          ))}
        </Box>
      )}

      {/* Add Tag Dialog */}
      <AddTagDialog
        open={showAddDialog}
        onClose={() => setShowAddDialog(false)}
        onTagCreated={handleTagCreated}
      />

      {/* Context Menu */}
      <Menu
        anchorEl={contextMenu.anchorEl}
        open={Boolean(contextMenu.anchorEl)}
        onClose={handleCloseContextMenu}
      >
        <MenuItem onClick={handleDeleteTag} disabled={!contextMenu.tag}>
          <Delete sx={{ mr: 1 }} />
          删除标签
        </MenuItem>

        <Divider />

        <Box sx={{ p: 2, maxWidth: 220 }}>
          <Typography
            variant="caption"
            color="text.secondary"
            sx={{ mb: 1, display: "block" }}
          >
            修改颜色
          </Typography>
          <Box sx={{ display: "flex", flexWrap: "wrap", gap: 1 }}>
            {Object.entries(TAG_COLORS).map(([key, value]) => (
              <Box
                key={key}
                onClick={() => handleUpdateTagColor(key)}
                sx={{
                  width: 20,
                  height: 20,
                  borderRadius: "50%",
                  bgcolor: value,
                  cursor: "pointer",
                  border:
                    contextMenu.tag?.color === value
                      ? "2px solid black"
                      : "1px solid #ddd",
                  "&:hover": { transform: "scale(1.2)" },
                  transition: "transform 0.2s",
                }}
                title={key}
              />
            ))}
          </Box>
        </Box>
      </Menu>
    </Box>
  );
}
