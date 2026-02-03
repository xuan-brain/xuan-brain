import * as React from "react";
import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Viewer, Worker } from "@react-pdf-viewer/core";
import { defaultLayoutPlugin } from "@react-pdf-viewer/default-layout";
// Import styles
import "@react-pdf-viewer/core/lib/styles/index.css";
import "@react-pdf-viewer/default-layout/lib/styles/index.css";

// Configure worker
import * as pdfjs from "pdfjs-dist";
pdfjs.GlobalWorkerOptions.workerSrc = new URL(
  "pdfjs-dist/build/pdf.worker.min.mjs",
  import.meta.url,
).toString();

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
  const [pdfUrl, setPdfUrl] = useState<string>("");
  const [loading, setLoading] = useState(true);
  const [containerHeight, setContainerHeight] = useState<number>(
    window.innerHeight,
  );
  const [error, setError] = useState<string>("");

  // Update height on mount and resize
  useEffect(() => {
    const updateHeight = () => setContainerHeight(window.innerHeight);
    updateHeight();
    window.addEventListener("resize", updateHeight);
    return () => window.removeEventListener("resize", updateHeight);
  }, []);

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
        const blob = new Blob([uint8Array], { type: "application/pdf" });
        const url = URL.createObjectURL(blob);
        setPdfUrl(url);
        await currentWindow.setTitle(info.paper_title);
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setLoading(false);
      }
    };

    initPDF();
  }, []);

  // Initialize plugin BEFORE any conditional returns
  // This is required because defaultLayoutPlugin uses React Hooks internally
  const defaultLayoutPluginInstance = defaultLayoutPlugin();

  const handleClose = async () => {
    const currentWindow = getCurrentWindow();
    await currentWindow.close();
  };

  if (loading) {
    return (
      <div
        style={{
          height: `${containerHeight}px`,
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
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

  if (error || !pdfUrl) {
    return (
      <div
        style={{
          height: `${containerHeight}px`,
          display: "flex",
          flexDirection: "column",
          justifyContent: "center",
          alignItems: "center",
          gap: 16,
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
    <div
      style={{
        height: `${containerHeight}px`,
        width: "100vw",
        display: "flex",
        flexDirection: "column",
      }}
    >
      {/* Header */}
      <div
        style={{
          height: "40px",
          padding: "4px 16px",
          borderBottom: "1px solid #d9d9d9",
          background: "#fff",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          flexShrink: 0,
          fontSize: "13px",
        }}
      >
        <span
          style={{
            flex: 1,
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}
        >
          PDF Viewer - Full Featured
        </span>
        <button
          onClick={handleClose}
          style={{
            padding: "4px 12px",
            cursor: "pointer",
            border: "1px solid #d9d9d9",
            borderRadius: "4px",
            background: "#fff",
            fontSize: "12px",
          }}
        >
          Close
        </button>
      </div>

      {/* PDF Viewer with Default Layout Plugin */}
      <div style={{ flex: 1, overflow: "hidden" }}>
        <Worker
          workerUrl={new URL(
            "pdfjs-dist/build/pdf.worker.min.mjs",
            import.meta.url,
          ).toString()}
        >
          <Viewer fileUrl={pdfUrl} plugins={[defaultLayoutPluginInstance]} />
        </Worker>
      </div>
    </div>
  );
};

export default PDFViewerPage;
