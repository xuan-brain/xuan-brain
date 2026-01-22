import { useState, useEffect } from "react";
import { Box, Typography, IconButton, Chip } from "@mui/material";
import { Sync, Description, Search, Memory } from "@mui/icons-material";
import { useI18n } from "../lib/i18n";
import ThemeSwitcher from "./ThemeSwitcher";

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
    <Box
      sx={{
        height: 32,
        bgcolor: "background.paper",
        borderTop: 1,
        borderColor: "divider",
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        px: 2,
        userSelect: "none",
      }}
    >
      {/* Left section */}
      <Box sx={{ display: "flex", alignItems: "center", gap: 2 }}>
        {/* Sync status */}
        <Box sx={{ display: "flex", alignItems: "center", gap: 0.5 }}>
          <IconButton
            size="small"
            onClick={handleSync}
            disabled={isSyncing}
            sx={{
              p: 0.5,
              color: "text.secondary",
              "&:hover": { color: "text.primary" },
              animation: isSyncing ? "pulse 1s infinite" : "none",
              "@keyframes pulse": {
                "0%": { opacity: 1 },
                "50%": { opacity: 0.5 },
                "100%": { opacity: 1 },
              },
            }}
          >
            <Sync fontSize="small" />
          </IconButton>
          <Chip
            size="small"
            label={t(`status.${syncStatus}`)}
            color={
              syncStatus === "synced"
                ? "success"
                : syncStatus === "syncing"
                  ? "warning"
                  : "default"
            }
            sx={{ height: 20, fontSize: "0.7rem" }}
          />
        </Box>

        {/* Document count */}
        <Box sx={{ display: "flex", alignItems: "center", gap: 0.5 }}>
          <Description fontSize="small" sx={{ fontSize: 14 }} />
          <Typography variant="caption" sx={{ fontSize: "0.7rem" }}>
            {t("status.documents")}: {documentCount}
          </Typography>
        </Box>

        {/* Search status */}
        <Box
          sx={{
            display: { xs: "none", md: "flex" },
            alignItems: "center",
            gap: 0.5,
          }}
        >
          <Search fontSize="small" sx={{ fontSize: 14 }} />
          <Typography variant="caption" sx={{ fontSize: "0.7rem" }}>
            {t("status.searchStatus")}
          </Typography>
        </Box>

        {/* Memory usage */}
        <Box
          sx={{
            display: { xs: "none", lg: "flex" },
            alignItems: "center",
            gap: 0.5,
          }}
        >
          <Memory fontSize="small" sx={{ fontSize: 14 }} />
          <Typography variant="caption" sx={{ fontSize: "0.7rem" }}>
            {memoryUsage}
          </Typography>
        </Box>
      </Box>

      {/* Right section */}
      <Box sx={{ display: "flex", alignItems: "center", gap: 2 }}>
        {/* Theme Switcher (Dark Mode, Language, Theme, Accent Color) */}
        <ThemeSwitcher />

        {/* Version */}
        <Typography
          variant="caption"
          sx={{ display: { xs: "none", sm: "block" }, fontSize: "0.7rem" }}
        >
          {t("status.version")} 0.1.0
        </Typography>

        {/* Clock */}
        <Box
          sx={{
            fontFamily: "monospace",
            bgcolor: "action.disabledBackground",
            px: 1,
            py: 0.5,
            borderRadius: 0.5,
          }}
        >
          <Typography variant="caption" sx={{ fontSize: "0.65rem" }}>
            {currentTime.toLocaleTimeString()}
          </Typography>
        </Box>
      </Box>
    </Box>
  );
}
