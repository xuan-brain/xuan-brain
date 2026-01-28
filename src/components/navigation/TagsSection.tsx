import { useState, useEffect, useCallback } from "react";
import { Tag, Dropdown, Typography, Spin } from "antd";
import { DeleteOutlined } from "@ant-design/icons";
import type { MenuProps } from "antd";
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

  // Handle delete tag
  const handleDeleteTag = async (tag: Tag) => {
    const tagId = tag.id;
    try {
      await invokeCommand("delete_label", { id: tagId });
      await loadTags();
    } catch (error) {
      console.error("Failed to delete tag:", error);
      alert(`删除标签失败: ${error}`);
    }
  };

  // Handle update tag color
  const handleUpdateTagColor = async (tag: Tag, colorKey: string) => {
    try {
      await invokeCommand("update_label", {
        id: tag.id,
        color: colorKey,
      });
      await loadTags();
    } catch (error) {
      console.error("Failed to update tag color:", error);
      alert(`修改标签颜色失败: ${error}`);
    }
  };

  return (
    <div style={{ padding: "4px 0" }}>
      {/* Tags list */}
      {loading ? (
        <div style={{ padding: "8px 16px", textAlign: "center" }}>
          <Spin size="small" />
          <Typography.Text type="secondary" style={{ marginLeft: 8 }}>
            加载中...
          </Typography.Text>
        </div>
      ) : tags.length === 0 ? (
        <div style={{ padding: "8px 16px", textAlign: "center" }}>
          <Typography.Text type="secondary">暂无标签</Typography.Text>
        </div>
      ) : (
        <div
          style={{
            display: "flex",
            flexWrap: "wrap",
            gap: 4,
            padding: "0 4px",
          }}
        >
          {tags.map((tag) => {
            const menuItems: MenuProps["items"] = [
              {
                key: "delete",
                label: "删除标签",
                icon: <DeleteOutlined />,
                onClick: () => handleDeleteTag(tag),
                danger: true,
              },
              {
                type: "divider",
              },
              {
                key: "colors",
                label: (
                  <div>
                    <Typography.Text
                      type="secondary"
                      style={{
                        fontSize: 12,
                        marginBottom: 8,
                        display: "block",
                      }}
                    >
                      修改颜色
                    </Typography.Text>
                    <div style={{ display: "flex", flexWrap: "wrap", gap: 4 }}>
                      {Object.entries(TAG_COLORS).map(([key, value]) => (
                        <div
                          key={key}
                          onClick={(e) => {
                            e.stopPropagation();
                            handleUpdateTagColor(tag, key);
                          }}
                          style={{
                            width: 20,
                            height: 20,
                            borderRadius: "50%",
                            backgroundColor: value,
                            cursor: "pointer",
                            border:
                              tag.color === value
                                ? "2px solid currentColor"
                                : "1px solid var(--ant-color-border, #ddd)",
                            transition: "transform 0.2s",
                          }}
                          title={key}
                        />
                      ))}
                    </div>
                  </div>
                ),
              },
            ];

            return (
              <Dropdown
                key={tag.id}
                menu={{ items: menuItems }}
                trigger={["contextMenu"]}
              >
                <Tag
                  style={{
                    backgroundColor: tag.color,
                    color: "white",
                    cursor: "pointer",
                    marginBottom: 2,
                  }}
                >
                  {tag.name} ({tag.count})
                </Tag>
              </Dropdown>
            );
          })}
        </div>
      )}

      {/* Add Tag Dialog */}
      <AddTagDialog
        open={showAddDialog}
        onClose={() => setShowAddDialog(false)}
        onTagCreated={handleTagCreated}
      />
    </div>
  );
}
