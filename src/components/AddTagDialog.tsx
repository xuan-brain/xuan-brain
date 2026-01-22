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
} from "@mui/material";
import { Close } from "@mui/icons-material";

// Lazy load invoke - works in both Tauri and browser
const invokeCommand = async <T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> => {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
};

interface AddTagDialogProps {
  open: boolean;
  onClose: () => void;
  onTagCreated: () => void;
}

export default function AddTagDialog({
  open,
  onClose,
  onTagCreated,
}: AddTagDialogProps) {
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
      setError("标签名称不能为空");
      return;
    }

    if (name.length > 30) {
      setError("标签名称最多30个字符");
      return;
    }

    setLoading(true);
    try {
      await invokeCommand("create_label", {
        name: name.trim(),
      });
      setName("");
      setError("");
      onTagCreated();
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
    if (e.key === "Enter" && !loading && name.trim() && name.length <= 30) {
      handleSubmit();
    }
  };

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidth>
      <DialogTitle>
        <Typography variant="h6" component="div">
          添加标签
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
          label="标签名称"
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
          placeholder="请输入标签名称"
        />
        <Typography
          variant="caption"
          color="text.secondary"
          sx={{ mt: 1, display: "block" }}
        >
          标签名称不能为空，最多30个字符
        </Typography>
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClose} disabled={loading}>
          取消
        </Button>
        <Button
          onClick={handleSubmit}
          variant="contained"
          disabled={loading || !name.trim() || name.length > 30}
        >
          {loading ? "添加中..." : "添加"}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
