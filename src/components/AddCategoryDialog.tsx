import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Button,
  IconButton,
  Typography,
} from "@mui/material";
import { Close } from "@mui/icons-material";

interface AddCategoryDialogProps {
  open: boolean;
  onClose: () => void;
  onCategoryCreated: () => void;
  parentPath?: string;
}

export default function AddCategoryDialog({
  open,
  onClose,
  onCategoryCreated,
  parentPath,
}: AddCategoryDialogProps) {
  const [name, setName] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  // Reset form when dialog opens
  useEffect(() => {
    if (open) {
      setName("");
      setError("");
    }
  }, [open]);

  const handleSubmit = async () => {
    if (!name.trim()) {
      setError("名称不能为空");
      return;
    }

    if (name.length > 50) {
      setError("名称最多50个字符");
      return;
    }

    setLoading(true);
    try {
      await invoke("create_category", {
        name: name.trim(),
        parentPath: parentPath || null,
      });
      setName("");
      setError("");
      onCategoryCreated();
      onClose();
    } catch (err) {
      setError(err as string);
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    setName("");
    setError("");
    onClose();
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !loading && name.trim() && name.length <= 50) {
      handleSubmit();
    }
  };

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidth>
      <DialogTitle>
        <Typography variant="h6" component="div">
          {parentPath ? "添加子分类" : "添加分类"}
        </Typography>
        <IconButton
          aria-label="close"
          onClick={handleClose}
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
        <TextField
          autoFocus
          margin="dense"
          label="分类名称"
          fullWidth
          variant="outlined"
          value={name}
          onChange={(e) => {
            setName(e.target.value);
            setError("");
          }}
          onKeyPress={handleKeyPress}
          error={!!error}
          helperText={error}
          disabled={loading}
          sx={{ mt: 2 }}
          placeholder="请输入分类名称"
        />
        {parentPath && (
          <TextField
            margin="dense"
            label="父分类"
            fullWidth
            variant="filled"
            value={parentPath}
            disabled
            sx={{ mt: 2 }}
          />
        )}
        <Typography
          variant="caption"
          color="text.secondary"
          sx={{ mt: 1, display: "block" }}
        >
          分类名称不能为空，最多50个字符
        </Typography>
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClose} disabled={loading}>
          取消
        </Button>
        <Button
          onClick={handleSubmit}
          variant="contained"
          disabled={loading || !name.trim() || name.length > 50}
        >
          {loading ? "添加中..." : "添加"}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
