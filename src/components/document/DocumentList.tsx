import { useState, useEffect } from "react";
import { Table, Tag, Space } from "antd";
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

interface PaperDto {
  id: number;
  title: string;
  publication_year?: number;
  journal_name?: string;
  conference_name?: string;
  authors: string[];
  labels: LabelDto[];
}

interface DocumentListProps {
  onDocumentSelect: (document: any) => void;
}

export default function DocumentList({ onDocumentSelect }: DocumentListProps) {
  const { t } = useI18n();
  const { accentColor } = useAppStore();
  const [rows, setRows] = useState<PaperDto[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedDocumentId, setSelectedDocumentId] = useState<number | null>(
    null,
  );

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
  }, []);

  const loadPapers = async () => {
    setLoading(true);
    try {
      const papers = await invokeCommand<PaperDto[]>("get_all_papers");
      console.info("Loaded papers:", papers.length);
      setRows(papers);
      if (papers.length > 0) {
        onDocumentSelect(papers[0]);
        setSelectedDocumentId(papers[0].id);
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
        setSelectedDocumentId(demoData[0].id);
      }
    } finally {
      setLoading(false);
    }
  };

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
      <DocumentToolbar onRefresh={loadPapers} />

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
          onRow={(record) => ({
            onClick: () => {
              onDocumentSelect(record);
              setSelectedDocumentId(record.id);
            },
            style: {
              cursor: "pointer",
              backgroundColor:
                selectedDocumentId === record.id
                  ? `${accentColor}20`
                  : undefined,
              color: selectedDocumentId === record.id ? accentColor : undefined,
              transition: "background-color 0.2s, color 0.2s",
            },
          })}
        />
      </div>
    </div>
  );
}
