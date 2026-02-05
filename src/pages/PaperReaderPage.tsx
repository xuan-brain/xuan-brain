import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Layout, Spin, Empty, Button, message } from "antd";
import { ArrowLeftOutlined } from "@ant-design/icons";
import { useTabsStore } from "@/stores/useTabsStore";
import DocumentDetails from "@/components/document/DocumentDetails";
import { PdfAnnotator } from "pdfjs-annotation-extension-for-react";
import "pdfjs-annotation-extension-for-react/style";

const { Sider, Content } = Layout;

// Helper for invoke
async function invokeCommand<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

interface PdfAttachmentInfo {
  file_path: string;
  file_name: string;
  paper_id: number;
  paper_title: string;
}

interface ActionsProps {
  getAnnotations: () => unknown;
}

export default function PaperReaderPage() {
  const { paperId } = useParams<{ paperId: string }>();
  const navigate = useNavigate();
  const { addTab, removeTab, getTabByPaperId } = useTabsStore();

  const [loading, setLoading] = useState(true);
  const [pdfData, setPdfData] = useState<Uint8Array | null>(null);
  const [error, setError] = useState<string>("");
  const [paperInfo, setPaperInfo] = useState<{ title: string } | null>(null);
  const [originalPdfData, setOriginalPdfData] = useState<Uint8Array | null>(
    null,
  );
  const [saving, setSaving] = useState(false);
  const [exporting, setExporting] = useState(false);

  const id = paperId ? parseInt(paperId, 10) : 0;

  // Initialize tab
  useEffect(() => {
    if (paperInfo && paperInfo.title) {
      const existingTab = getTabByPaperId(id);
      if (!existingTab) {
        addTab(id, paperInfo.title, `/papers/${paperId}`);
      }
    }
  }, [id, paperInfo, addTab, getTabByPaperId, paperId]);

  // Load PDF
  useEffect(() => {
    const loadPDF = async () => {
      if (!id) {
        setError("Invalid paper ID");
        setLoading(false);
        return;
      }

      try {
        setLoading(true);
        setError("");

        const info = await invokeCommand<PdfAttachmentInfo>(
          "get_pdf_attachment_path",
          { paperId: id },
        );

        const contents = await invokeCommand<number[]>("read_pdf_file", {
          filePath: info.file_path,
        });

        const uint8Array = new Uint8Array(contents);
        const pdfDataCopy = new Uint8Array(uint8Array);

        setPdfData(uint8Array);
        setOriginalPdfData(pdfDataCopy);
        setPaperInfo({ title: info.paper_title });

        console.info("PDF loaded successfully:", info.paper_title);
      } catch (err) {
        console.error("Failed to load PDF:", err);
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setLoading(false);
      }
    };

    loadPDF();
  }, [id]);

  const handleClose = () => {
    const tab = getTabByPaperId(id);
    if (tab) {
      removeTab(tab.id);
    }
    navigate("/papers");
  };

  const handleSave = async (annotations: unknown) => {
    if (!paperInfo) return;

    try {
      const info = await invokeCommand<PdfAttachmentInfo>(
        "get_pdf_attachment_path",
        { paperId: id },
      );

      await invokeCommand("save_annotations_data", {
        filePath: info.file_path,
        annotationsJson: JSON.stringify(annotations),
      });

      message.success("Annotations saved");
    } catch (err) {
      console.error("Failed to save annotations:", err);
      message.error("Failed to save annotations");
    }
  };

  const handleExport = async () => {
    if (!paperInfo || !pdfData) return;

    const { save } = await import("@tauri-apps/plugin-dialog");
    const path = await save({
      defaultPath: `${paperInfo.title.replace(/[^a-zA-Z0-9\u4e00-\u9fa5]/g, "_")}_annotated.pdf`,
      filters: [
        {
          name: "PDF",
          extensions: ["pdf"],
        },
      ],
    });

    if (path && typeof path === "string") {
      setExporting(true);
      try {
        const info = await invokeCommand<PdfAttachmentInfo>(
          "get_pdf_attachment_path",
          { paperId: id },
        );

        await invokeCommand("export_pdf_with_annotations", {
          sourceFilePath: info.file_path,
          exportFilePath: path,
          pdfData: Array.from(pdfData),
        });

        message.success("PDF exported successfully");
      } catch (err) {
        console.error("Failed to export PDF:", err);
        message.error(`Failed to export PDF: ${err}`);
      } finally {
        setExporting(false);
      }
    }
  };

  const handleSavePDF = async (props: ActionsProps) => {
    if (!paperInfo || !originalPdfData) return;

    setSaving(true);
    try {
      const info = await invokeCommand<PdfAttachmentInfo>(
        "get_pdf_attachment_path",
        { paperId: id },
      );

      const currentAnnotations = props.getAnnotations();

      await invokeCommand("save_pdf_with_annotations_data", {
        filePath: info.file_path,
        pdfData: Array.from(originalPdfData),
        annotationsJson: JSON.stringify(currentAnnotations),
      });

      message.success("PDF saved successfully!");
    } catch (err) {
      console.error("Failed to save PDF:", err);
      message.error(`Failed to save PDF: ${err}`);
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div
        style={{
          height: "100vh",
          width: "100vw",
          display: "flex",
          flexDirection: "column",
          justifyContent: "center",
          alignItems: "center",
        }}
      >
        <Spin size="large" />
        <div style={{ marginTop: 16 }}>Loading paper...</div>
      </div>
    );
  }

  if (!paperInfo) {
    return (
      <div
        style={{
          height: "100vh",
          width: "100vw",
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
        }}
      >
        <Empty description={error || "Paper not found"} />
      </div>
    );
  }

  return (
    <Layout style={{ height: "100%", flexDirection: "row" }}>
      {/* Left Sidebar - Document Details */}
      <Sider
        width={400}
        style={{
          background: "var(--ant-color-bg-container, #ffffff)",
          borderRight: "1px solid var(--ant-color-border, #d9d9d9)",
          overflow: "auto",
        }}
        className="pdf-reader-sidebar"
      >
        <div style={{ padding: 16 }}>
          <div style={{ marginBottom: 16 }}>
            <Button
              icon={<ArrowLeftOutlined />}
              onClick={handleClose}
              size="small"
            >
              Close
            </Button>
          </div>
          <DocumentDetails
            document={{
              id,
              title: paperInfo.title,
              authors: [],
              year: new Date().getFullYear(),
            }}
          />
        </div>
      </Sider>

      {/* Main Content - PDF Viewer */}
      <Content
        style={{
          flex: 1,
          background: "#525659",
          display: "flex",
          flexDirection: "column",
          overflow: "hidden",
        }}
      >
        {/* PDF Toolbar */}
        <div
          style={{
            padding: "8px 16px",
            background: "var(--ant-color-bg-elevated, #ffffff)",
            borderBottom: "1px solid var(--ant-color-border, #d9d9d9)",
            display: "flex",
            justifyContent: "flex-end",
            gap: 8,
          }}
        >
          {pdfData && (
            <>
              <button
                onClick={() => handleSavePDF({ getAnnotations: () => null })}
                disabled={saving}
                style={{
                  padding: "6px 12px",
                  cursor: saving ? "not-allowed" : "pointer",
                  border: "1px solid #d9d9d9",
                  borderRadius: 4,
                  background: saving ? "#f5f5f5" : "#fff",
                  color: saving ? "#bfbfbf" : "#000",
                  opacity: saving ? 0.6 : 1,
                }}
              >
                {saving ? "Saving..." : "Save"}
              </button>
              <button
                onClick={handleExport}
                disabled={exporting}
                style={{
                  padding: "6px 12px",
                  cursor: exporting ? "not-allowed" : "pointer",
                  border: "1px solid #1976d2",
                  borderRadius: 4,
                  background: exporting ? "#f0f7ff" : "#1976d2",
                  color: exporting ? "#9cb8e1" : "#fff",
                }}
              >
                {exporting ? "Exporting..." : "Export"}
              </button>
            </>
          )}
        </div>

        {/* PDF Viewer */}
        <div style={{ flex: 1, overflow: "auto" }}>
          {error ? (
            <div
              style={{
                height: "100%",
                display: "flex",
                flexDirection: "column",
                justifyContent: "center",
                alignItems: "center",
                gap: 16,
                color: "#fff",
              }}
            >
              <div style={{ color: "#ff4d4f", fontSize: 16 }}>{error}</div>
              <Empty description="No PDF attached to this paper" />
            </div>
          ) : pdfData ? (
            <PdfAnnotator
              title={paperInfo.title}
              data={pdfData}
              user={{ id: "user-1", name: "User" }}
              onSave={handleSave}
              actions={() => <></>}
              layoutStyle={{ width: "100%", height: "100%" }}
            />
          ) : (
            <Empty description="No PDF available" />
          )}
        </div>
      </Content>
    </Layout>
  );
}
