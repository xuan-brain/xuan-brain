import * as React from "react";
import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { PDFViewer } from "@embedpdf/react-pdf-viewer";

async function invokeCommand<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

const PDFViewerPage: React.FC = () => {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>("");
  const [fileUrl, setFileUrl] = useState<string>("");

  useEffect(() => {
    const initPDF = async () => {
      setLoading(true);
      const url = new URL(window.location.href);
      const filePathFromQuery = url.searchParams.get("path");
      const titleFromQuery = url.searchParams.get("title");

      if (!filePathFromQuery) {
        setError("No PDF path provided in URL");
        setLoading(false);
        return;
      }

      try {
        const contents = await invokeCommand<number[]>("read_pdf_file", {
          filePath: filePathFromQuery,
        });
        const uint8Array = new Uint8Array(contents);

        // Create object URL and store it in state
        const blob = new Blob([uint8Array], { type: "application/pdf" });
        const url = URL.createObjectURL(blob);
        setFileUrl(url);

        const currentWindow = getCurrentWindow();
        await currentWindow.setTitle(titleFromQuery || "PDF Viewer");
      } catch (err) {
        console.error("Failed to load PDF:", err);
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setLoading(false);
      }
    };

    initPDF();

    // Cleanup object URL when component unmounts
    return () => {
      if (fileUrl) {
        URL.revokeObjectURL(fileUrl);
      }
    };
  }, []);

  const handleClose = async () => {
    const currentWindow = getCurrentWindow();
    await currentWindow.close();
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

  if (error) {
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
        <div style={{ fontSize: 16, color: "#ff4d4f" }}>{error}</div>
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

  if (!fileUrl) {
    return null; // or some other placeholder
  }

  return (
    <div style={{ height: "100vh", width: "100vw" }}>
      <PDFViewer
        config={{
          src: fileUrl,
          theme: { preference: "light" },
        }}
      />
    </div>
  );
};

export default PDFViewerPage;
