import { useState, useEffect, useCallback } from "react";
import { Table, Tag, Space, Modal } from "antd";
import { FileOutlined } from "@ant-design/icons";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { ColumnsType } from "antd/es/table";
import { useI18n } from "../../lib/i18n";
import { useAppStore } from "../../stores/useAppStore";

import DocumentToolbar from "./DocumentToolbar";

// Lazy load invoke helper - works in both Tauri and browser
async function invokeCommand<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

// Predefined color palette for tags
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

interface LabelDto {
  id: number;
  name: string;
  color: string;
}

interface AttachmentDto {
  id: number;
  paper_id: number;
  file_name: string;
  file_type: string;
  created_at: string;
}

interface PaperDto {
  id: number;
  title: string;
  publication_year?: number;
  journal_name?: string;
  conference_name?: string;
  authors: string[];
  labels: LabelDto[];
  attachment_count?: number;
}

interface DocumentListProps {
  onDocumentSelect: (document: any) => void;
  categoryId?: string | null;
}

const AttachmentList = ({ paperId }: { paperId: number }) => {
  const [attachments, setAttachments] = useState<AttachmentDto[]>([]);
  const [loading, setLoading] = useState(true);

  const loadAttachments = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invokeCommand<AttachmentDto[]>("get_attachments", {
        paperId: paperId,
      });
      setAttachments(data);
    } catch (error) {
      console.error("Failed to load attachments:", error);
    } finally {
      setLoading(false);
    }
  }, [paperId]);

  useEffect(() => {
    loadAttachments();

    const handleUpdate = (e: Event) => {
      const detail = (e as CustomEvent).detail;
      if (detail && detail.paperId === paperId) {
        loadAttachments();
      }
    };

    window.addEventListener("attachment-updated", handleUpdate);
    return () => window.removeEventListener("attachment-updated", handleUpdate);
  }, [loadAttachments, paperId]);

  if (loading) return <div style={{ padding: "8px 0" }}>Loading...</div>;
  if (attachments.length === 0)
    return (
      <div style={{ color: "#999", padding: "8px 0" }}>No attachments</div>
    );

  return (
    <div style={{ padding: "8px 0" }}>
      <Space direction="vertical" style={{ width: "100%" }}>
        {attachments.map((att) => (
          <div
            key={att.id}
            style={{
              display: "flex",
              alignItems: "center",
              gap: "8px",
              padding: "4px 8px",
              backgroundColor: "#f5f5f5",
              borderRadius: "4px",
            }}
          >
            <FileOutlined />
            <span style={{ flex: 1 }}>{att.file_name}</span>
            <span style={{ color: "#999", fontSize: "12px" }}>
              {new Date(att.created_at).toLocaleDateString()}
            </span>
          </div>
        ))}
      </Space>
    </div>
  );
};

export default function DocumentList({
  onDocumentSelect,
  categoryId,
}: DocumentListProps) {
  const { t } = useI18n();
  const { selectedDocument, accentColor } = useAppStore();
  const [rows, setRows] = useState<PaperDto[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadPapers();

    const handlePaperUpdate = (e: Event) => {
      const detail = (e as CustomEvent).detail as PaperDto;
      setRows((prev) =>
        prev.map((row) => (row.id === detail.id ? { ...row, ...detail } : row)),
      );
    };

    window.addEventListener("paper-updated", handlePaperUpdate);
    return () => window.removeEventListener("paper-updated", handlePaperUpdate);
  }, [categoryId]);

  const loadPapers = async () => {
    setLoading(true);
    try {
      let papers: PaperDto[];
      if (categoryId === "trash") {
        papers = await invokeCommand<PaperDto[]>("get_deleted_papers");
      } else if (categoryId) {
        // Load papers for specific category
        papers = await invokeCommand<PaperDto[]>("get_papers_by_category", {
          categoryPath: categoryId,
        });
      } else {
        // Load all papers
        papers = await invokeCommand<PaperDto[]>("get_all_papers");
      }
      setRows(papers);
      if (papers.length > 0) {
        onDocumentSelect(papers[0]);
      }
    } catch (error) {
      console.error("Failed to load papers:", error);
      // Demo data if backend fails (dev mode without tauri)
      const demoData = [
        {
          id: 1,
          title: "Attention Is All You Need",
          authors: ["Vaswani et al."],
          publication_year: 2017,
          conference_name: "NIPS",
          labels: [],
        },
      ];
      setRows(demoData);
      if (demoData.length > 0) {
        onDocumentSelect(demoData[0]);
      }
    } finally {
      setLoading(false);
    }
  };

  const handleDoubleClick = useCallback(async (record: PaperDto) => {
    console.info("Double clicked paper:", record.id, record.title);
    // Check if paper has PDF attachments
    if ((record.attachment_count || 0) > 0) {
      try {
        // Create a new window for PDF viewer
        const label = `pdf-viewer-${record.id}`;
        const webview = new WebviewWindow(label, {
          url: `http://localhost:1420/src/pdf-viewer.html`,
          title: record.title,
          width: 1000,
          height: 800,
          resizable: true,
          center: true,
          decorations: true,
          focus: true,
        });

        // Listen for creation errors
        webview.once("tauri://error", (e) => {
          console.error("Failed to create PDF viewer window:", e);
          Modal.error({
            title: "Failed to open PDF viewer",
            content: `Could not open PDF viewer window: ${e.payload}`,
          });
        });

        console.info(`Opening PDF viewer window for paper ${record.id}`);
      } catch (err) {
        console.error("Failed to open PDF viewer window:", err);
        Modal.error({
          title: "Failed to open PDF viewer",
          content: "Could not open PDF viewer window. Please try again.",
        });
      }
    } else {
      Modal.info({
        title: "No PDF attachment",
        content:
          "This paper has no PDF attachment. Please add a PDF file first.",
      });
    }
  }, []);

  const handleRowClick = useCallback(
    (record: PaperDto) => {
      onDocumentSelect(record);
    },
    [onDocumentSelect],
  );

  const columns: ColumnsType<PaperDto> = [
    {
      title: t("document.title"),
      dataIndex: "title",
      key: "title",
      ellipsis: true,
      width: 250,
    },
    {
      title: t("document.authors"),
      dataIndex: "authors",
      key: "authors",
      ellipsis: true,
      width: 200,
      render: (authors: string[]) => (
        <div style={{ overflow: "hidden", whiteSpace: "nowrap" }}>
          <Space size="small" style={{ flexWrap: "nowrap" }}>
            {authors?.map((author, index) => (
              <Tag key={index}>{author}</Tag>
            ))}
          </Space>
        </div>
      ),
    },
    {
      title: t("document.source"),
      dataIndex: "source",
      key: "source",
      ellipsis: true,
      width: 150,
      render: (_value, record: PaperDto) =>
        record.journal_name || record.conference_name || "",
    },
    {
      title: t("document.year"),
      dataIndex: "publication_year",
      key: "publication_year",
      ellipsis: true,
      width: 80,
    },
    {
      title: t("document.labels"),
      dataIndex: "labels",
      key: "labels",
      ellipsis: true,
      width: 320,
      render: (labels: LabelDto[]) => (
        <div style={{ overflow: "hidden", whiteSpace: "nowrap" }}>
          <Space size="small" style={{ flexWrap: "nowrap" }}>
            {labels?.map((label) => (
              <Tag
                key={label.id}
                style={{
                  backgroundColor: TAG_COLORS[label.color] || TAG_COLORS.blue,
                  color: "#fff",
                  maxWidth: "100px",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  verticalAlign: "top",
                }}
              >
                {label.name}
              </Tag>
            ))}
          </Space>
        </div>
      ),
    },
  ];

  return (
    <div
      style={{
        height: "100%",
        width: "100%",
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      {/* Toolbar */}
      <DocumentToolbar onRefresh={loadPapers} selectedCategoryId={categoryId} />

      {/* Table Container */}
      <div
        style={{
          flex: 1,
          overflow: "auto",
          minHeight: 0,
        }}
      >
        <Table
          dataSource={rows}
          columns={columns}
          loading={loading}
          rowKey="id"
          size="small"
          pagination={false}
          expandable={{
            expandedRowRender: (record) => (
              <div style={{ marginLeft: "50px" }}>
                <AttachmentList paperId={record.id} />
              </div>
            ),
            rowExpandable: (record) => (record.attachment_count || 0) > 0,
            expandIcon: ({ expanded, onExpand, record }) =>
              (record.attachment_count || 0) > 0 ? (
                <FileOutlined
                  onClick={(e) => onExpand(record, e)}
                  style={{
                    cursor: "pointer",
                    marginRight: 8,
                    color: expanded ? accentColor : undefined,
                  }}
                />
              ) : null,
          }}
          onRow={(record) => {
            const isSelected = selectedDocument?.id === record.id;
            return {
              onClick: () => {
                console.info("Row clicked", record.id);
                handleRowClick(record);
              },
              onDoubleClick: () => {
                console.info("Row double clicked", record.id);
                handleDoubleClick(record);
              },
              style: {
                cursor: "pointer",
                backgroundColor: isSelected ? `${accentColor}40` : undefined,
                userSelect: "none",
              },
            };
          }}
        />
      </div>
    </div>
  );
}
