import { useState, useEffect } from "react";
import { Button, Tag, Space, Typography } from "antd";
import {
  SyncOutlined,
  FileTextOutlined,
  SearchOutlined,
  ApiOutlined,
} from "@ant-design/icons";
import { useI18n } from "../../lib/i18n";
import ThemeSwitcher from "../navigation/ThemeSwitcher";
import LlmSwitcher from "./LlmSwitcher";
import GrobidSwitcher from "./GrobidSwitcher";

export default function StatusBar() {
  const { t } = useI18n();
  const [currentTime, setCurrentTime] = useState(new Date());
  const [syncStatus, setSyncStatus] = useState<
    "synced" | "syncing" | "unsynced"
  >("synced");
  const [isSyncing, setIsSyncing] = useState(false);
  const [memoryUsage, setMemoryUsage] = useState("0 MB");
  const documentCount = 0; // TODO: Load from backend

  // Update time and memory usage every second
  useEffect(() => {
    const intervalId = setInterval(() => {
      setCurrentTime(new Date());
      // Simulate memory usage (in actual projects, use performance.memory or Tauri API)
      const memory = Math.floor(Math.random() * 100 + 50);
      setMemoryUsage(`${memory} MB`);
    }, 1000);

    return () => clearInterval(intervalId);
  }, []);

  // Handle sync button click
  const handleSync = () => {
    setIsSyncing(true);
    setSyncStatus("syncing");
    setTimeout(() => {
      setIsSyncing(false);
      setSyncStatus("synced");
    }, 2000);
  };

  return (
    <div
      style={{
        height: 32,
        borderTop: "1px solid var(--ant-color-border)",
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "0 12px",
        userSelect: "none",
        backgroundColor: "var(--ant-color-bg-container)",
      }}
    >
      {/* Left section */}
      <Space size="middle">
        {/* Sync status */}
        <Space size="small">
          <Button
            type="text"
            size="small"
            icon={<SyncOutlined spin={isSyncing} />}
            onClick={handleSync}
            disabled={isSyncing}
            style={{
              padding: "0 4px",
              height: 24,
            }}
          />
          <Tag
            color={
              syncStatus === "synced"
                ? "success"
                : syncStatus === "syncing"
                  ? "warning"
                  : "default"
            }
            style={{ margin: 0, height: 24, fontSize: 12 }}
          >
            {t(`status.${syncStatus}`)}
          </Tag>
        </Space>

        {/* Document count */}
        <Space size="small" style={{ fontSize: 12 }}>
          <FileTextOutlined />
          <Typography.Text style={{ fontSize: 12 }}>
            {t("status.documents")}: {documentCount}
          </Typography.Text>
        </Space>

        {/* Search status */}
        <Space size="small" style={{ fontSize: 12 }} className="hidden-md-down">
          <SearchOutlined />
          <Typography.Text style={{ fontSize: 12 }}>
            {t("status.searchStatus")}
          </Typography.Text>
        </Space>

        {/* Memory usage */}
        <Space size="small" style={{ fontSize: 12 }} className="hidden-lg-down">
          <ApiOutlined />
          <Typography.Text style={{ fontSize: 12 }}>
            {memoryUsage}
          </Typography.Text>
        </Space>
      </Space>

      {/* Right section */}
      <Space size="middle">
        {/* LLM Switcher */}
        <LlmSwitcher />

        {/* GROBID Switcher */}
        <GrobidSwitcher />

        {/* Theme Switcher (Dark Mode, Language, Theme, Accent Color) */}
        <ThemeSwitcher />

        {/* Version */}
        <Typography.Text style={{ fontSize: 12 }} className="hidden-sm-down">
          {t("status.version")} 0.1.0
        </Typography.Text>

        {/* Clock */}
        <div
          style={{
            fontFamily: "monospace",
            backgroundColor: "var(--ant-color-bg-layout)",
            padding: "2px 8px",
            borderRadius: 4,
          }}
        >
          <Typography.Text style={{ fontSize: 12 }}>
            {currentTime.toLocaleTimeString()}
          </Typography.Text>
        </div>
      </Space>
    </div>
  );
}
