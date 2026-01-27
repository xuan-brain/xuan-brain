import { useState, useEffect } from "react";
import { DataGrid, GridColDef } from "@mui/x-data-grid";
import { Box, Chip } from "@mui/material";
import { useI18n } from "../../lib/i18n";
import DocumentToolbar from "./DocumentToolbar";

// Lazy load invoke helper - works in both Tauri and browser
async function invokeCommand<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

// Predefined color palette for tags
const TAG_COLORS: Record<string, string> = {
  red: "#ef4444",
  orange: "#f97316",
  amber: "#f59e0b",
  yellow: "#eab308",
  lime: "#84cc16",
  green: "#22c55e",
  emerald: "#10b981",
  teal: "#14b8a6",
  cyan: "#06b6d4",
  sky: "#0ea5e9",
  blue: "#3b82f6",
  indigo: "#6366f1",
  violet: "#8b5cf6",
  purple: "#a855f7",
  fuchsia: "#d946ef",
  pink: "#ec4899",
  rose: "#f43f5e",
};

interface LabelDto {
  id: number;
  name: string;
  color: string;
}

interface PaperDto {
  id: number;
  title: string;
  publication_year?: number;
  journal_name?: string;
  conference_name?: string;
  authors: string[];
  labels: LabelDto[];
}

interface DocumentListProps {
  onDocumentSelect: (document: any) => void;
}

export default function DocumentList({ onDocumentSelect }: DocumentListProps) {
  const { t } = useI18n();
  const [rows, setRows] = useState<PaperDto[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadPapers();
  }, []);

  const loadPapers = async () => {
    setLoading(true);
    try {
      const papers = await invokeCommand<PaperDto[]>("get_all_papers");
      console.info("Loaded papers:", papers.length);
      setRows(papers);
      if (papers.length > 0) {
        onDocumentSelect(papers[0]);
      }
    } catch (error) {
      console.error("Failed to load papers:", error);
      // Demo data if backend fails (dev mode without tauri)
      const demoData = [
        {
          id: 1,
          title: "Attention Is All You Need",
          authors: ["Vaswani et al."],
          publication_year: 2017,
          conference_name: "NIPS",
          labels: [],
        },
      ];
      setRows(demoData);
      if (demoData.length > 0) {
        onDocumentSelect(demoData[0]);
      }
    } finally {
      setLoading(false);
    }
  };

  const columns: GridColDef[] = [
    { field: "title", headerName: t("document.title"), flex: 2, minWidth: 200 },
    {
      field: "authors",
      headerName: t("document.authors"),
      flex: 1,
      minWidth: 150,
      renderCell: (params) => {
        return (
          <Box
            sx={{
              display: "flex",
              gap: 0.5,
              flexWrap: "wrap",
              alignItems: "center",
              height: "100%",
            }}
          >
            {params.row.authors?.map((author: string, index: number) => (
              <Chip
                key={index}
                label={author}
                size="small"
                sx={{ fontSize: "0.875rem" }}
              />
            ))}
          </Box>
        );
      },
    },
    {
      field: "source",
      headerName: t("document.source"),
      flex: 1,
      minWidth: 150,
      valueGetter: (_value: unknown, row: PaperDto) =>
        row.journal_name || row.conference_name || "",
    },
    { field: "publication_year", headerName: t("document.year"), width: 90 },
    {
      field: "labels",
      headerName: t("document.labels"),
      flex: 1,
      minWidth: 150,
      renderCell: (params) => {
        return (
          <Box
            sx={{
              display: "flex",
              gap: 0.5,
              flexWrap: "wrap",
              alignItems: "center",
              height: "100%",
            }}
          >
            {params.row.labels?.map((label: LabelDto) => (
              <Chip
                key={label.id}
                label={label.name}
                size="small"
                sx={{
                  fontSize: "0.75rem",
                  maxWidth: "100px",
                  backgroundColor: TAG_COLORS[label.color] || TAG_COLORS.blue,
                  color: "#fff",
                  "& .MuiChip-label": {
                    paddingLeft: "4px",
                    paddingRight: "4px",
                  },
                }}
              />
            ))}
          </Box>
        );
      },
    },
  ];

  return (
    <Box
      sx={{
        height: "100%",
        width: "100%",
        display: "flex",
        flexDirection: "column",
      }}
    >
      {/* Toolbar */}
      <DocumentToolbar onRefresh={loadPapers} />

      {/* DataGrid */}
      <DataGrid
        rows={rows}
        columns={columns}
        loading={loading}
        density="compact"
        onRowClick={(params) => {
          onDocumentSelect(params.row);
        }}
        pageSizeOptions={[]}
        hideFooter
        disableColumnFilter
        disableColumnMenu
        disableColumnSelector
        sx={{
          "& .MuiDataGrid-cell": {
            borderBottom: "1px solid rgba(224, 224, 224, 0.5)",
          },
          "& .MuiDataGrid-cell:focus": {
            outline: "none",
          },
          "& .MuiDataGrid-cell:focus-within": {
            outline: "none",
          },
        }}
      />
    </Box>
  );
}
