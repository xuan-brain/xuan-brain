import { useState } from "react";
import { Button, Space, Modal, Input } from "antd";
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
  selectedCategoryId?: string | null;
}

export default function DocumentToolbar({
  onRefresh,
  selectedCategoryId,
}: DocumentToolbarProps) {
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
        categoryPath: selectedCategoryId,
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
      await invokeCommand("import_paper_by_doi", {
        doi: doiInput.trim(),
        categoryPath: selectedCategoryId,
      });

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
      <div
        style={{
          borderBottom: "1px solid var(--ant-color-border)",
          minHeight: 48,
          padding: "8px 16px",
          backgroundColor: "var(--ant-color-bg-container)",
        }}
      >
        <Space size="small">
          <Button size="small" onClick={handleDoiButtonClick}>
            {t("toolbar.doi")}
          </Button>
          <Button size="small" onClick={handleArxivButtonClick}>
            {t("toolbar.arxiv")}
          </Button>
        </Space>
      </div>

      {/* DOI Import Dialog */}
      <Modal
        open={doiDialogOpen}
        onCancel={handleDoiDialogClose}
        title={t("toolbar.importByDoi")}
        width={480}
        footer={
          <>
            <Button onClick={handleDoiDialogClose}>{t("dialog.cancel")}</Button>
            <Button type="primary" onClick={handleDoiSubmit}>
              {t("toolbar.import")}
            </Button>
          </>
        }
      >
        <div style={{ marginTop: 16 }}>
          <div style={{ marginBottom: 16, display: "block" }}>
            {t("toolbar.doiDescription")}
          </div>
          <Input
            autoFocus
            placeholder={t("toolbar.doiPlaceholder")}
            value={doiInput}
            onChange={(e) => setDoiInput(e.target.value)}
            onPressEnter={handleDoiSubmit}
          />
        </div>
      </Modal>

      {/* arXiv Import Dialog */}
      <Modal
        open={arxivDialogOpen}
        onCancel={handleArxivDialogClose}
        title={t("toolbar.importByArxiv")}
        width={480}
        footer={
          <>
            <Button onClick={handleArxivDialogClose}>
              {t("dialog.cancel")}
            </Button>
            <Button type="primary" onClick={handleArxivSubmit}>
              {t("toolbar.import")}
            </Button>
          </>
        }
      >
        <div style={{ marginTop: 16 }}>
          <div style={{ marginBottom: 16, display: "block" }}>
            {t("toolbar.arxivDescription")}
          </div>
          <Input
            autoFocus
            placeholder={t("toolbar.arxivPlaceholder")}
            value={arxivInput}
            onChange={(e) => setArxivInput(e.target.value)}
            onPressEnter={handleArxivSubmit}
          />
        </div>
      </Modal>
    </>
  );
}
