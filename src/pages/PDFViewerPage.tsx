import * as React from "react";
import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { PdfAnnotator } from "pdfjs-annotation-extension-for-react";
import "pdfjs-annotation-extension-for-react/style";

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

const PDFViewerPage: React.FC = () => {
  const [pdfData, setPdfData] = useState<Uint8Array | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>("");
  const [paperTitle, setPaperTitle] = useState<string>("");

  useEffect(() => {
    const initPDF = async () => {
      const currentWindow = getCurrentWindow();
      const label = currentWindow.label;
      const idMatch = label.match(/pdf-viewer-(\d+)/);

      if (!idMatch) {
        setError("Invalid PDF viewer window");
        setLoading(false);
        return;
      }

      try {
        const id = parseInt(idMatch[1], 10);
        const info = await invokeCommand<PdfAttachmentInfo>(
          "get_pdf_attachment_path",
          { paperId: id },
        );
        const contents = await invokeCommand<number[]>("read_pdf_file", {
          filePath: info.file_path,
        });
        const uint8Array = new Uint8Array(contents);
        setPdfData(uint8Array);
        setPaperTitle(info.paper_title);
        await currentWindow.setTitle(info.paper_title);
        console.info("PDF loaded successfully:", info.paper_title);
      } catch (err) {
        console.error("Failed to load PDF:", err);
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setLoading(false);
      }
    };

    initPDF();
  }, []);

  const handleClose = async () => {
    const currentWindow = getCurrentWindow();
    await currentWindow.close();
  };

  const handleSave = (annotations: unknown) => {
    console.info("Annotations saved:", annotations);
  };

  if (loading) {
    return (
      <div
        style={{
          height: "100vh",
          width: "100vw",
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
          backgroundColor: "#fff",
        }}
      >
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 18, marginBottom: 8 }}>Loading PDF...</div>
          <div style={{ fontSize: 14, color: "#666" }}>
            Please wait while we prepare your document
          </div>
        </div>
      </div>
    );
  }

  if (error || !pdfData) {
    return (
      <div
        style={{
          height: "100vh",
          width: "100vw",
          display: "flex",
          flexDirection: "column",
          justifyContent: "center",
          alignItems: "center",
          gap: 16,
          backgroundColor: "#fff",
        }}
      >
        <div style={{ fontSize: 16, color: "#ff4d4f" }}>
          {error || "No PDF loaded"}
        </div>
        <button
          onClick={handleClose}
          style={{
            padding: "8px 16px",
            cursor: "pointer",
            border: "1px solid #d9d9d9",
            borderRadius: "4px",
            background: "#fff",
          }}
        >
          Close
        </button>
      </div>
    );
  }

  return (
    <PdfAnnotator
      title={paperTitle}
      data={pdfData}
      user={{ id: "user-1", name: "User" }}
      onSave={handleSave}
      layoutStyle={{ width: "100vw", height: "100vh" }}
    />
  );
};

export default PDFViewerPage;
