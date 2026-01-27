import { useState, useEffect } from "react";
import { Modal, Input, Button, Typography, Alert } from "antd";
import { CloseOutlined } from "@ant-design/icons";

// Lazy load invoke helper - works in both Tauri and browser
async function invokeCommand<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

interface EditCategoryDialogProps {
  open: boolean;
  categoryPath: string;
  currentName: string;
  onCategoryUpdated?: () => void;
  onClose?: () => void;
}

export default function EditCategoryDialog({
  open,
  categoryPath,
  currentName,
  onCategoryUpdated,
  onClose,
}: EditCategoryDialogProps) {
  const [categoryName, setCategoryName] = useState("");
  const [errorMessage, setErrorMessage] = useState("");
  const [loading, setLoading] = useState(false);

  // Update form when currentName changes or dialog opens
  useEffect(() => {
    if (open) {
      setCategoryName(currentName);
      setErrorMessage("");
    }
  }, [open, currentName]);

  const resetForm = () => {
    setCategoryName(currentName);
    setErrorMessage("");
  };

  const closeDialog = () => {
    resetForm();
    if (onClose) {
      onClose();
    }
  };

  const handleUpdateCategory = async () => {
    // Validate
    if (!categoryName.trim()) {
      setErrorMessage("请输入分类名称");
      return;
    }

    if (categoryName.trim().length > 50) {
      setErrorMessage("分类名称不能超过50个字符");
      return;
    }

    // Check if name changed
    if (categoryName.trim() === currentName.trim()) {
      closeDialog();
      return;
    }

    setLoading(true);
    try {
      await invokeCommand("update_category", {
        path: categoryPath,
        name: categoryName.trim(),
      });
      console.info(
        "Category updated successfully:",
        categoryPath,
        "to",
        categoryName.trim(),
      );

      // Reset and close
      resetForm();
      if (onClose) {
        onClose();
      }

      // Notify parent to refresh categories
      if (onCategoryUpdated) {
        onCategoryUpdated();
      }
    } catch (error) {
      console.error("Failed to update category:", error);
      setErrorMessage(`更新分类失败: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !loading) {
      handleUpdateCategory();
    }
  };

  return (
    <Modal
      open={open}
      onCancel={closeDialog}
      title={
        <div style={{ position: "relative", paddingRight: 32 }}>
          <Typography.Text strong>编辑分类</Typography.Text>
        </div>
      }
      closeIcon={<CloseOutlined />}
      width={480}
      footer={
        <>
          <Button onClick={closeDialog} disabled={loading}>
            取消
          </Button>
          <Button
            type="primary"
            onClick={handleUpdateCategory}
            loading={loading}
            disabled={!categoryName.trim() || categoryName.length > 50}
          >
            保存
          </Button>
        </>
      }
    >
      {/* Error message */}
      {errorMessage && (
        <Alert
          message={errorMessage}
          type="error"
          showIcon
          style={{ marginBottom: 16 }}
        />
      )}

      {/* Category path info */}
      <div style={{ marginBottom: 16 }}>
        <Typography.Text type="secondary">
          分类路径:{" "}
          <code
            style={{
              padding: "2px 6px",
              background: "rgba(0, 0, 0, 0.06)",
              borderRadius: 4,
            }}
          >
            {categoryPath}
          </code>
        </Typography.Text>
      </div>

      {/* Category name input */}
      <div>
        <Input
          autoFocus
          placeholder="输入分类名称..."
          value={categoryName}
          onChange={(e) => {
            setCategoryName(e.target.value);
            setErrorMessage("");
          }}
          onPressEnter={handleKeyPress}
          disabled={loading}
        />
        <Typography.Text type="secondary" style={{ fontSize: 12 }}>
          最多 50 个字符
        </Typography.Text>
      </div>
    </Modal>
  );
}
