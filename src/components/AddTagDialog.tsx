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
import { Close, Check } from "@mui/icons-material";
import { useI18n } from "../lib/i18n";

// Lazy load invoke helper - works in both Tauri and browser
async function invokeCommand<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

// Predefined color palette (must match backend and TagsSection)
const TAG_COLORS: Record<string, string> = {
  red: "#ef4444",
  orange: "#f97316",
  amber: "#f59e0b",
  yellow: "#eab308",
  lime: "#84cc16",
  green: "#22c55e",
  emerald: "#10b981",
  teal: "#14b8a6",
  cyan: "#06b6d4",
  sky: "#0ea5e9",
  blue: "#3b82f6",
  indigo: "#6366f1",
  violet: "#8b5cf6",
  purple: "#a855f7",
  fuchsia: "#d946ef",
  pink: "#ec4899",
  rose: "#f43f5e",
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
  const { t } = useI18n();
  const [name, setName] = useState("");
  const [selectedColor, setSelectedColor] = useState("blue");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  // Reset form when dialog opens
  useEffect(() => {
    if (open) {
      setName("");
      setSelectedColor("blue");
      setError("");
    }
  }, [open]);

  const handleSubmit = async () => {
    if (!name.trim()) {
      setError(t("dialog.tagNameRequired"));
      return;
    }

    if (name.length > 30) {
      setError(t("dialog.tagNameMaxLength"));
      return;
    }

    setLoading(true);
    try {
      await invokeCommand("create_label", {
        name: name.trim(),
        color: selectedColor,
      });
      setName("");
      setSelectedColor("blue");
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
    setSelectedColor("blue");
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
          {t("dialog.addTag")}
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
        {/* Tag name input */}
        <TextField
          autoFocus
          margin="dense"
          label={t("dialog.tagName")}
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
          placeholder={t("dialog.enterTagName")}
        />

        {/* Color picker */}
        <Box sx={{ mt: 3 }}>
          <Typography
            variant="caption"
            component="legend"
            sx={{ mb: 1.5, display: "block", fontWeight: 500 }}
          >
            {t("dialog.selectColor")}
          </Typography>
          <Box
            sx={{
              display: "flex",
              flexWrap: "wrap",
              gap: 1,
            }}
          >
            {Object.entries(TAG_COLORS).map(([colorName, colorHex]) => (
              <Box
                key={colorName}
                onClick={() => setSelectedColor(colorName)}
                sx={{
                  width: 32,
                  height: 32,
                  borderRadius: "50%",
                  bgcolor: colorHex,
                  border: 2,
                  borderColor:
                    selectedColor === colorName
                      ? "text.primary"
                      : "transparent",
                  cursor: "pointer",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  transition: "all 0.2s",
                  "&:hover": {
                    transform: "scale(1.1)",
                  },
                }}
                title={colorName}
              >
                {selectedColor === colorName && (
                  <Check
                    sx={{
                      color: "white",
                      fontSize: 20,
                      filter: "drop-shadow(0 1px 2px rgba(0,0,0,0.3))",
                    }}
                  />
                )}
              </Box>
            ))}
          </Box>
        </Box>

        <Typography
          variant="caption"
          color="text.secondary"
          sx={{ mt: 2, display: "block" }}
        >
          {t("dialog.tagNameRules")}
        </Typography>
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClose} disabled={loading}>
          {t("dialog.cancel")}
        </Button>
        <Button
          onClick={handleSubmit}
          variant="contained"
          disabled={loading || !name.trim() || name.length > 30}
        >
          {loading ? t("dialog.adding") : t("dialog.add")}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
