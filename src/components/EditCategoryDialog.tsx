import { useState, useEffect } from "react";
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Button,
  IconButton,
  Typography,
  Box,
} from "@mui/material";
import { Close } from "@mui/icons-material";

// Lazy load invoke - works in both Tauri and browser
const invokeCommand = async <T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> => {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
};

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
    <Dialog
      open={open}
      onClose={closeDialog}
      maxWidth="sm"
      fullWidth
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          closeDialog();
        }
      }}
    >
      <DialogTitle>
        <Typography variant="h6" component="div">
          编辑分类
        </Typography>
        <IconButton
          aria-label="close"
          onClick={closeDialog}
          sx={{
            position: "absolute",
            right: 8,
            top: 8,
            color: "grey.500",
          }}
        >
          <Close />
        </IconButton>
      </DialogTitle>

      <DialogContent dividers>
        {/* Error message */}
        {errorMessage && (
          <Box
            sx={{
              mb: 2,
              p: 1.5,
              typography: "body2",
              color: "error.dark",
              bgcolor: "error.50",
              borderRadius: 1,
            }}
          >
            {errorMessage}
          </Box>
        )}

        {/* Category path info */}
        <Box sx={{ mb: 2 }}>
          <Typography variant="body2" color="text.secondary">
            分类路径:{" "}
            <Typography
              component="code"
              sx={{
                px: 0.5,
                py: 0.25,
                bgcolor: "action.hover",
                borderRadius: 0.5,
                fontSize: "0.75rem",
              }}
            >
              {categoryPath}
            </Typography>
          </Typography>
        </Box>

        {/* Category name input */}
        <TextField
          autoFocus
          margin="dense"
          label="分类名称"
          fullWidth
          variant="outlined"
          value={categoryName}
          onChange={(e) => {
            setCategoryName(e.target.value);
            setErrorMessage("");
          }}
          onKeyPress={handleKeyPress}
          disabled={loading}
          placeholder="输入分类名称..."
          helperText="最多 50 个字符"
        />
      </DialogContent>

      <DialogActions>
        <Button onClick={closeDialog} disabled={loading}>
          取消
        </Button>
        <Button
          onClick={handleUpdateCategory}
          variant="contained"
          disabled={loading || !categoryName.trim() || categoryName.length > 50}
        >
          {loading ? "保存中..." : "保存"}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
