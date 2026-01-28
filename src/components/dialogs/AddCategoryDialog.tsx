import { useState, useEffect } from "react";
import { Modal, Input, Button, Typography, Alert } from "antd";
import { CloseOutlined } from "@ant-design/icons";
import { useI18n } from "../../lib/i18n";

async function invokeCommand<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

export interface CategoryDialogData {
  parentPath?: string;
  parentName?: string;
}

export default function AddCategoryDialog({ open, onClose, onCategoryCreated, parentPath, parentName }: { open: boolean; onClose: () => void; onCategoryCreated: () => void; parentPath?: string; parentName?: string }) {
  const { t } = useI18n();
  const [name, setName] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (open) {
      setName("");
      setError("");
    }
  }, [open]);

  const handleClose = () => {
    setName("");
    setError("");
    if (onClose) {
      onClose();
    }
  };

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
      await invokeCommand("create_category", { name: name.trim(), parentPath: parentPath || null });
      console.info("Category created successfully:", name.trim());
      setName("");
      setError("");
      if (onCategoryCreated) {
        onCategoryCreated();
      }
      handleClose();
    } catch (err) {
      setError(err as string);
    } finally {
      setLoading(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !loading && name.trim() && name.length <= 50) {
      handleSubmit();
    }
  };

  return (
    <Modal open={open} onCancel={handleClose} title={<div style={{ position: "relative", paddingRight: 32 }}><Typography.Text strong>{parentPath ? t("dialog.addSubcategory") : t("dialog.addCategory")}</Typography.Text></div>} closeIcon={<CloseOutlined style={{ color: "var(--ant-color-text)" }} />} width={480} footer={<> <Button onClick={handleClose} disabled={loading}>{t("dialog.cancel")}</Button> <Button type="primary" onClick={handleSubmit} loading={loading} disabled={!name.trim() || name.length > 50}>{t("dialog.add")}</Button> </>}>
      {error && <Alert message={error} type="error" showIcon style={{ marginBottom: 16 }} />}
      <div style={{ marginBottom: 16 }}>
        <Input autoFocus placeholder={t("dialog.enterCategoryName")} value={name} onChange={(e) => { setName(e.target.value); setError(""); }} onPressEnter={handleKeyPress} status={error ? "error" : ""} disabled={loading} />
        {error && <Typography.Text type="danger" style={{ fontSize: 12 }}>{error}</Typography.Text>}
      </div>
      {parentName && <div style={{ marginBottom: 16 }}>
        <div style={{ marginBottom: 4 }}><Typography.Text type="secondary">{t("dialog.parentCategory")}</Typography.Text></div>
        <Input value={parentName} disabled />
      </div>}
      <Typography.Text type="secondary" style={{ fontSize: 12 }}>{t("dialog.categoryNameRules")}</Typography.Text>
    </Modal>
  );
}
