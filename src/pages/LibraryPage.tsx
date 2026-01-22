import { Box, Typography, Paper, Button } from "@mui/material";
import { Add, Search } from "@mui/icons-material";
import { useI18n } from "../lib/i18n";

export default function LibraryPage() {
  const { t } = useI18n();

  return (
    <Box sx={{ p: 3 }}>
      {/* 页面标题和操作栏 */}
      <Box
        sx={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          mb: 3,
        }}
      >
        <Typography variant="h5" component="h1">
          {t("main.title")}
        </Typography>
        <Button variant="contained" startIcon={<Add />}>
          {t("main.importDocuments")}
        </Button>
      </Box>

      {/* 搜索栏 */}
      <Paper sx={{ p: 2, mb: 3 }}>
        <Box sx={{ display: "flex", gap: 2, alignItems: "center" }}>
          <Box sx={{ flex: 1 }}>
            <input
              type="text"
              placeholder={t("main.search")}
              style={{
                width: "100%",
                padding: "8px 12px",
                border: "1px solid #ccc",
                borderRadius: "4px",
                fontSize: "14px",
              }}
            />
          </Box>
          <Button variant="outlined" startIcon={<Search />}>
            {t("main.search")}
          </Button>
        </Box>
      </Paper>

      {/* 文献列表占位 */}
      <Paper sx={{ p: 4, textAlign: "center" }}>
        <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
          {t("main.noDocuments")}
        </Typography>
        <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
          {t("main.startUsing")}
        </Typography>
        <Button variant="outlined" startIcon={<Add />}>
          {t("main.importFirst")}
        </Button>
      </Paper>

      {/* 统计信息 */}
      <Box sx={{ mt: 3, display: "flex", gap: 3 }}>
        <Paper sx={{ p: 2, flex: 1, textAlign: "center" }}>
          <Typography variant="h6" color="primary">
            0
          </Typography>
          <Typography variant="caption" color="text.secondary">
            {t("statistics.totalDocuments")}
          </Typography>
        </Paper>
        <Paper sx={{ p: 2, flex: 1, textAlign: "center" }}>
          <Typography variant="h6" color="secondary">
            0
          </Typography>
          <Typography variant="caption" color="text.secondary">
            {t("statistics.totalCategories")}
          </Typography>
        </Paper>
        <Paper sx={{ p: 2, flex: 1, textAlign: "center" }}>
          <Typography variant="h6" color="info">
            0
          </Typography>
          <Typography variant="caption" color="text.secondary">
            {t("statistics.totalTags")}
          </Typography>
        </Paper>
      </Box>
    </Box>
  );
}
