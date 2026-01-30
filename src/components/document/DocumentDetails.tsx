import { useEffect, useState } from "react";
import {
  Typography,
  Tag,
  Space,
  Divider,
  Spin,
  Card,
  Select,
  Button,
  message,
  Input,
  InputNumber,
  Row,
  Col,
  TreeSelect,
} from "antd";
import {
  PlusOutlined,
  EditOutlined,
  SaveOutlined,
  CloseOutlined,
  FolderOpenOutlined,
} from "@ant-design/icons";
import { useI18n } from "../../lib/i18n";

const { TextArea } = Input;

// Helper for invoke (duplicate from DocumentList, maybe should move to a shared lib later)
async function invokeCommand<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

interface LabelDto {
  id: number;
  name: string;
  color: string;
}

interface CategoryNode {
  id: number;
  path: string;
  name: string;
  parent_id?: number | null;
  sort_order: number;
}

interface PaperDetailDto {
  id: number;
  title: string;
  abstract_text?: string;
  doi?: string;
  publication_year?: number;
  publication_date?: string;
  journal_name?: string;
  conference_name?: string;
  volume?: string;
  issue?: string;
  pages?: string;
  url?: string;
  citation_count?: number;
  read_status?: string;
  notes?: string;
  authors: string[];
  labels: LabelDto[];
  category_id?: number;
  category_name?: string;
}

interface DocumentDetailsProps {
  document?: {
    id: number;
    title: string;
    authors: string[];
    year: number;
    abstract?: string;
    keywords?: string[];
    fileType?: string;
    fileSize?: string;
    addedDate?: string;
    tags?: { id: number; name: string; color: string }[];
  } | null;
}

// Helper to build tree data for TreeSelect
function buildTreeData(categories: CategoryNode[]) {
  const nodeMap = new Map<number, any>();
  const rootNodes: any[] = [];

  // First pass: create nodes
  categories.forEach((cat) => {
    nodeMap.set(cat.id, {
      title: cat.name,
      value: cat.id,
      key: cat.id,
      children: [],
    });
  });

  // Second pass: build hierarchy
  categories.forEach((cat) => {
    const node = nodeMap.get(cat.id);
    if (cat.parent_id && nodeMap.has(cat.parent_id)) {
      const parent = nodeMap.get(cat.parent_id);
      parent.children.push(node);
    } else {
      rootNodes.push(node);
    }
  });

  return rootNodes;
}

export default function DocumentDetails({ document }: DocumentDetailsProps) {
  const { t } = useI18n();
  const [details, setDetails] = useState<PaperDetailDto | null>(null);
  const [loading, setLoading] = useState(false);
  const [allLabels, setAllLabels] = useState<LabelDto[]>([]);
  const [categories, setCategories] = useState<any[]>([]);
  const [addingLabel, setAddingLabel] = useState(false);
  const [addingCategory, setAddingCategory] = useState(false);
  const [actionLoading, setActionLoading] = useState(false);

  // Edit mode state
  const [isEditing, setIsEditing] = useState(false);
  const [editForm, setEditForm] = useState<PaperDetailDto | null>(null);

  useEffect(() => {
    if (document?.id) {
      loadPaperDetails(document.id);
      loadAllLabels();
      loadCategories();
    } else {
      setDetails(null);
      setIsEditing(false);
      setAddingCategory(false);
    }
  }, [document]);

  const loadAllLabels = async () => {
    try {
      const labels = await invokeCommand<LabelDto[]>("get_all_labels");
      setAllLabels(labels);
    } catch (error) {
      console.error("Failed to load labels:", error);
    }
  };

  const loadCategories = async () => {
    try {
      const cats = await invokeCommand<CategoryNode[]>("load_categories");
      setCategories(buildTreeData(cats));
    } catch (error) {
      console.error("Failed to load categories:", error);
    }
  };

  const loadPaperDetails = async (id: number) => {
    setLoading(true);
    try {
      const data = await invokeCommand<PaperDetailDto>("get_paper", { id });
      setDetails(data);
      return data;
    } catch (error) {
      console.error("Failed to load paper details:", error);
      // Fallback for dev/demo
      if (!(window as any).__TAURI_INTERNALS__) {
        const demoData = {
          id: id,
          title: "Demo Paper Title",
          abstract_text:
            "This is a demo abstract because the backend call failed or is not available.",
          authors: ["Demo Author"],
          publication_year: 2024,
          journal_name: "Demo Journal",
          labels: [],
        };
        setDetails(demoData);
        return demoData;
      }
      return null;
    } finally {
      setLoading(false);
    }
  };

  const notifyUpdate = (data: PaperDetailDto) => {
    window.dispatchEvent(new CustomEvent("paper-updated", { detail: data }));
  };

  const handleSetCategory = async (categoryId: number) => {
    if (!details) return;
    setActionLoading(true);
    try {
      await invokeCommand("update_paper_category", {
        paperId: details.id,
        categoryId: categoryId,
      });
      const updated = await loadPaperDetails(details.id);
      if (updated) notifyUpdate(updated);
      setAddingCategory(false);
      message.success("Category updated");
    } catch (error) {
      console.error("Failed to update category:", error);
      message.error("Failed to update category");
    } finally {
      setActionLoading(false);
    }
  };

  const handleAddLabel = async (labelId: number) => {
    if (!details) return;
    setActionLoading(true);
    try {
      await invokeCommand("add_paper_label", {
        paperId: details.id,
        labelId: labelId,
      });
      const updated = await loadPaperDetails(details.id);
      if (updated) notifyUpdate(updated);
      setAddingLabel(false);
      message.success("Label added");
    } catch (error) {
      console.error("Failed to add label:", error);
      message.error("Failed to add label");
    } finally {
      setActionLoading(false);
    }
  };

  const handleRemoveLabel = async (labelId: number) => {
    if (!details) return;
    try {
      await invokeCommand("remove_paper_label", {
        paperId: details.id,
        labelId: labelId,
      });
      const updated = await loadPaperDetails(details.id);
      if (updated) notifyUpdate(updated);
      message.success("Label removed");
    } catch (error) {
      console.error("Failed to remove label:", error);
      message.error("Failed to remove label");
    }
  };

  const startEdit = () => {
    setEditForm(JSON.parse(JSON.stringify(details))); // Deep copy
    setIsEditing(true);
  };

  const cancelEdit = () => {
    setIsEditing(false);
    setEditForm(null);
  };

  const saveEdit = async () => {
    if (!editForm || !details) return;
    setActionLoading(true);
    try {
      // 1. Update basic details
      await invokeCommand("update_paper_details", {
        payload: {
          id: details.id,
          title: editForm.title,
          publication_year: editForm.publication_year,
          journal_name: editForm.journal_name,
          conference_name: editForm.conference_name,
          volume: editForm.volume,
          issue: editForm.issue,
          pages: editForm.pages,
          url: editForm.url,
          doi: editForm.doi,
          abstract_text: editForm.abstract_text,
          notes: editForm.notes,
          read_status: editForm.read_status,
        },
      });

      // 2. Update category if changed
      if (editForm.category_id !== details.category_id) {
        await invokeCommand("update_paper_category", {
          paperId: details.id,
          categoryId: editForm.category_id || null,
        });
      }

      const updated = await loadPaperDetails(details.id);
      if (updated) notifyUpdate(updated);
      setIsEditing(false);
      message.success("Saved successfully");
    } catch (error) {
      console.error("Failed to save details:", error);
      message.error("Failed to save details");
    } finally {
      setActionLoading(false);
    }
  };

  if (!document) {
    return (
      <div
        style={{
          padding: 16,
          height: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        <Typography.Text type="secondary" italic>
          {t("document.select_to_view") || "Select a document to view details"}
        </Typography.Text>
      </div>
    );
  }

  if (loading && !details) {
    return (
      <div
        style={{
          padding: 16,
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
          height: "100%",
        }}
      >
        <Spin />
      </div>
    );
  }

  if (!details) {
    return (
      <div style={{ padding: 16, display: "flex", justifyContent: "center" }}>
        <Typography.Text type="danger">Failed to load details.</Typography.Text>
      </div>
    );
  }

  // Helper to handle input change
  const handleInputChange = (field: keyof PaperDetailDto, value: any) => {
    if (editForm) {
      setEditForm({ ...editForm, [field]: value });
    }
  };

  return (
    <div style={{ padding: 24, height: "100%", overflow: "auto" }}>
      {/* Header Actions */}
      <div
        style={{
          display: "flex",
          justifyContent: "flex-end",
          marginBottom: 16,
        }}
      >
        {isEditing ? (
          <Space>
            <Button onClick={cancelEdit} icon={<CloseOutlined />}>
              Cancel
            </Button>
            <Button
              type="primary"
              onClick={saveEdit}
              icon={<SaveOutlined />}
              loading={actionLoading}
            >
              Save
            </Button>
          </Space>
        ) : (
          <Button onClick={startEdit} icon={<EditOutlined />} type="text">
            Edit
          </Button>
        )}
      </div>

      {/* Title */}
      <div style={{ marginBottom: 16 }}>
        {isEditing ? (
          <TextArea
            value={editForm?.title}
            onChange={(e) => handleInputChange("title", e.target.value)}
            autoSize={{ minRows: 1, maxRows: 3 }}
            style={{ fontSize: "16px", fontWeight: "bold" }}
          />
        ) : (
          <Typography.Title level={5} style={{ margin: 0 }}>
            {details.title}
          </Typography.Title>
        )}
      </div>

      {/* Metadata Tags */}
      <Space size="small" wrap style={{ marginBottom: 16 }}>
        {isEditing ? (
          <>
            <TreeSelect
              style={{ width: 200 }}
              value={editForm?.category_id}
              dropdownStyle={{ maxHeight: 400, overflow: "auto" }}
              treeData={categories}
              placeholder={t("dialog.selectCategory") || "Select Category"}
              treeDefaultExpandAll
              allowClear
              onChange={(value) => handleInputChange("category_id", value)}
              size="small"
            />
            <InputNumber
              placeholder="Year"
              value={editForm?.publication_year}
              onChange={(v) => handleInputChange("publication_year", v)}
              size="small"
              style={{ width: 80 }}
            />
            <Input
              placeholder="Journal/Conf"
              value={editForm?.journal_name || editForm?.conference_name}
              onChange={(e) => {
                handleInputChange("journal_name", e.target.value);
              }}
              size="small"
              style={{ width: 120 }}
            />
            <Select
              value={editForm?.read_status}
              onChange={(v) => handleInputChange("read_status", v)}
              size="small"
              style={{ width: 100 }}
              options={[
                { value: "unread", label: "Unread" },
                { value: "reading", label: "Reading" },
                { value: "read", label: "Read" },
              ]}
            />
          </>
        ) : (
          <>
            {addingCategory ? (
              <TreeSelect
                style={{ width: 200 }}
                dropdownStyle={{ maxHeight: 400, overflow: "auto" }}
                treeData={categories}
                placeholder={t("dialog.selectCategory") || "Select Category"}
                treeDefaultExpandAll
                autoFocus
                defaultOpen
                value={details.category_id}
                onBlur={() => setAddingCategory(false)}
                onChange={handleSetCategory}
                size="small"
              />
            ) : details.category_name ? (
              <Tag
                icon={<FolderOpenOutlined />}
                color="orange"
                onClick={() => setAddingCategory(true)}
                style={{ cursor: "pointer" }}
              >
                {details.category_name}
              </Tag>
            ) : (
              <Tag
                onClick={() => setAddingCategory(true)}
                style={{
                  borderStyle: "dashed",
                  cursor: "pointer",
                  backgroundColor: "transparent",
                }}
              >
                <PlusOutlined /> {t("dialog.addCategory") || "Add Category"}
              </Tag>
            )}
            {details.publication_year && <Tag>{details.publication_year}</Tag>}
            {(details.journal_name || details.conference_name) && (
              <Tag color="blue">
                {details.journal_name || details.conference_name}
              </Tag>
            )}
            {details.read_status && (
              <Tag
                color={details.read_status === "read" ? "success" : "default"}
              >
                {details.read_status}
              </Tag>
            )}
          </>
        )}
      </Space>

      {/* Labels/Tags Section */}
      <div style={{ marginBottom: 16 }}>
        <Space size={[0, 8]} wrap>
          {details.labels &&
            details.labels.map((label) => (
              <Tag
                key={label.id}
                color={label.color}
                closable={!isEditing}
                onClose={(e) => {
                  e.preventDefault();
                  handleRemoveLabel(label.id);
                }}
              >
                {label.name}
              </Tag>
            ))}
          {/* Tag adding logic */}
          {addingLabel ? (
            <Select
              style={{ width: 120 }}
              size="small"
              autoFocus
              defaultOpen
              onBlur={() => setAddingLabel(false)}
              onChange={handleAddLabel}
              loading={actionLoading}
              placeholder="Select label"
              options={allLabels
                .filter((l) => !details.labels?.some((pl) => pl.id === l.id))
                .map((l) => ({ label: l.name, value: l.id }))}
            />
          ) : (
            <Tag
              onClick={() => {
                setAddingLabel(true);
                loadAllLabels();
              }}
              style={{
                borderStyle: "dashed",
                cursor: "pointer",
                backgroundColor: "transparent",
              }}
            >
              <PlusOutlined /> {t("dialog.addTag") || "Add Tag"}
            </Tag>
          )}
        </Space>
      </div>

      <Typography.Text
        style={{
          display: "block",
          marginBottom: 16,
          fontWeight: 500,
        }}
      >
        {details.authors.join(", ")}
      </Typography.Text>

      {/* Additional Fields Form in Edit Mode */}
      {isEditing && (
        <div
          style={{
            marginBottom: 16,
            backgroundColor: "#f5f5f5",
            padding: 12,
            borderRadius: 4,
          }}
        >
          <Row gutter={[16, 16]}>
            <Col span={12}>
              <Input
                placeholder="DOI"
                prefix="DOI: "
                value={editForm?.doi}
                onChange={(e) => handleInputChange("doi", e.target.value)}
              />
            </Col>
            <Col span={12}>
              <Input
                placeholder="URL"
                prefix="URL: "
                value={editForm?.url}
                onChange={(e) => handleInputChange("url", e.target.value)}
              />
            </Col>
            <Col span={8}>
              <Input
                placeholder="Volume"
                prefix="Vol: "
                value={editForm?.volume}
                onChange={(e) => handleInputChange("volume", e.target.value)}
              />
            </Col>
            <Col span={8}>
              <Input
                placeholder="Issue"
                prefix="Issue: "
                value={editForm?.issue}
                onChange={(e) => handleInputChange("issue", e.target.value)}
              />
            </Col>
            <Col span={8}>
              <Input
                placeholder="Pages"
                prefix="PP: "
                value={editForm?.pages}
                onChange={(e) => handleInputChange("pages", e.target.value)}
              />
            </Col>
          </Row>
        </div>
      )}

      {/* View Mode DOI/URL */}
      {!isEditing && (details.doi || details.url) && (
        <div style={{ marginBottom: 16 }}>
          {details.doi && (
            <Typography.Text style={{ fontSize: 12 }}>
              DOI:{" "}
              <a
                href={`https://doi.org/${details.doi}`}
                target="_blank"
                rel="noopener noreferrer"
              >
                {details.doi}
              </a>
            </Typography.Text>
          )}
          {details.url && (
            <Typography.Text style={{ fontSize: 12, display: "block" }}>
              URL:{" "}
              <a href={details.url} target="_blank" rel="noopener noreferrer">
                {details.url}
              </a>
            </Typography.Text>
          )}
        </div>
      )}

      <Divider style={{ margin: "16px 0" }} />

      {/* Abstract */}
      <div style={{ marginBottom: 24 }}>
        <Typography.Text strong>Abstract</Typography.Text>
        {isEditing ? (
          <TextArea
            value={editForm?.abstract_text}
            onChange={(e) => handleInputChange("abstract_text", e.target.value)}
            autoSize={{ minRows: 3, maxRows: 10 }}
            style={{ marginTop: 8 }}
          />
        ) : (
          details.abstract_text && (
            <Typography.Paragraph
              style={{
                whiteSpace: "pre-wrap",
                lineHeight: 1.6,
                marginTop: 8,
              }}
            >
              {details.abstract_text}
            </Typography.Paragraph>
          )
        )}
      </div>

      {/* Notes */}
      <div style={{ marginBottom: 24 }}>
        <Typography.Text strong>Notes</Typography.Text>
        {isEditing ? (
          <TextArea
            value={editForm?.notes}
            onChange={(e) => handleInputChange("notes", e.target.value)}
            autoSize={{ minRows: 2, maxRows: 6 }}
            style={{ marginTop: 8 }}
            placeholder="Add notes..."
          />
        ) : (
          details.notes && (
            <Card
              size="small"
              style={{
                backgroundColor:
                  "var(--ant-color-fill-alter, rgba(0, 0, 0, 0.02))",
                marginTop: 8,
              }}
            >
              <Typography.Text style={{ whiteSpace: "pre-wrap", fontSize: 12 }}>
                {details.notes}
              </Typography.Text>
            </Card>
          )
        )}
      </div>

      <div
        style={{
          marginTop: 32,
          paddingTop: 16,
          borderTop:
            "1px solid var(--ant-color-border-secondary, rgba(0, 0, 0, 0.06))",
        }}
      >
        <Typography.Text type="secondary" style={{ fontSize: 12 }}>
          ID: {details.id} | Citations: {details.citation_count || 0}
        </Typography.Text>
      </div>
    </div>
  );
}
