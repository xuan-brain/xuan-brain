import { useState, useEffect } from "react";
import { DataGrid, GridColDef } from "@mui/x-data-grid";
import { Box } from "@mui/material";
import { useI18n } from "../../lib/i18n";

// Lazy load invoke helper - works in both Tauri and browser
async function invokeCommand<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

interface PaperDto {
  id: number;
  title: string;
  publication_year?: number;
  journal_name?: string;
  conference_name?: string;
  authors: string[];
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
      valueGetter: (_value: unknown, row: PaperDto) => {
        return row.authors ? row.authors.join(", ") : "";
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
  ];

  return (
    <Box sx={{ height: "100%", width: "100%" }}>
      <DataGrid
        rows={rows}
        columns={columns}
        loading={loading}
        density="compact"
        onRowClick={(params) => onDocumentSelect(params.row)}
        initialState={{
          pagination: {
            paginationModel: { page: 0, pageSize: 25 },
          },
        }}
        pageSizeOptions={[25, 50, 100]}
      />
    </Box>
  );
}
