import { useEffect, useState } from "react";
import { Typography, Tag, Space, Divider, Spin, Card } from "antd";
import { useI18n } from "../../lib/i18n";

// Helper for invoke (duplicate from DocumentList, maybe should move to a shared lib later)
async function invokeCommand<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
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

export default function DocumentDetails({ document }: DocumentDetailsProps) {
  const { t } = useI18n();
  const [details, setDetails] = useState<PaperDetailDto | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (document?.id) {
      loadPaperDetails(document.id);
    } else {
      setDetails(null);
    }
  }, [document]);

  const loadPaperDetails = async (id: number) => {
    setLoading(true);
    try {
      const data = await invokeCommand<PaperDetailDto>("get_paper", { id });
      setDetails(data);
    } catch (error) {
      console.error("Failed to load paper details:", error);
      // Fallback for dev/demo without backend
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      if (!(window as any).__TAURI_INTERNALS__) {
        setDetails({
          id: id,
          title: "Demo Paper Title",
          abstract_text:
            "This is a demo abstract because the backend call failed or is not available.",
          authors: ["Demo Author"],
          publication_year: 2024,
          journal_name: "Demo Journal",
        });
      }
    } finally {
      setLoading(false);
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

  if (loading) {
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

  return (
    <div style={{ padding: 24, height: "100%", overflow: "auto" }}>
      <Typography.Title level={5} style={{ marginBottom: 16 }}>
        {details.title}
      </Typography.Title>

      <Space size="small" wrap style={{ marginBottom: 16 }}>
        {details.publication_year && <Tag>{details.publication_year}</Tag>}
        {(details.journal_name || details.conference_name) && (
          <Tag color="blue">
            {details.journal_name || details.conference_name}
          </Tag>
        )}
        {details.read_status && (
          <Tag color={details.read_status === "read" ? "success" : "default"}>
            {details.read_status}
          </Tag>
        )}
      </Space>

      <Typography.Text
        style={{
          display: "block",
          marginBottom: 16,
          fontWeight: 500,
        }}
      >
        {details.authors.join(", ")}
      </Typography.Text>

      {(details.doi || details.url) && (
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

      {details.abstract_text && (
        <div style={{ marginBottom: 24 }}>
          <Typography.Text strong>Abstract</Typography.Text>
          <Typography.Paragraph
            style={{
              whiteSpace: "pre-wrap",
              lineHeight: 1.6,
              marginTop: 8,
            }}
          >
            {details.abstract_text}
          </Typography.Paragraph>
        </div>
      )}

      {details.notes && (
        <div style={{ marginBottom: 24 }}>
          <Typography.Text strong>Notes</Typography.Text>
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
        </div>
      )}

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
