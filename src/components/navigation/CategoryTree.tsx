import { useState, useCallback } from "react";
import { Tree, Dropdown, Spin } from "antd";
import { FolderOutlined } from "@ant-design/icons";
import { PlusOutlined, EditOutlined, DeleteOutlined } from "@ant-design/icons";
import { useI18n } from "../../lib/i18n";
import { invoke } from "@tauri-apps/api/core";
import { useCategoryTree } from "./useCategoryTree";
import AddCategoryDialog, type { CategoryDialogData } from "../dialogs/AddCategoryDialog";
import EditCategoryDialog from "../dialogs/EditCategoryDialog";

interface ContextMenuState {
  x: number;
  y: number;
  nodeId: string | null;
  nodeName: string | null;
}

export default function CategoryTree() {
  const { t } = useI18n();
  const {
    treeData,
    loading,
    expandedKeys,
    setExpandedKeys,
    selectedKeys,
    setSelectedKeys,
    loadCategoriesData,
    onDrop,
  } = useCategoryTree();

  const [contextMenu, setContextMenu] = useState<ContextMenuState>({
    x: 0,
    y: 0,
    nodeId: null,
    nodeName: null,
  });

  const [dialog, setDialog] = useState<{
    open: boolean;
    mode: "add" | "edit" | null;
    data: CategoryDialogData | { path: string; name: string };
  }>({
    open: false,
    mode: null,
    data: {},
  });

  // Handle right-click context menu
  const handleRightClick = useCallback(
    (event: React.MouseEvent, node: any) => {
      event.preventDefault();
      event.stopPropagation();

      setContextMenu({
        x: event.clientX,
        y: event.clientY,
        nodeId: node.key as string,
        nodeName: node.title as string,
      });
    },
    [],
  );

  // Close context menu
  const closeContextMenu = useCallback(() => {
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
      setDialog({
        open: true,
        mode: "add",
        data: {
          parentPath: contextMenu.nodeId,
          parentName: contextMenu.nodeName || "",
        },
      });
      closeContextMenu();
    }
  }, [contextMenu.nodeId, contextMenu.nodeName]);

  // Handle edit category
  const handleEditCategory = useCallback(() => {
    if (contextMenu.nodeId && contextMenu.nodeName) {
      setDialog({
        open: true,
        mode: "edit",
        data: {
          path: contextMenu.nodeId,
          name: contextMenu.nodeName,
        },
      });
      closeContextMenu();
    }
  }, [contextMenu.nodeId, contextMenu.nodeName]);

  // Handle delete category
  const handleDeleteCategory = useCallback(async () => {
    if (!contextMenu.nodeId) return;

    try {
      await invoke("delete_category", { path: contextMenu.nodeId });
      console.info("Successfully deleted category:", contextMenu.nodeId);
      await loadCategoriesData();
      closeContextMenu();
    } catch (err) {
      console.error("Failed to delete category:", err);
    }
  }, [contextMenu.nodeId, loadCategoriesData, closeContextMenu]);

  // Context menu items
  const menuItems = [
    {
      key: "add",
      label: t("dialog.addSubcategory"),
      icon: <PlusOutlined style={{ color: "var(--ant-color-text)" }} />,
      onClick: handleAddSubcategory,
    },
    {
      key: "edit",
      label: t("dialog.rename"),
      icon: <EditOutlined style={{ color: "var(--ant-color-text)" }} />,
      onClick: handleEditCategory,
    },
    {
      type: "divider",
    },
    {
      key: "delete",
      label: t("dialog.delete"),
      icon: <DeleteOutlined style={{ color: "var(--ant-color-text)" }} />,
      onClick: handleDeleteCategory,
      danger: true,
    },
  ];

  // Handle tree select
  const handleSelect = useCallback(
    (selectedKeysValue: React.Key[]) => {
      setSelectedKeys(selectedKeysValue);
    },
    [setSelectedKeys],
  );

  // Handle tree expand
  const handleExpand = useCallback(
    (expandedKeysValue: React.Key[]) => {
      setExpandedKeys(expandedKeysValue);
    },
    [setExpandedKeys],
  );

  // Handle dialog close
  const closeDialog = useCallback(() => {
    setDialog({
      open: false,
      mode: null,
      data: {},
    });
  }, []);

  // Handle category operation complete
  const handleOperationComplete = useCallback(async () => {
    closeDialog();
    await loadCategoriesData();
  }, [closeDialog, loadCategoriesData]);

  // Convert tree data to Ant Design format
  const treeRenderData = treeData.map((node) => ({
    ...node,
    title: (
      <span style={{ color: "var(--ant-color-text)", fontSize: 13 }}>
        {node.title as string}
      </span>
    ),
    icon: <FolderOutlined style={{ color: "var(--ant-color-text)" }} />,
  }));

  if (loading) {
    return (
      <div
        style={{
          display: "flex",
          justifyContent: "center",
          padding: 16,
        }}
      >
        <Spin size="small" />
      </div>
    );
  }

  return (
    <div style={{ position: "relative" }}>
      {/* Tree Component */}
      <Tree
        treeData={treeRenderData}
        expandedKeys={expandedKeys}
        selectedKeys={selectedKeys}
        onExpand={handleExpand}
        onSelect={handleSelect}
        onRightClick={handleRightClick}
        draggable
        onDrop={onDrop}
        showIcon
        blockNode
        className="category-tree"
      />

      {/* Context Menu */}
      {contextMenu.x !== 0 && contextMenu.y !== 0 && (
        <Dropdown
          menu={{ items: menuItems }}
          trigger={[]}
          open
          onOpenChange={(open) => {
            if (!open) closeContextMenu();
          }}
          getPopupContainer={() => document.body}
          overlayStyle={{
            position: "fixed",
            left: contextMenu.x,
            top: contextMenu.y,
          }}
        >
          <div
            style={{
              position: "absolute",
              left: contextMenu.x,
              top: contextMenu.y,
              width: 0,
              height: 0,
            }}
          />
        </Dropdown>
      )}

      {/* Add Category Dialog */}
      <AddCategoryDialog
        open={dialog.open && dialog.mode === "add"}
        onClose={closeDialog}
        onCategoryCreated={handleOperationComplete}
        data={dialog.data as CategoryDialogData}
      />

      {/* Edit Category Dialog */}
      <EditCategoryDialog
        open={dialog.open && dialog.mode === "edit"}
        path={(dialog.data as { path: string }).path || ""}
        name={(dialog.data as { name: string }).name || ""}
        onClose={closeDialog}
        onCategoryUpdated={handleOperationComplete}
      />
    </div>
  );
}
