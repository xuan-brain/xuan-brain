import { useState, useEffect, useCallback } from "react";
import Navigation from "../navigation/Navigation";
import StatusBar from "./StatusBar";
import DocumentList from "../document/DocumentList";
import DocumentDetails from "../document/DocumentDetails";
import { useAppStore } from "../../stores/useAppStore";

interface LayoutProps {
  children?: React.ReactNode;
}

const STORAGE_KEY = "xuan-brain-layout-widths";
const MIN_WIDTH_PERCENT = 10;
const MAX_WIDTH_PERCENT = 40;

function loadWidths(): { left: number; right: number } {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      const widths = JSON.parse(saved);
      return {
        left: Math.max(
          MIN_WIDTH_PERCENT,
          Math.min(MAX_WIDTH_PERCENT, widths.left),
        ),
        right: Math.max(
          MIN_WIDTH_PERCENT,
          Math.min(MAX_WIDTH_PERCENT, widths.right),
        ),
      };
    }
  } catch (e) {
    console.error("Failed to load layout widths:", e);
  }
  return { left: 15, right: 15 };
}

export default function Layout({ children }: LayoutProps) {
  const { selectedDocument, setSelectedDocument } = useAppStore();
  const savedWidths = loadWidths();
  const [leftWidth, setLeftWidth] = useState(savedWidths.left);
  const [rightWidth, setRightWidth] = useState(savedWidths.right);
  const middleWidth = 100 - leftWidth - rightWidth;

  const [isDraggingLeft, setIsDraggingLeft] = useState(false);
  const [isDraggingRight, setIsDraggingRight] = useState(false);
  const [startX, setStartX] = useState(0);
  const [startLeftWidth, setStartLeftWidth] = useState(0);
  const [startRightWidth, setStartRightWidth] = useState(0);

  // Save widths to localStorage
  useEffect(() => {
    try {
      localStorage.setItem(
        STORAGE_KEY,
        JSON.stringify({ left: leftWidth, right: rightWidth }),
      );
    } catch (e) {
      console.error("Failed to save layout widths:", e);
    }
  }, [leftWidth, rightWidth]);

  // Mouse move handler
  const handleMouseMove = useCallback(
    (event: MouseEvent) => {
      if (!isDraggingLeft && !isDraggingRight) return;

      const containerWidth = window.innerWidth;
      const deltaX = event.clientX - startX;
      const deltaPercent = (deltaX / containerWidth) * 100;

      if (isDraggingLeft) {
        const newWidth = startLeftWidth + deltaPercent;
        setLeftWidth(
          Math.max(MIN_WIDTH_PERCENT, Math.min(MAX_WIDTH_PERCENT, newWidth)),
        );
      }

      if (isDraggingRight) {
        const newWidth = startRightWidth - deltaPercent;
        setRightWidth(
          Math.max(MIN_WIDTH_PERCENT, Math.min(MAX_WIDTH_PERCENT, newWidth)),
        );
      }
    },
    [isDraggingLeft, isDraggingRight, startX, startLeftWidth, startRightWidth],
  );

  // Mouse up handler
  const handleMouseUp = useCallback(() => {
    setIsDraggingLeft(false);
    setIsDraggingRight(false);
  }, []);

  // Add/remove global event listeners
  useEffect(() => {
    if (isDraggingLeft || isDraggingRight) {
      document.addEventListener("mousemove", handleMouseMove);
      document.addEventListener("mouseup", handleMouseUp);
      document.body.style.cursor = "col-resize";
      document.body.style.userSelect = "none";

      return () => {
        document.removeEventListener("mousemove", handleMouseMove);
        document.removeEventListener("mouseup", handleMouseUp);
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
      };
    }
  }, [isDraggingLeft, isDraggingRight, handleMouseMove, handleMouseUp]);

  // Left resizer mouse down
  const handleLeftResizerMouseDown = (event: React.MouseEvent) => {
    event.preventDefault();
    setIsDraggingLeft(true);
    setStartX(event.clientX);
    setStartLeftWidth(leftWidth);
  };

  // Right resizer mouse down
  const handleRightResizerMouseDown = (event: React.MouseEvent) => {
    event.preventDefault();
    setIsDraggingRight(true);
    setStartX(event.clientX);
    setStartRightWidth(rightWidth);
  };

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100vh",
        overflow: "hidden",
      }}
    >
      <div
        style={{
          display: "flex",
          flex: 1,
          overflow: "hidden",
        }}
      >
        {/* Left Sidebar - Navigation */}
        <div
          style={{
            width: `${leftWidth}%`,
            minWidth: `${MIN_WIDTH_PERCENT}%`,
            maxWidth: `${MAX_WIDTH_PERCENT}%`,
            borderRight: "1px solid var(--ant-color-border)",
            overflow: "hidden",
            display: "flex",
            flexDirection: "column",
          }}
        >
          <Navigation />
        </div>

        {/* Left Resizer */}
        <div
          onMouseDown={handleLeftResizerMouseDown}
          style={{
            width: "2px",
            cursor: "col-resize",
            backgroundColor: "var(--ant-color-border)",
            zIndex: 10,
            flexShrink: 0,
          }}
          onMouseEnter={(e) => e.currentTarget.style.backgroundColor = "var(--ant-color-primary)"}
          onMouseLeave={(e) => e.currentTarget.style.backgroundColor = "var(--ant-color-border)"}
        />

        {/* Main Content Area */}
        <main
          style={{
            width: `${middleWidth}%`,
            flex: "1 1 auto",
            overflow: "auto",
            backgroundColor: "var(--ant-color-bg-layout)",
            minWidth: 0,
          }}
        >
          {children || <DocumentList onDocumentSelect={setSelectedDocument} />}
        </main>

        {/* Right Resizer */}
        <div
          onMouseDown={handleRightResizerMouseDown}
          style={{
            width: "2px",
            cursor: "col-resize",
            backgroundColor: "var(--ant-color-border)",
            zIndex: 10,
            flexShrink: 0,
          }}
          onMouseEnter={(e) => e.currentTarget.style.backgroundColor = "var(--ant-color-primary)"}
          onMouseLeave={(e) => e.currentTarget.style.backgroundColor = "var(--ant-color-border)"}
        />

        {/* Right Sidebar - Document Details */}
        <div
          style={{
            width: `${rightWidth}%`,
            minWidth: `${MIN_WIDTH_PERCENT}%`,
            maxWidth: `${MAX_WIDTH_PERCENT}%`,
            borderLeft: "1px solid var(--ant-color-border)",
            overflow: "auto",
            flexShrink: 0,
          }}
        >
          <DocumentDetails document={selectedDocument} />
        </div>
      </div>

      {/* Bottom Status Bar */}
      <StatusBar />
    </div>
  );
}
