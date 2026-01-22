import { useState, useCallback, useEffect, useRef } from "react";
import { Tree, type NodeModel, type DropOptions, type TreeMethods, type RenderParams } from "@minoru/react-dnd-treeview";
import { DndProvider } from "react-dnd";
import { HTML5Backend } from "react-dnd-html5-backend";
import {
  Box,
  Typography,
  IconButton,
  Menu,
  MenuItem,
  ListItemIcon,
  ListItemText,
  TextField,
  Paper,
  CircularProgress,
} from "@mui/material";
import {
  Folder,
  FolderOpen,
  MoreVert,
  Edit,
  Delete,
  Add,
  Check,
  Close,
} from "@mui/icons-material";
import AddCategoryDialog from "./AddCategoryDialog";

// Lazy load invoke - works in both Tauri and browser
const invokeCommand = async <T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> => {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
};

// Extended NodeModel with our custom fields
interface ExtendedNodeModel extends NodeModel {
  path?: string;
  isEditing?: boolean;
}

interface CategoryNode {
  id: number;
  path: string;
  name: string;
  parent_id: number | null;
  sort_order: number;
}

interface ContextMenuState {
  mouseX: number;
  mouseY: number;
  nodeId: number | null;
  nodePath: string | null;
  nodeName: string | null;
}

export default function CategoryTree() {
  const [treeData, setTreeData] = useState<ExtendedNodeModel[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedNode, setSelectedNode] = useState<NodeModel["id"] | null>(null);
  const [contextMenu, setContextMenu] = useState<ContextMenuState>({
    mouseX: 0,
    mouseY: 0,
    nodeId: null,
    nodePath: null,
    nodeName: null,
  });
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [addDialogParentPath, setAddDialogParentPath] = useState<string>("");
  const treeRef = useRef<TreeMethods>(null);

  // Load categories from backend
  const loadCategoriesData = useCallback(async () => {
    setLoading(true);
    try {
      const categories = await invokeCommand<CategoryNode[]>("load_categories");

      // Convert backend data to react-dnd-treeview format
      const newTreeData: ExtendedNodeModel[] = categories.map((cat) => ({
        id: cat.id,
        parent: cat.parent_id || 0,
        text: cat.name,
        path: cat.path,
        isEditing: false,
      }));

      setTreeData(newTreeData);
    } catch (err) {
      console.error("Failed to load categories:", err);
      // Demo data fallback (works in both Tauri and browser)
      setTreeData([
        { id: 1, parent: 0, text: "计算机科学", path: "1" },
        { id: 2, parent: 1, text: "人工智能", path: "1.2" },
        { id: 3, parent: 1, text: "机器学习", path: "1.3" },
        { id: 4, parent: 0, text: "物理学", path: "4" },
      ]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadCategoriesData();
  }, [loadCategoriesData]);

  // Handle drop (drag and drop)
  const handleDrop = useCallback(
    async (tree: NodeModel[], options: DropOptions) => {
      const { dragSourceId, dropTargetId, destinationIndex } = options;

      if (!dragSourceId || !dropTargetId) {
        return;
      }

      const draggedNode = tree.find((node) => node.id === dragSourceId) as ExtendedNodeModel;
      if (!draggedNode) return;

      const targetNode = tree.find((node) => node.id === dropTargetId) as ExtendedNodeModel;
      if (!targetNode) return;

      // Calculate position
      let position: "above" | "below" | "child" = "child";
      const targetChildren = tree.filter(
        (node) => node.parent === dropTargetId,
      );

      if (destinationIndex === targetChildren.length) {
        // Dropped after all children
        position = "child";
      } else {
        // Determine if above or below based on destination index
        position = "below";
      }

      // Call backend to move category
      try {
        await invokeCommand("move_category", {
          draggedPath: draggedNode.path,
          targetPath: targetNode.path,
          position: position,
        });

        // Reload categories after successful move
        await loadCategoriesData();
      } catch (err) {
        console.error("Failed to move category:", err);
      }
    },
    [treeData, loadCategoriesData],
  );

  // Handle node selection
  const handleSelect = useCallback((nodeId: NodeModel["id"]) => {
    setSelectedNode(nodeId);
  }, []);

  // Handle right-click context menu
  const handleContextMenu = useCallback(
    (
      event: React.MouseEvent,
      nodeId: number,
      nodePath: string,
      nodeName: string,
    ) => {
      event.preventDefault();
      event.stopPropagation();

      setContextMenu({
        mouseX: event.clientX,
        mouseY: event.clientY,
        nodeId,
        nodePath,
        nodeName,
      });
    },
    [],
  );

  // Close context menu
  const handleCloseContextMenu = useCallback(() => {
    setContextMenu({
      mouseX: 0,
      mouseY: 0,
      nodeId: null,
      nodePath: null,
      nodeName: null,
    });
  }, []);

  // Handle add subcategory
  const handleAddSubcategory = useCallback(() => {
    if (contextMenu.nodePath) {
      setAddDialogParentPath(contextMenu.nodePath);
      setShowAddDialog(true);
    }
    handleCloseContextMenu();
  }, [contextMenu.nodePath, handleCloseContextMenu]);

  // Handle delete category
  const handleDeleteCategory = useCallback(async () => {
    if (!contextMenu.nodePath) return;

    try {
      await invokeCommand("delete_category", {
        path: contextMenu.nodePath,
      });
      await loadCategoriesData();
    } catch (err) {
      console.error("Failed to delete category:", err);
    }

    handleCloseContextMenu();
  }, [contextMenu.nodePath, loadCategoriesData, handleCloseContextMenu]);

  // Handle inline edit (double-click)
  const handleDoubleClick = useCallback((nodeId: number) => {
    setTreeData((prev) =>
      prev.map((node) =>
        node.id === nodeId ? { ...node, isEditing: true } : node,
      ),
    );
  }, []);

  // Handle inline edit save
  const handleEditSave = useCallback(
    async (nodeId: number, newName: string) => {
      const node = treeData.find((n) => n.id === nodeId) as ExtendedNodeModel;
      if (!node || !newName.trim()) {
        // Cancel edit if name is empty
        setTreeData((prev) =>
          prev.map((n) => (n.id === nodeId ? { ...n, isEditing: false } : n)),
        );
        return;
      }

      try {
        await invokeCommand("update_category", {
          path: node.path,
          name: newName.trim(),
        });
        await loadCategoriesData();
      } catch (err) {
        console.error("Failed to update category:", err);
        // Cancel edit on error
        setTreeData((prev) =>
          prev.map((n) => (n.id === nodeId ? { ...n, isEditing: false } : n)),
        );
      }
    },
    [treeData, loadCategoriesData],
  );

  // Handle inline edit cancel
  const handleEditCancel = useCallback((nodeId: number) => {
    setTreeData((prev) =>
      prev.map((n) => (n.id === nodeId ? { ...n, isEditing: false } : n)),
    );
  }, []);

  // Render node
  const renderNode = useCallback(
    (node: NodeModel, params: RenderParams) => {
      const extendedNode = node as ExtendedNodeModel;
      const nodePath = extendedNode.path;
      const isEditing = extendedNode.isEditing || false;
      const { depth, isOpen, onToggle } = params;

      return (
        <Box
          sx={{
            display: "flex",
            alignItems: "center",
            py: 0.5,
            px: depth === 0 ? 0 : 1,
            mx: depth === 0 ? 0 : -1,
            borderRadius: 1,
            cursor: "pointer",
            "&:hover": {
              bgcolor: "action.hover",
            },
            bgcolor:
              selectedNode === node.id ? "action.selected" : "transparent",
          }}
          onContextMenu={(e) =>
            handleContextMenu(e, node.id as number, nodePath || "", node.text)
          }
          onDoubleClick={() => handleDoubleClick(node.id as number)}
          onClick={() => handleSelect(node.id)}
        >
          {/* Expand/Collapse Icon */}
          {depth > 0 && (
            <Box
              sx={{
                width: 24,
                height: 24,
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                mr: 0.5,
              }}
              onClick={(e) => {
                e.stopPropagation();
                onToggle();
              }}
            >
              {isOpen ? <ExpandMoreIcon /> : <ExpandLessIcon />}
            </Box>
          )}

          {/* Folder Icon */}
          <Box sx={{ mr: 1, minWidth: 20 }}>
            {isOpen ? (
              <FolderOpen fontSize="small" />
            ) : (
              <Folder fontSize="small" />
            )}
          </Box>

          {/* Node Text or Edit Input */}
          {isEditing ? (
            <Box
              sx={{ display: "flex", alignItems: "center", flex: 1, gap: 0.5 }}
            >
              <TextField
                size="small"
                defaultValue={node.text}
                autoFocus
                onKeyDown={(e: React.KeyboardEvent<HTMLDivElement>) => {
                  if (e.key === "Enter") {
                    const target = e.target as HTMLInputElement;
                    handleEditSave(node.id as number, target.value);
                  } else if (e.key === "Escape") {
                    handleEditCancel(node.id as number);
                  }
                }}
                sx={{
                  flex: 1,
                  "& .MuiInputBase-root": {
                    height: 28,
                    fontSize: "0.875rem",
                  },
                }}
                onFocus={(e: React.FocusEvent<HTMLInputElement>) => e.target.select()}
              />
              <IconButton
                size="small"
                onClick={(e) => {
                  e.stopPropagation();
                  const input = document.querySelector(
                    "input:focus",
                  ) as HTMLInputElement;
                  if (input) {
                    handleEditSave(node.id as number, input.value);
                  }
                }}
              >
                <Check fontSize="small" />
              </IconButton>
              <IconButton
                size="small"
                onClick={(e) => {
                  e.stopPropagation();
                  handleEditCancel(node.id as number);
                }}
              >
                <Close fontSize="small" />
              </IconButton>
            </Box>
          ) : (
            <Typography variant="body2" sx={{ flex: 1 }}>
              {node.text}
            </Typography>
          )}

          {/* More Options Button */}
          <IconButton
            size="small"
            onClick={(e) => {
              e.stopPropagation();
              handleContextMenu(e, node.id as number, nodePath || "", node.text);
            }}
            sx={{
              opacity: 0,
              "&:hover": { opacity: 1 },
              ".MuiBox-root:hover &": { opacity: 1 },
            }}
          >
            <MoreVert fontSize="small" />
          </IconButton>
        </Box>
      );
    },
    [
      selectedNode,
      handleContextMenu,
      handleDoubleClick,
      handleSelect,
      handleEditSave,
      handleEditCancel,
    ],
  );

  // Custom render drag preview
  const renderDragPreview = useCallback((props: any) => {
    return (
      <Paper
        sx={{
          px: 1,
          py: 0.5,
          bgcolor: "primary.main",
          color: "primary.contrastText",
          opacity: 0.8,
          cursor: "grabbing",
        }}
      >
        <Typography variant="body2">{props.item.text}</Typography>
      </Paper>
    );
  }, []);

  if (loading) {
    return (
      <Box sx={{ display: "flex", justifyContent: "center", p: 2 }}>
        <CircularProgress size={20} />
      </Box>
    );
  }

  return (
    <DndProvider backend={HTML5Backend}>
      <Box sx={{ position: "relative" }}>
        <Tree
          ref={treeRef}
          tree={treeData}
          rootId={0}
          onDrop={handleDrop}
          dragPreviewRender={renderDragPreview}
          classes={{
            root: "tree-root",
            draggingSource: "dragging-source",
            dropTarget: "drop-target",
          }}
          sort={false}
          render={renderNode}
        />

        {/* Context Menu */}
        <Menu
          open={contextMenu.mouseY !== 0 && contextMenu.nodeId !== null}
          onClose={handleCloseContextMenu}
          anchorReference="anchorPosition"
          anchorPosition={
            contextMenu.mouseY !== 0 && contextMenu.nodeId !== null
              ? { top: contextMenu.mouseY, left: contextMenu.mouseX }
              : undefined
          }
        >
          <MenuItem onClick={handleAddSubcategory}>
            <ListItemIcon>
              <Add fontSize="small" />
            </ListItemIcon>
            <ListItemText>添加子分类</ListItemText>
          </MenuItem>
          <MenuItem
            onClick={() => {
              handleDoubleClick(contextMenu.nodeId!);
              handleCloseContextMenu();
            }}
          >
            <ListItemIcon>
              <Edit fontSize="small" />
            </ListItemIcon>
            <ListItemText>重命名</ListItemText>
          </MenuItem>
          <MenuItem onClick={handleDeleteCategory}>
            <ListItemIcon>
              <Delete fontSize="small" />
            </ListItemIcon>
            <ListItemText>删除</ListItemText>
          </MenuItem>
        </Menu>

        {/* Add Category Dialog */}
        <AddCategoryDialog
          open={showAddDialog}
          onClose={() => setShowAddDialog(false)}
          onCategoryCreated={async () => {
            setShowAddDialog(false);
            await loadCategoriesData();
          }}
          parentPath={addDialogParentPath}
        />
      </Box>
    </DndProvider>
  );
}

// Helper icons
const ExpandMoreIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="16"
    height="16"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <polyline points="6 9 12 15 18 9"></polyline>
  </svg>
);

const ExpandLessIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="16"
    height="16"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <polyline points="18 15 12 9 6 15"></polyline>
  </svg>
);
