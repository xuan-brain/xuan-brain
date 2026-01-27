import { useState, useCallback } from "react";
import { useTree } from "@headless-tree/react";
import {
  syncDataLoaderFeature,
  dragAndDropFeature,
  selectionFeature,
} from "@headless-tree/core";
import {
  Box,
  Typography,
  IconButton,
  Menu,
  MenuItem,
  ListItemIcon,
  ListItemText,
  CircularProgress,
} from "@mui/material";
import {
  FolderOpen,
  MoreVert,
  Edit,
  Delete,
  Add,
  ExpandMore,
  ChevronRight,
} from "@mui/icons-material";
import { useI18n } from "../../lib/i18n";
import { invoke } from "@tauri-apps/api/core";
import AddCategoryDialog from "../dialogs/AddCategoryDialog";
import EditCategoryDialog from "../dialogs/EditCategoryDialog";
import { useCategoryTree, TreeDataItem } from "./useCategoryTree";

interface ContextMenuState {
  mouseX: number;
  mouseY: number;
  nodeId: string | null;
  nodeName: string | null;
}

export default function CategoryTree() {
  const { t } = useI18n();
  const {
    treeData,
    loading,
    expandedItems,
    setExpandedItems,
    loadCategoriesData,
    mapDropToBackend,
  } = useCategoryTree();

  const [contextMenu, setContextMenu] = useState<ContextMenuState>({
    mouseX: 0,
    mouseY: 0,
    nodeId: null,
    nodeName: null,
  });
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [addDialogParentPath, setAddDialogParentPath] = useState<string>("");
  const [addDialogParentName, setAddDialogParentName] = useState<string>("");
  const [showEditDialog, setShowEditDialog] = useState(false);
  const [editDialogPath, setEditDialogPath] = useState<string>("");
  const [editDialogName, setEditDialogName] = useState<string>("");

  // Handle right-click context menu
  const handleContextMenu = useCallback(
    (event: React.MouseEvent, nodeId: string, nodeName: string) => {
      event.preventDefault();
      event.stopPropagation();

      setContextMenu({
        mouseX: event.clientX,
        mouseY: event.clientY,
        nodeId,
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
      nodeName: null,
    });
  }, []);

  // Handle add subcategory
  const handleAddSubcategory = useCallback(() => {
    if (contextMenu.nodeId) {
      setAddDialogParentPath(contextMenu.nodeId);
      setAddDialogParentName(contextMenu.nodeName || "");
      setShowAddDialog(true);
    }
    handleCloseContextMenu();
  }, [contextMenu.nodeId, contextMenu.nodeName, handleCloseContextMenu]);

  // Handle delete category
  const handleDeleteCategory = useCallback(async () => {
    if (!contextMenu.nodeId) return;

    try {
      await invoke("delete_category", {
        path: contextMenu.nodeId,
      });
      console.info("Successfully deleted category:", contextMenu.nodeId);
      await loadCategoriesData();
    } catch (err) {
      console.error("Failed to delete category:", err);
    }

    handleCloseContextMenu();
  }, [contextMenu.nodeId, loadCategoriesData, handleCloseContextMenu]);

  if (loading) {
    return (
      <Box sx={{ display: "flex", justifyContent: "center", p: 2 }}>
        <CircularProgress size={20} />
      </Box>
    );
  }

  return (
    <CategoryTreeContent
      treeData={treeData}
      expandedItems={expandedItems}
      setExpandedItems={setExpandedItems}
      mapDropToBackend={mapDropToBackend}
      loadCategoriesData={loadCategoriesData}
      handleContextMenu={handleContextMenu}
      handleAddSubcategory={handleAddSubcategory}
      handleDeleteCategory={handleDeleteCategory}
      handleCloseContextMenu={handleCloseContextMenu}
      contextMenu={contextMenu}
      showAddDialog={showAddDialog}
      addDialogParentPath={addDialogParentPath}
      addDialogParentName={addDialogParentName}
      showEditDialog={showEditDialog}
      editDialogPath={editDialogPath}
      editDialogName={editDialogName}
      setShowAddDialog={setShowAddDialog}
      setShowEditDialog={setShowEditDialog}
      setEditDialogPath={setEditDialogPath}
      setEditDialogName={setEditDialogName}
      t={t}
    />
  );
}

interface CategoryTreeContentProps {
  treeData: Record<string, TreeDataItem>;
  expandedItems: string[];
  setExpandedItems: React.Dispatch<React.SetStateAction<string[]>>;
  mapDropToBackend: (
    items: any[],
    target: any,
  ) => {
    draggedPath: string;
    targetPath: string | null;
    position: "child" | "above" | "below";
  };
  loadCategoriesData: () => Promise<void>;
  handleContextMenu: (
    event: React.MouseEvent,
    nodeId: string,
    nodeName: string,
  ) => void;
  handleAddSubcategory: () => void;
  handleDeleteCategory: () => void;
  handleCloseContextMenu: () => void;
  contextMenu: ContextMenuState;
  showAddDialog: boolean;
  addDialogParentPath: string;
  addDialogParentName: string;
  showEditDialog: boolean;
  editDialogPath: string;
  editDialogName: string;
  setShowAddDialog: (show: boolean) => void;
  setShowEditDialog: (show: boolean) => void;
  setEditDialogPath: (path: string) => void;
  setEditDialogName: (name: string) => void;
  t: (key: string) => string;
}

function CategoryTreeContent({
  treeData,
  expandedItems,
  setExpandedItems,
  mapDropToBackend,
  loadCategoriesData,
  handleContextMenu,
  handleAddSubcategory,
  handleDeleteCategory,
  handleCloseContextMenu,
  contextMenu,
  showAddDialog,
  addDialogParentPath,
  addDialogParentName,
  showEditDialog,
  editDialogPath,
  editDialogName,
  setShowAddDialog,
  setShowEditDialog,
  setEditDialogPath,
  setEditDialogName,
  t,
}: CategoryTreeContentProps) {
  // Initialize tree
  const tree = useTree<TreeDataItem>({
    rootItemId: "root",
    getItemName: (item) => item.getItemData().itemName,
    isItemFolder: (item) => item.getItemData().isFolder !== false,
    dataLoader: {
      getItem: (itemId) => treeData[itemId],
      getChildren: (itemId) => treeData[itemId]?.childrenIds || [],
    },
    initialState: {
      expandedItems,
    },
    indent: 24,
    canReorder: true,
    onDrop: async (items, target) => {
      const { draggedPath, targetPath, position } = mapDropToBackend(
        items,
        target,
      );

      // Don't drop on itself
      if (draggedPath === targetPath) {
        return;
      }

      try {
        await invoke("move_category", {
          draggedPath,
          targetPath: targetPath === "root" ? null : targetPath,
          position,
        });
        console.info(
          "Successfully moved category:",
          draggedPath,
          "to",
          targetPath,
        );
        await loadCategoriesData();
      } catch (err) {
        console.error("Failed to move category:", err);
      }
    },
    features: [syncDataLoaderFeature, dragAndDropFeature, selectionFeature],
  });

  const items = tree.getItems();

  return (
    <Box sx={{ position: "relative" }}>
      <div {...tree.getContainerProps()} className="tree-root">
        {items.map((item) => {
          const itemData = item.getItemData();
          const level = item.getItemMeta().level;
          const isExpanded = item.isExpanded();
          const children = item.getChildren();
          const hasChildren = children.length > 0;
          const isSelected = item.isSelected();
          const itemId = item.getId();

          // Skip root node in rendering
          if (itemId === "root") return null;

          return (
            <Box
              key={itemId}
              {...item.getProps()}
              className="tree-item"
              sx={{
                display: "flex",
                alignItems: "center",
                py: 0.25,
                pl: level * 1.5,
                pr: 0.5,
                borderRadius: 1,
                cursor: "pointer",
                transition: "background-color 150ms",
                "&:hover": {
                  bgcolor: "action.hover",
                },
                bgcolor: isSelected ? "action.selected" : "transparent",
              }}
              onContextMenu={(e: React.MouseEvent) =>
                handleContextMenu(e, itemId, itemData.itemName)
              }
            >
              {/* Expand/Collapse Icon */}
              {hasChildren ? (
                <Box
                  sx={{
                    width: 20,
                    height: 20,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    mr: 0.3,
                  }}
                  onClick={(e) => {
                    e.stopPropagation();
                    const itemId = item.getId();
                    if (isExpanded) {
                      setExpandedItems(
                        expandedItems.filter((id) => id !== itemId),
                      );
                    } else {
                      setExpandedItems([...expandedItems, itemId]);
                    }
                  }}
                >
                  {isExpanded ? (
                    <ExpandMore sx={{ fontSize: 14 }} />
                  ) : (
                    <ChevronRight sx={{ fontSize: 14 }} />
                  )}
                </Box>
              ) : (
                <Box sx={{ width: 20, mr: 0.3 }} />
              )}

              {/* Folder Icon */}
              <Box sx={{ mr: 0.5, minWidth: 20 }}>
                <FolderOpen fontSize="small" />
              </Box>

              {/* Node Text */}
              <Typography variant="body2" sx={{ flex: 1 }}>
                {itemData.itemName}
              </Typography>

              {/* More Options Button */}
              <IconButton
                size="small"
                onClick={(e: React.MouseEvent) => {
                  e.stopPropagation();
                  handleContextMenu(e, itemId, itemData.itemName);
                }}
                sx={{
                  opacity: 0,
                  "&:hover": { opacity: 1 },
                  ".tree-item:hover &": { opacity: 1 },
                }}
              >
                <MoreVert fontSize="small" />
              </IconButton>
            </Box>
          );
        })}
        <div style={tree.getDragLineStyle()} className="dragline" />
      </div>

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
          <ListItemText>{t("dialog.addSubcategory")}</ListItemText>
        </MenuItem>
        <MenuItem
          onClick={() => {
            if (contextMenu.nodeId && contextMenu.nodeName) {
              setEditDialogPath(contextMenu.nodeId);
              setEditDialogName(contextMenu.nodeName);
              setShowEditDialog(true);
            }
            handleCloseContextMenu();
          }}
        >
          <ListItemIcon>
            <Edit fontSize="small" />
          </ListItemIcon>
          <ListItemText>{t("dialog.rename")}</ListItemText>
        </MenuItem>
        <MenuItem onClick={handleDeleteCategory}>
          <ListItemIcon>
            <Delete fontSize="small" />
          </ListItemIcon>
          <ListItemText>{t("dialog.delete")}</ListItemText>
        </MenuItem>
      </Menu>

      {/* Edit Category Dialog */}
      <EditCategoryDialog
        open={showEditDialog}
        categoryPath={editDialogPath}
        currentName={editDialogName}
        onClose={() => setShowEditDialog(false)}
        onCategoryUpdated={async () => {
          setShowEditDialog(false);
          await loadCategoriesData();
        }}
      />

      {/* Add Category Dialog */}
      <AddCategoryDialog
        open={showAddDialog}
        onClose={() => setShowAddDialog(false)}
        onCategoryCreated={async () => {
          setShowAddDialog(false);
          await loadCategoriesData();
        }}
        parentPath={addDialogParentPath}
        parentName={addDialogParentName}
      />
    </Box>
  );
}
