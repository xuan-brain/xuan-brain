import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Layout, Spin, Empty, Button } from "antd";
import { ArrowLeftOutlined } from "@ant-design/icons";
import { useTabsStore } from "@/stores/useTabsStore";
import DocumentDetails from "@/components/document/DocumentDetails";
import { PDFViewer } from "@embedpdf/react-pdf-viewer";

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

export default function PaperReaderPage() {
  const { paperId } = useParams<{ paperId: string }>();
  const navigate = useNavigate();
  const { addTab, removeTab, getTabByPaperId } = useTabsStore();

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>("");
  const [paperInfo, setPaperInfo] = useState<{ title: string } | null>(null);
  const [fileUrl, setFileUrl] = useState<string>("");

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
        const blob = new Blob([uint8Array], { type: "application/pdf" });
        const url = URL.createObjectURL(blob);

        setFileUrl(url);
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

    return () => {
      if (fileUrl) {
        URL.revokeObjectURL(fileUrl);
      }
    };
  }, [id]);

  const handleClose = () => {
    const tab = getTabByPaperId(id);
    if (tab) {
      removeTab(tab.id);
    }
    navigate("/papers");
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
          display: "flex",
          flexDirection: "column",
          overflow: "hidden",
        }}
      >
        {/* PDF Viewer */}
        <div style={{ flex: 1, overflow: "auto", height: "100%" }}>
          {error ? (
            <div
              style={{
                height: "100%",
                display: "flex",
                flexDirection: "column",
                justifyContent: "center",
                alignItems: "center",
                gap: 16,
              }}
            >
              <div style={{ color: "#ff4d4f", fontSize: 16 }}>{error}</div>
              <Empty description="No PDF attached to this paper" />
            </div>
          ) : fileUrl ? (
            <PDFViewer
              config={{
                src: fileUrl,
                theme: { preference: "light" },
              }}
            />
          ) : (
            <Empty description="No PDF available" />
          )}
        </div>
      </Content>
    </Layout>
  );
}
