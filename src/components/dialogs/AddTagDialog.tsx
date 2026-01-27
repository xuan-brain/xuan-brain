import { useState, useEffect } from "react";
import { Modal, Input, Button, Typography } from "antd";
import { CloseOutlined, CheckOutlined } from "@ant-design/icons";
import { useI18n } from "../../lib/i18n";

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
      console.info("Tag created successfully:", name.trim());
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
    <Modal
      open={open}
      onCancel={handleClose}
      title={
        <div style={{ position: "relative", paddingRight: 32 }}>
          <Typography.Text strong>{t("dialog.addTag")}</Typography.Text>
        </div>
      }
      closeIcon={<CloseOutlined />}
      width={480}
      footer={
        <>
          <Button onClick={handleClose} disabled={loading}>
            {t("dialog.cancel")}
          </Button>
          <Button
            type="primary"
            onClick={handleSubmit}
            loading={loading}
            disabled={!name.trim() || name.length > 30}
          >
            {t("dialog.add")}
          </Button>
        </>
      }
    >
      {/* Tag name input */}
      <div style={{ marginBottom: 16 }}>
        <Input
          autoFocus
          placeholder={t("dialog.enterTagName")}
          value={name}
          onChange={(e) => {
            setName(e.target.value);
            setError("");
          }}
          onPressEnter={handleKeyPress}
          status={error ? "error" : ""}
          disabled={loading}
        />
        {error && (
          <Typography.Text type="danger" style={{ fontSize: 12 }}>
            {error}
          </Typography.Text>
        )}
      </div>

      {/* Color picker */}
      <div style={{ marginBottom: 16 }}>
        <Typography.Text strong style={{ marginBottom: 8, display: "block" }}>
          {t("dialog.selectColor")}
        </Typography.Text>
        <div
          style={{
            display: "flex",
            flexWrap: "wrap",
            gap: 8,
          }}
        >
          {Object.entries(TAG_COLORS).map(([colorName, colorHex]) => (
            <div
              key={colorName}
              onClick={() => setSelectedColor(colorName)}
              style={{
                width: 32,
                height: 32,
                borderRadius: "50%",
                backgroundColor: colorHex,
                border:
                  selectedColor === colorName
                    ? "2px solid currentColor"
                    : "2px solid transparent",
                cursor: "pointer",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                transition: "all 0.2s",
              }}
              title={colorName}
            >
              {selectedColor === colorName && (
                <CheckOutlined
                  style={{
                    color: "white",
                    fontSize: 16,
                    filter: "drop-shadow(0 1px 2px rgba(0,0,0,0.3))",
                  }}
                />
              )}
            </div>
          ))}
        </div>
      </div>

      <Typography.Text type="secondary" style={{ fontSize: 12 }}>
        {t("dialog.tagNameRules")}
      </Typography.Text>
    </Modal>
  );
}
