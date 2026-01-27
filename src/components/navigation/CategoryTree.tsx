import { useState, useCallback } from "react";
import { useTree } from "@headless-tree/react";
import {
  syncDataLoaderFeature,
  dragAndDropFeature,
  selectionFeature,
} from "@headless-tree/core";
import { Dropdown, Button, Spin, Typography } from "antd";
import {
  FolderOutlined,
  MoreOutlined,
  EditOutlined,
  DeleteOutlined,
  PlusOutlined,
  CaretDownOutlined,
  CaretRightOutlined,
} from "@ant-design/icons";
import type { MenuProps } from "antd";
import { useI18n } from "../../lib/i18n";
import { invoke } from "@tauri-apps/api/core";
import AddCategoryDialog from "../dialogs/AddCategoryDialog";
import EditCategoryDialog from "../dialogs/EditCategoryDialog";
import { useCategoryTree, TreeDataItem } from "./useCategoryTree";

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

  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    nodeId: string | null;
    nodeName: string | null;
  }>({
    x: 0,
    y: 0,
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
        x: event.clientX,
        y: event.clientY,
        nodeId,
        nodeName,
      });
    },
    [],
  );

  // Close context menu
  const handleCloseContextMenu = useCallback(() => {
    setContextMenu({
      x: 0,
      y: 0,
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

  // Handle edit category
  const handleEditCategory = useCallback(() => {
    if (contextMenu.nodeId && contextMenu.nodeName) {
      setEditDialogPath(contextMenu.nodeId);
      setEditDialogName(contextMenu.nodeName);
      setShowEditDialog(true);
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

  // Context menu items
  const menuItems: MenuProps["items"] = [
    {
      key: "add",
      label: t("dialog.addSubcategory"),
      icon: <PlusOutlined />,
      onClick: handleAddSubcategory,
    },
    {
      key: "edit",
      label: t("dialog.rename"),
      icon: <EditOutlined />,
      onClick: handleEditCategory,
    },
    {
      type: "divider",
    },
    {
      key: "delete",
      label: t("dialog.delete"),
      icon: <DeleteOutlined />,
      onClick: handleDeleteCategory,
      danger: true,
    },
  ];

  if (loading) {
    return (
      <div style={{ display: "flex", justifyContent: "center", padding: 16 }}>
        <Spin size="small" />
      </div>
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
      contextMenu={contextMenu}
      handleCloseContextMenu={handleCloseContextMenu}
      menuItems={menuItems}
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
  handleCloseContextMenu: () => void;
  contextMenu: {
    x: number;
    y: number;
    nodeId: string | null;
    nodeName: string | null;
  };
  menuItems: MenuProps["items"];
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
}

function CategoryTreeContent({
  treeData,
  expandedItems,
  setExpandedItems,
  mapDropToBackend,
  loadCategoriesData,
  handleContextMenu,
  handleCloseContextMenu,
  contextMenu,
  menuItems,
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
    <div style={{ position: "relative" }}>
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
            <Dropdown
              key={itemId}
              menu={{ items: menuItems }}
              trigger={["contextMenu"]}
              open={
                contextMenu.x !== 0 &&
                contextMenu.nodeId === itemId &&
                contextMenu.y !== 0
              }
              onOpenChange={(open) => {
                if (!open) handleCloseContextMenu();
              }}
            >
              <div
                {...item.getProps()}
                className="tree-item"
                style={{
                  display: "flex",
                  alignItems: "center",
                  padding: "4px 0",
                  paddingLeft: level * 12,
                  paddingRight: 4,
                  borderRadius: 4,
                  cursor: "pointer",
                  transition: "background-color 150ms",
                  backgroundColor: isSelected
                    ? "var(--ant-color-primary-bg, rgba(24, 144, 255, 0.1))"
                    : "transparent",
                }}
                onContextMenu={(e: React.MouseEvent) =>
                  handleContextMenu(e, itemId, itemData.itemName)
                }
              >
                {/* Expand/Collapse Icon */}
                {hasChildren ? (
                  <div
                    style={{
                      width: 20,
                      height: 20,
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "center",
                      marginRight: 2,
                      cursor: "pointer",
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
                      <CaretDownOutlined style={{ fontSize: 12 }} />
                    ) : (
                      <CaretRightOutlined style={{ fontSize: 12 }} />
                    )}
                  </div>
                ) : (
                  <div style={{ width: 20, marginRight: 2 }} />
                )}

                {/* Folder Icon */}
                <div style={{ marginRight: 6, minWidth: 20 }}>
                  <FolderOutlined style={{ fontSize: 14 }} />
                </div>

                {/* Node Text */}
                <Typography.Text style={{ flex: 1, fontSize: 13 }}>
                  {itemData.itemName}
                </Typography.Text>

                {/* More Options Button */}
                <Button
                  type="text"
                  size="small"
                  icon={<MoreOutlined />}
                  onClick={(e: React.MouseEvent) => {
                    e.stopPropagation();
                    handleContextMenu(e, itemId, itemData.itemName);
                  }}
                  style={{
                    opacity: 0,
                  }}
                  className="tree-item-more-btn"
                />
              </div>
            </Dropdown>
          );
        })}
        <div style={tree.getDragLineStyle()} className="dragline" />
      </div>

      {/* Edit Category Dialog */}
      <EditCategoryDialog
        open={showEditDialog}
        categoryPath={editDialogPath}
        currentName={editDialogName}
        onClose={() => {
          setShowEditDialog(false);
          setEditDialogPath("");
          setEditDialogName("");
        }}
        onCategoryUpdated={async () => {
          setShowEditDialog(false);
          setEditDialogPath("");
          setEditDialogName("");
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
    </div>
  );
}
