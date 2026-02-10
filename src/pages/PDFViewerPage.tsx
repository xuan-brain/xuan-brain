import * as React from "react";
import { useEffect, useState, useRef, useMemo } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Viewer, Worker } from "@react-pdf-viewer/core";
import { defaultLayoutPlugin } from "@react-pdf-viewer/default-layout";
import {
  highlightPlugin,
  HighlightArea,
  MessageIcon,
  Trigger,
} from "@react-pdf-viewer/highlight";
import { Button, Position, Tooltip } from "@react-pdf-viewer/core";
import "@react-pdf-viewer/core/lib/styles/index.css";
import "@react-pdf-viewer/default-layout/lib/styles/index.css";
import "@react-pdf-viewer/highlight/lib/styles/index.css";

async function invokeCommand<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

// Annotation types
interface Note {
  id: string;
  content: string;
  highlightAreas: HighlightArea[];
  quote: string;
  timestamp: number;
}

const PDFViewerPage: React.FC = () => {
  const [pdfData, setPdfData] = useState<Uint8Array | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>("");
  const [paperTitle, setPaperTitle] = useState<string>("");
  const [filePath, setFilePath] = useState<string>("");
  const [totalPages, setTotalPages] = useState<number>(0);
  const [fileUrl, setFileUrl] = useState<string>("");
  const [notes, setNotes] = useState<Note[]>([]);
  const [currentMessage, setCurrentMessage] = useState<string>("");
  const noteElesRef = useRef<Map<string, HTMLElement>>(new Map());
  const viewerRef = useRef<any>(null);
  const jumpToHighlightAreaRef = useRef<((area: HighlightArea) => void) | null>(
    null,
  );
  const notesRef = useRef(notes);
  const currentMessageRef = useRef(currentMessage);

  // Keep refs in sync with state
  useEffect(() => {
    notesRef.current = notes;
  }, [notes]);

  useEffect(() => {
    currentMessageRef.current = currentMessage;
  }, [currentMessage]);

  // Render highlight target (the button shown after selecting text)
  const renderHighlightTarget = useMemo(() => {
    return (props: any) => {
      return (
        <div
          style={{
            background: "#eee",
            display: "flex",
            position: "absolute",
            left: `${props.selectionRegion.left}%`,
            top: `${props.selectionRegion.top + props.selectionRegion.height}%`,
            transform: "translate(0, 8px)",
            zIndex: 1,
          }}
        >
          <Tooltip
            position={Position.TopCenter}
            target={
              <Button onClick={props.toggle}>
                <MessageIcon />
              </Button>
            }
            content={() => <div style={{ width: "100px" }}>Add note</div>}
            offset={{ left: 0, top: -8 }}
          />
        </div>
      );
    };
  }, []);

  // Render highlight content (the form shown after clicking the button)
  const renderHighlightContent = useMemo(() => {
    return (props: any) => {
      const addNote = () => {
        const msg = currentMessageRef.current;
        if (msg.trim() !== "") {
          const note: Note = {
            id: Date.now().toString(),
            content: msg,
            highlightAreas: props.highlightAreas,
            quote: props.selectedText,
            timestamp: Date.now(),
          };
          setNotes([...notesRef.current, note]);
          setCurrentMessage("");
          props.cancel();
          console.info("Note added:", note);
        }
      };

      const handleMessageChange = (
        e: React.ChangeEvent<HTMLTextAreaElement>,
      ) => {
        setCurrentMessage(e.target.value);
      };

      return (
        <div
          style={{
            background: "#fff",
            border: "1px solid rgba(0, 0, 0, 0.3)",
            borderRadius: "4px",
            padding: "8px",
            position: "absolute",
            left: `${props.selectionRegion.left}%`,
            top: `${props.selectionRegion.top + props.selectionRegion.height}%`,
            zIndex: 1,
            minWidth: "200px",
          }}
        >
          <div>
            <textarea
              rows={3}
              style={{
                border: "1px solid rgba(0, 0, 0, 0.3)",
                borderRadius: "4px",
                padding: "4px",
                width: "100%",
                fontFamily: "inherit",
                fontSize: "14px",
                resize: "vertical",
              }}
              placeholder="Enter your note..."
              onChange={handleMessageChange}
              autoFocus
            />
          </div>
          <div
            style={{
              display: "flex",
              marginTop: "8px",
              gap: "8px",
            }}
          >
            <button
              onClick={addNote}
              style={{
                padding: "4px 12px",
                cursor: "pointer",
                border: "1px solid #1976d2",
                borderRadius: "4px",
                background: "#1976d2",
                color: "#fff",
              }}
            >
              Add
            </button>
            <button
              onClick={props.cancel}
              style={{
                padding: "4px 12px",
                cursor: "pointer",
                border: "1px solid #d9d9d9",
                borderRadius: "4px",
                background: "#fff",
                color: "#000",
              }}
            >
              Cancel
            </button>
          </div>
        </div>
      );
    };
  }, []);

  // Render highlights on the PDF
  const renderHighlights = useMemo(() => {
    return (props: any) => (
      <div>
        {notesRef.current.map((note) => (
          <React.Fragment key={note.id}>
            {note.highlightAreas
              .filter((area) => area.pageIndex === props.pageIndex)
              .map((area: HighlightArea, idx: number) => (
                <div
                  key={idx}
                  style={Object.assign(
                    {},
                    {
                      background: "yellow",
                      opacity: 0.4,
                      cursor: "pointer",
                    },
                    props.getCssProperties(area, props.rotation),
                  )}
                  onClick={() => jumpToNote(note.id)}
                  ref={(ref): void => {
                    if (ref) {
                      noteElesRef.current.set(note.id, ref as HTMLElement);
                    }
                  }}
                />
              ))}
          </React.Fragment>
        ))}
      </div>
    );
  }, []);

  // Jump to a specific note in the sidebar
  const jumpToNote = (noteId: string) => {
    if (noteElesRef.current.has(noteId)) {
      const element = noteElesRef.current.get(noteId);
      if (element) {
        element.scrollIntoView({ behavior: "smooth", block: "center" });
        element.style.background = "orange";
        setTimeout(() => {
          element.style.background = "yellow";
        }, 1000);
      }
    }
  };

  // Initialize highlight plugin instance
  const highlightPluginInstance = highlightPlugin({
    renderHighlightTarget,
    renderHighlightContent,
    renderHighlights,
    trigger: Trigger.TextSelection,
  });

  // Get jumpToHighlightArea function from highlight plugin
  const { jumpToHighlightArea } = highlightPluginInstance;

  useEffect(() => {
    jumpToHighlightAreaRef.current = jumpToHighlightArea;
  }, [jumpToHighlightArea]);

  // Sidebar notes component
  const sidebarNotes = useMemo(() => {
    return (
      <>
        {notesRef.current.length === 0 && (
          <div
            style={{
              padding: "16px",
              textAlign: "center",
              color: "#666",
            }}
          >
            No notes yet. Select text to add notes.
          </div>
        )}
        {notesRef.current.map((note) => (
          <div
            key={note.id}
            style={{
              padding: "12px",
              borderBottom: "1px solid #f0f0f0",
              cursor: "pointer",
            }}
            onClick={() => {
              if (
                note.highlightAreas.length > 0 &&
                jumpToHighlightAreaRef.current
              ) {
                jumpToHighlightAreaRef.current(note.highlightAreas[0]);
              }
            }}
          >
            <div
              style={{
                fontSize: "12px",
                color: "#666",
                marginBottom: "4px",
              }}
            >
              {new Date(note.timestamp).toLocaleString()}
            </div>
            <div
              style={{
                fontSize: "13px",
                fontWeight: 500,
                marginBottom: "4px",
                background: "#f5f5f5",
                padding: "8px",
                borderRadius: "4px",
              }}
            >
              {note.quote}
            </div>
            <div
              style={{
                fontSize: "14px",
                color: "#333",
              }}
            >
              {note.content}
            </div>
            <button
              onClick={(e) => {
                e.stopPropagation();
                setNotes(notesRef.current.filter((n) => n.id !== note.id));
                console.info("Note deleted:", note.id);
              }}
              style={{
                marginTop: "8px",
                padding: "2px 8px",
                fontSize: "12px",
                cursor: "pointer",
                border: "1px solid #ff4d4f",
                borderRadius: "4px",
                background: "#fff",
                color: "#ff4d4f",
              }}
            >
              Delete
            </button>
          </div>
        ))}
      </>
    );
  }, [notes]);

  // Initialize the defaultLayoutPlugin at the top level to avoid hook order issues
  // with a sidebar tab for annotations
  const defaultLayoutPluginInstance = defaultLayoutPlugin({
    sidebarTabs: (defaultTabs) =>
      defaultTabs.concat({
        content: sidebarNotes,
        icon: <MessageIcon />,
        title: "Annotations",
      }),
  });

  // Cleanup object URL when component unmounts or pdfData changes
  useEffect(() => {
    return () => {
      if (fileUrl) {
        URL.revokeObjectURL(fileUrl);
      }
    };
  }, [fileUrl]);

  const handleClose = async () => {
    const currentWindow = getCurrentWindow();
    await currentWindow.close();
  };

  const handleExport = async () => {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const path = await save({
      defaultPath: `${paperTitle.replace(/[^a-zA-Z0-9\u4e00-\u9fa5]/g, "_")}_annotated.pdf`,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });

    if (path && typeof path === "string") {
      try {
        await invokeCommand("export_pdf_with_annotations", {
          sourceFilePath: filePath,
          exportFilePath: path,
          pdfData: Array.from(pdfData!),
        });
      } catch (err) {
        console.error("Failed to export PDF:", err);
        alert(`Failed to export PDF: ${err}`);
      }
    }
  };

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
        setPaperTitle(titleFromQuery || "PDF Viewer");
        setFilePath(filePathFromQuery);

        const contents = await invokeCommand<number[]>("read_pdf_file", {
          filePath: filePathFromQuery,
        });
        const uint8Array = new Uint8Array(contents);
        setPdfData(uint8Array);

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
  }, []);

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
    <div
      style={{
        height: "100vh",
        width: "100vw",
        display: "flex",
        flexDirection: "column",
        backgroundColor: "#f5f5f5",
      }}
    >
      <div
        style={{
          padding: "10px 16px",
          borderBottom: "1px solid #d9d9d9",
          display: "flex",
          alignItems: "center",
          gap: "12px",
          backgroundColor: "#fff",
        }}
      >
        <div style={{ flex: 1, fontSize: 14, color: "#666" }}>
          <span>File: {paperTitle}</span>
          <span style={{ marginLeft: "12px" }}>Total pages: {totalPages}</span>
          <span style={{ marginLeft: "12px" }}>
            Annotations: {notes.length}
          </span>
        </div>
        <div style={{ padding: "10px 16px", display: "flex", gap: "8px" }}>
          <button
            onClick={handleClose}
            style={{
              padding: "6px 12px",
              cursor: "pointer",
              border: "1px solid #ff4d4f",
              borderRadius: "4px",
              background: "#fff",
              color: "#ff4d4f",
            }}
          >
            Close
          </button>
          <button
            onClick={handleExport}
            style={{
              padding: "6px 12px",
              cursor: "pointer",
              border: "1px solid #1976d2",
              borderRadius: "4px",
              background: "#1976d2",
              color: "#fff",
            }}
          >
            Export PDF
          </button>
        </div>
      </div>
      <div style={{ flex: 1, overflow: "hidden" }}>
        <Worker workerUrl="https://unpkg.com/pdfjs-dist@3.4.120/build/pdf.worker.min.js">
          <Viewer
            ref={viewerRef}
            fileUrl={fileUrl}
            plugins={[defaultLayoutPluginInstance, highlightPluginInstance]}
            onDocumentLoad={(e) => {
              const doc = e.doc;
              setTotalPages(doc.numPages);
              console.info("PDF loaded with", doc.numPages, "pages");
            }}
          />
        </Worker>
      </div>
    </div>
  );
};

export default PDFViewerPage;
