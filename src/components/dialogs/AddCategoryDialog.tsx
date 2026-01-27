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
import { useI18n } from "../../lib/i18n";

// Lazy load invoke helper - works in both Tauri and browser
async function invokeCommand<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

interface AddCategoryDialogProps {
  open: boolean;
  onClose: () => void;
  onCategoryCreated: () => void;
  parentPath?: string;
  parentName?: string;
}

export default function AddCategoryDialog({
  open,
  onClose,
  onCategoryCreated,
  parentPath,
  parentName,
}: AddCategoryDialogProps) {
  const { t } = useI18n();
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
      setError(t("dialog.categoryNameRequired"));
      return;
    }

    if (name.length > 50) {
      setError(t("dialog.categoryNameMaxLength"));
      return;
    }

    setLoading(true);
    try {
      await invokeCommand("create_category", {
        name: name.trim(),
        parentPath: parentPath || null,
      });
      console.info("Category created successfully:", name.trim());
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
          {parentPath ? t("dialog.addSubcategory") : t("dialog.addCategory")}
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
          label={t("dialog.categoryName")}
          fullWidth
          variant="outlined"
          value={name}
          onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
            setName(e.target.value);
            setError("");
          }}
          onKeyPress={handleKeyPress}
          error={!!error}
          helperText={error}
          disabled={loading}
          sx={{ mt: 2 }}
          placeholder={t("dialog.enterCategoryName")}
        />
        {parentName && (
          <TextField
            margin="dense"
            label={t("dialog.parentCategory")}
            fullWidth
            variant="filled"
            value={parentName}
            disabled
            sx={{ mt: 2 }}
          />
        )}
        <Typography
          variant="caption"
          color="text.secondary"
          sx={{ mt: 1, display: "block" }}
        >
          {t("dialog.categoryNameRules")}
        </Typography>
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClose} disabled={loading}>
          {t("dialog.cancel")}
        </Button>
        <Button
          onClick={handleSubmit}
          variant="contained"
          disabled={loading || !name.trim() || name.length > 50}
        >
          {loading ? t("dialog.adding") : t("dialog.add")}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
