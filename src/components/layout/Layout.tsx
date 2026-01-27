import { useState, useEffect, useCallback } from "react";
import { Layout } from "antd";
import Navigation from "../navigation/Navigation";
import StatusBar from "./StatusBar";
import DocumentList from "../document/DocumentList";
import DocumentDetails from "../document/DocumentDetails";
import { useAppStore } from "../../stores/useAppStore";
import "./Layout.css";

const { Sider, Content, Footer } = Layout;

interface MainLayoutProps {
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

export default function MainLayout({ children }: MainLayoutProps) {
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
    <Layout className="main-layout">
      {/* Main Content Area with Sidebars */}
      <Layout className="main-content-area">
        {/* Left Sidebar - Navigation */}
        <Sider width={`${leftWidth}%`} className="left-sidebar">
          <Navigation />
        </Sider>

        {/* Left Resizer */}
        <div
          className="resizer"
          onMouseDown={handleLeftResizerMouseDown}
          onMouseEnter={(e) => e.currentTarget.classList.add("resizer-hover")}
          onMouseLeave={(e) =>
            e.currentTarget.classList.remove("resizer-hover")
          }
        />

        {/* Main Content Area */}
        <Content style={{ width: `${middleWidth}%` }} className="main-content">
          {children || <DocumentList onDocumentSelect={setSelectedDocument} />}
        </Content>

        {/* Right Resizer */}
        <div
          className="resizer"
          onMouseDown={handleRightResizerMouseDown}
          onMouseEnter={(e) => e.currentTarget.classList.add("resizer-hover")}
          onMouseLeave={(e) =>
            e.currentTarget.classList.remove("resizer-hover")
          }
        />

        {/* Right Sidebar - Document Details */}
        <Sider width={`${rightWidth}%`} className="right-sidebar">
          <DocumentDetails document={selectedDocument} />
        </Sider>
      </Layout>

      {/* Bottom Status Bar */}
      <Footer className="status-bar">
        <StatusBar />
      </Footer>
    </Layout>
  );
}
