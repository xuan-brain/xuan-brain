import { useState } from "react";
import {
  Button,
  Toolbar,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Typography,
  Box,
} from "@mui/material";
import { useI18n } from "../../lib/i18n";

// Lazy load invoke helper - works in both Tauri and browser
async function invokeCommand<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

interface DocumentToolbarProps {
  onRefresh?: () => void;
}

export default function DocumentToolbar({ onRefresh }: DocumentToolbarProps) {
  const { t } = useI18n();
  const [doiDialogOpen, setDoiDialogOpen] = useState(false);
  const [doiInput, setDoiInput] = useState("");
  const [arxivDialogOpen, setArxivDialogOpen] = useState(false);
  const [arxivInput, setArxivInput] = useState("");

  const handleDoiButtonClick = () => {
    setDoiDialogOpen(true);
    setDoiInput("");
  };

  const handleDoiDialogClose = () => {
    setDoiDialogOpen(false);
    setDoiInput("");
  };

  const handleArxivButtonClick = () => {
    setArxivDialogOpen(true);
    setArxivInput("");
  };

  const handleArxivDialogClose = () => {
    setArxivDialogOpen(false);
    setArxivInput("");
  };

  const handleArxivSubmit = async () => {
    if (!arxivInput.trim()) {
      return;
    }
    try {
      // Import paper by arXiv ID
      console.info("Importing paper with arXiv ID:", arxivInput.trim());
      await invokeCommand("import_paper_by_arxiv_id", {
        arxivId: arxivInput.trim(),
      });

      // Refresh the document list
      if (onRefresh) {
        await onRefresh();
      }

      handleArxivDialogClose();
    } catch (error) {
      console.error("Failed to import paper by arXiv ID:", error);
    }
  };

  const handleDoiSubmit = async () => {
    if (!doiInput.trim()) {
      return;
    }
    try {
      // Import paper by DOI
      console.info("Importing paper with DOI:", doiInput.trim());
      await invokeCommand("import_paper_by_doi", { doi: doiInput.trim() });

      // Refresh the document list
      if (onRefresh) {
        await onRefresh();
      }

      handleDoiDialogClose();
    } catch (error) {
      console.error("Failed to import paper by DOI:", error);
    }
  };

  return (
    <>
      {/* Toolbar */}
      <Toolbar
        variant="dense"
        sx={{
          bgcolor: "background.paper",
          borderBottom: "1px solid",
          borderColor: "divider",
          minHeight: "48px",
          px: 2,
        }}
      >
        <Button variant="outlined" size="small" onClick={handleDoiButtonClick}>
          {t("toolbar.doi")}
        </Button>
        <Button
          variant="outlined"
          size="small"
          onClick={handleArxivButtonClick}
          sx={{ ml: 1 }}
        >
          {t("toolbar.arxiv")}
        </Button>
      </Toolbar>

      {/* DOI Import Dialog */}
      <Dialog
        open={doiDialogOpen}
        onClose={handleDoiDialogClose}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>{t("toolbar.importByDoi")}</DialogTitle>
        <DialogContent>
          <Box sx={{ mt: 2 }}>
            <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
              {t("toolbar.doiDescription")}
            </Typography>
            <TextField
              autoFocus
              margin="dense"
              label={t("toolbar.doi")}
              fullWidth
              variant="outlined"
              placeholder={t("toolbar.doiPlaceholder")}
              value={doiInput}
              onChange={(e) => setDoiInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  handleDoiSubmit();
                }
              }}
            />
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={handleDoiDialogClose}>{t("dialog.cancel")}</Button>
          <Button onClick={handleDoiSubmit} variant="contained">
            {t("toolbar.import")}
          </Button>
        </DialogActions>
      </Dialog>

      {/* arXiv Import Dialog */}
      <Dialog
        open={arxivDialogOpen}
        onClose={handleArxivDialogClose}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>{t("toolbar.importByArxiv")}</DialogTitle>
        <DialogContent>
          <Box sx={{ mt: 2 }}>
            <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
              {t("toolbar.arxivDescription")}
            </Typography>
            <TextField
              autoFocus
              margin="dense"
              label="arXiv ID"
              fullWidth
              variant="outlined"
              placeholder={t("toolbar.arxivPlaceholder")}
              value={arxivInput}
              onChange={(e) => setArxivInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  handleArxivSubmit();
                }
              }}
            />
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={handleArxivDialogClose}>{t("dialog.cancel")}</Button>
          <Button onClick={handleArxivSubmit} variant="contained">
            {t("toolbar.import")}
          </Button>
        </DialogActions>
      </Dialog>
    </>
  );
}
