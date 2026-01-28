import { useState, useCallback } from "react";
import { Tree, Dropdown, Spin, Modal, Empty, Button, Alert } from "antd";
import {
  FolderOutlined,
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  ExclamationCircleOutlined,
} from "@ant-design/icons";
import { useI18n } from "../../lib/i18n";
import { invoke } from "@tauri-apps/api/core";
import AddCategoryDialog from "../dialogs/AddCategoryDialog";
import EditCategoryDialog from "../dialogs/EditCategoryDialog";
import { useCategoryTree } from "./useCategoryTree";

interface ContextMenuState {
  x: number;
  y: number;
  nodeId: string | null;
  nodeName: string | null;
}

interface DialogState {
  open: boolean;
  mode: "add" | "edit" | null;
  data: {
    parentPath?: string;
    parentName?: string;
    path?: string;
    name?: string;
  };
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
    loadCategories, // Fixed: match the name exported from hook
    onDrop,
    rawCount,
    errorMsg,
  } = useCategoryTree();

  const [contextMenu, setContextMenu] = useState<ContextMenuState>({
    x: 0,
    y: 0,
    nodeId: null,
    nodeName: null,
  });
  const [dialog, setDialog] = useState<DialogState>({
    open: false,
    mode: null,
    data: {},
  });

  const handleRightClick = useCallback(
    (event: React.MouseEvent, node: any) => {
      event.preventDefault();
      event.stopPropagation();
      // Select the node on right click if not already selected
      if (!selectedKeys.includes(node.key)) {
        setSelectedKeys([node.key]);
      }
      setContextMenu({
        x: event.clientX,
        y: event.clientY,
        nodeId: node.key,
        nodeName: node.title as string,
      });
    },
    [selectedKeys, setSelectedKeys],
  );

  const closeContextMenu = useCallback(() => {
    setContextMenu({ x: 0, y: 0, nodeId: null, nodeName: null });
  }, []);

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
  }, [contextMenu.nodeId, contextMenu.nodeName, closeContextMenu]);

  const handleEditCategory = useCallback(() => {
    if (contextMenu.nodeId && contextMenu.nodeName) {
      setDialog({
        open: true,
        mode: "edit",
        data: { path: contextMenu.nodeId, name: contextMenu.nodeName },
      });
      closeContextMenu();
    }
  }, [contextMenu.nodeId, contextMenu.nodeName, closeContextMenu]);

  const handleDeleteCategory = useCallback(() => {
    if (!contextMenu.nodeId) return;

    Modal.confirm({
      title: t("dialog.deleteCategory"),
      icon: <ExclamationCircleOutlined />,
      content: "确定要删除此分类及其所有子分类吗？此操作不可恢复。",
      okText: t("dialog.delete"),
      okType: "danger",
      cancelText: t("dialog.cancel"),
      onOk: async () => {
        try {
          await invoke("delete_category", { path: contextMenu.nodeId });
          await loadCategories();
          closeContextMenu();
        } catch (err) {
          console.error("Failed to delete category:", err);
          Modal.error({
            title: "删除失败",
            content: String(err),
          });
        }
      },
    });
  }, [contextMenu.nodeId, loadCategories, closeContextMenu, t]);

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
    { type: "divider" },
    {
      key: "delete",
      label: t("dialog.delete"),
      icon: <DeleteOutlined style={{ color: "var(--ant-color-text)" }} />,
      onClick: handleDeleteCategory,
      danger: true,
    },
  ];

  const handleSelect = useCallback(
    (selectedKeysValue: any) => setSelectedKeys(selectedKeysValue),
    [setSelectedKeys],
  );
  const handleExpand = useCallback(
    (expandedKeysValue: any) => setExpandedKeys(expandedKeysValue),
    [setExpandedKeys],
  );
  const closeDialog = useCallback(
    () => setDialog({ open: false, mode: null, data: {} }),
    [],
  );
  const handleOperationComplete = useCallback(async () => {
    closeDialog();
    await loadCategories();
  }, [closeDialog, loadCategories]);

  const titleRender = useCallback((node: any) => {
    return (
      <span style={{ color: "var(--ant-color-text)", fontSize: 13 }}>
        {node.title}
      </span>
    );
  }, []);

  // Add root category handler
  const handleAddRootCategory = () => {
    setDialog({
      open: true,
      mode: "add",
      data: { parentPath: undefined, parentName: undefined },
    });
  };

  if (loading) {
    return (
      <div style={{ display: "flex", justifyContent: "center", padding: 16 }}>
        <Spin size="small" />
      </div>
    );
  }

  if (errorMsg) {
    return (
      <div style={{ padding: 16 }}>
        <Alert
          message="加载分类失败"
          description={errorMsg}
          type="error"
          showIcon
          action={
            <Button size="small" onClick={() => loadCategories()}>
              重试
            </Button>
          }
        />
      </div>
    );
  }

  if (treeData.length === 0) {
    return (
      <div style={{ padding: "16px 8px" }}>
        {rawCount > 0 && (
          <Alert
            message="数据异常"
            description={`加载了 ${rawCount} 条分类数据，但无法构建树形结构。已尝试自动修复显示。`}
            type="warning"
            showIcon
            style={{ marginBottom: 16 }}
          />
        )}
        <Empty
          image={Empty.PRESENTED_IMAGE_SIMPLE}
          description={
            <span style={{ color: "var(--ant-color-text-secondary)" }}>
              暂无分类
            </span>
          }
        >
          <Button
            type="dashed"
            size="small"
            icon={<PlusOutlined />}
            onClick={handleAddRootCategory}
          >
            新建分类
          </Button>
        </Empty>
        <AddCategoryDialog
          open={dialog.open && dialog.mode === "add"}
          onClose={closeDialog}
          onCategoryCreated={handleOperationComplete}
          parentPath={dialog.data.parentPath}
          parentName={dialog.data.parentName}
        />
      </div>
    );
  }

  return (
    <div style={{ position: "relative" }}>
      <Tree
        treeData={treeData}
        expandedKeys={expandedKeys}
        selectedKeys={selectedKeys}
        onExpand={handleExpand}
        onSelect={handleSelect}
        onRightClick={(e) => handleRightClick(e.event, e.node)}
        draggable
        onDrop={onDrop}
        showIcon
        icon={<FolderOutlined style={{ color: "var(--ant-color-text)" }} />}
        titleRender={titleRender}
        blockNode
        className="category-tree"
      />
      {contextMenu.x !== 0 && contextMenu.y !== 0 && (
        <Dropdown
          menu={{ items: menuItems as any }}
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
      <AddCategoryDialog
        open={dialog.open && dialog.mode === "add"}
        onClose={closeDialog}
        onCategoryCreated={handleOperationComplete}
        parentPath={dialog.data.parentPath}
        parentName={dialog.data.parentName}
      />
      <EditCategoryDialog
        open={dialog.open && dialog.mode === "edit"}
        path={dialog.data.path || ""}
        name={dialog.data.name || ""}
        onClose={closeDialog}
        onCategoryUpdated={handleOperationComplete}
      />
    </div>
  );
}
