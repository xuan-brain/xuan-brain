import * as React from "react";
import { useEffect, useState, useRef } from "react";
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
  const [filePath, setFilePath] = useState<string>("");
  const [exporting, setExporting] = useState(false);
  const [saving, setSaving] = useState(false);
  const originalPdfDataRef = useRef<Uint8Array | null>(null);

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
        const pdfDataCopy = new Uint8Array(uint8Array);
        setPdfData(uint8Array);
        setPaperTitle(info.paper_title);
        setFilePath(info.file_path);
        await currentWindow.setTitle(info.paper_title);

        // Save original PDF data to ref for later use
        originalPdfDataRef.current = pdfDataCopy;
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

  const handleSave = async (annotations: unknown) => {
    // Save annotations to backend (no verbose logging for onSave callback)
    try {
      await invokeCommand("save_annotations_data", {
        filePath: filePath,
        annotationsJson: JSON.stringify(annotations),
      });
    } catch (err) {
      console.error("Failed to save annotations data:", err);
    }
  };

  const handleExport = async () => {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const path = await save({
      defaultPath: `${paperTitle.replace(/[^a-zA-Z0-9\u4e00-\u9fa5]/g, "_")}_annotated.pdf`,
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
        await invokeCommand("export_pdf_with_annotations", {
          sourceFilePath: filePath,
          exportFilePath: path,
          pdfData: Array.from(pdfData!),
        });
      } catch (err) {
        console.error("Failed to export PDF:", err);
        alert(`Failed to export PDF: ${err}`);
      } finally {
        setExporting(false);
      }
    }
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
      actions={({ getAnnotations }) => (
        <>
          <button
            onClick={async () => {
              setSaving(true);
              try {
                const currentAnnotations = getAnnotations();

                await invokeCommand("save_pdf_with_annotations_data", {
                  filePath: filePath,
                  pdfData: Array.from(
                    originalPdfDataRef.current || new Uint8Array(),
                  ),
                  annotationsJson: JSON.stringify(currentAnnotations),
                });

                alert("PDF saved successfully!");
              } catch (err) {
                console.error("Failed to save PDF:", err);
                alert(`Failed to save PDF: ${err}`);
              } finally {
                setSaving(false);
              }
            }}
            disabled={saving}
            style={{
              padding: "6px 12px",
              marginRight: 8,
              cursor: saving ? "not-allowed" : "pointer",
              border: "1px solid #d9d9d9",
              borderRadius: "4px",
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
              borderRadius: "4px",
              background: exporting ? "#f0f7ff" : "#1976d2",
              color: exporting ? "#9cb8e1" : "#fff",
            }}
          >
            {exporting ? "Exporting..." : "Export"}
          </button>
        </>
      )}
      layoutStyle={{ width: "100vw", height: "100vh" }}
    />
  );
};

export default PDFViewerPage;
