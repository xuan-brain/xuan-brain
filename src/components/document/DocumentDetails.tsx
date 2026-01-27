import { useEffect, useState } from "react";
import {
  Box,
  Typography,
  Paper,
  Divider,
  Chip,
  Stack,
  Link,
  CircularProgress,
} from "@mui/material";
import { useI18n } from "../../lib/i18n";

// Helper for invoke (duplicate from DocumentList, maybe should move to a shared lib later)
async function invokeCommand<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

interface PaperDetailDto {
  id: number;
  title: string;
  abstract_text?: string;
  doi?: string;
  publication_year?: number;
  publication_date?: string;
  journal_name?: string;
  conference_name?: string;
  volume?: string;
  issue?: string;
  pages?: string;
  url?: string;
  citation_count?: number;
  read_status?: string;
  notes?: string;
  authors: string[];
}

interface DocumentDetailsProps {
  document?: {
    id: number;
    title: string;
    authors: string[];
    year: number;
    abstract?: string;
    keywords?: string[];
    fileType?: string;
    fileSize?: string;
    addedDate?: string;
    tags?: { id: number; name: string; color: string }[];
  } | null;
}

export default function DocumentDetails({ document }: DocumentDetailsProps) {
  const { t } = useI18n();
  const [details, setDetails] = useState<PaperDetailDto | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (document?.id) {
      loadPaperDetails(document.id);
    } else {
      setDetails(null);
    }
  }, [document]);

  const loadPaperDetails = async (id: number) => {
    setLoading(true);
    try {
      const data = await invokeCommand<PaperDetailDto>("get_paper", { id });
      setDetails(data);
    } catch (error) {
      console.error("Failed to load paper details:", error);
      // Fallback for dev/demo without backend
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      if (!(window as any).__TAURI_INTERNALS__) {
        setDetails({
          id: id,
          title: "Demo Paper Title",
          abstract_text:
            "This is a demo abstract because the backend call failed or is not available.",
          authors: ["Demo Author"],
          publication_year: 2024,
          journal_name: "Demo Journal",
        });
      }
    } finally {
      setLoading(false);
    }
  };

  if (!document) {
    return (
      <Box
        sx={{
          p: 2,
          height: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        <Typography
          variant="body2"
          color="text.secondary"
          sx={{ fontStyle: "italic" }}
        >
          {t("document.select_to_view") || "Select a document to view details"}
        </Typography>
      </Box>
    );
  }

  if (loading) {
    return (
      <Box
        sx={{
          p: 2,
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
          height: "100%",
        }}
      >
        <CircularProgress />
      </Box>
    );
  }

  if (!details) {
    return (
      <Box sx={{ p: 2, display: "flex", justifyContent: "center" }}>
        <Typography variant="body2" color="error">
          Failed to load details.
        </Typography>
      </Box>
    );
  }

  return (
    <Box sx={{ p: 3, height: "100%", overflow: "auto" }}>
      <Typography variant="h5" gutterBottom fontWeight="bold" component="div">
        {details.title}
      </Typography>

      <Stack
        direction="row"
        spacing={1}
        sx={{ mb: 2, flexWrap: "wrap", gap: 1 }}
        alignItems="center"
        useFlexGap
      >
        {details.publication_year && (
          <Chip
            label={details.publication_year}
            size="small"
            variant="outlined"
          />
        )}
        {(details.journal_name || details.conference_name) && (
          <Chip
            label={details.journal_name || details.conference_name}
            size="small"
            color="primary"
            variant="outlined"
          />
        )}
        {details.read_status && (
          <Chip
            label={details.read_status}
            size="small"
            color={details.read_status === "read" ? "success" : "default"}
          />
        )}
      </Stack>

      <Typography
        variant="subtitle1"
        gutterBottom
        sx={{ fontWeight: "medium" }}
      >
        {details.authors.join(", ")}
      </Typography>

      {(details.doi || details.url) && (
        <Box sx={{ mb: 2 }}>
          {details.doi && (
            <Typography variant="body2">
              DOI:{" "}
              <Link
                href={`https://doi.org/${details.doi}`}
                target="_blank"
                rel="noopener"
              >
                {details.doi}
              </Link>
            </Typography>
          )}
          {details.url && (
            <Typography variant="body2">
              URL:{" "}
              <Link href={details.url} target="_blank" rel="noopener">
                {details.url}
              </Link>
            </Typography>
          )}
        </Box>
      )}

      <Divider sx={{ my: 2 }} />

      {details.abstract_text && (
        <Box sx={{ mb: 3 }}>
          <Typography variant="h6" gutterBottom>
            Abstract
          </Typography>
          <Typography
            variant="body1"
            sx={{ whiteSpace: "pre-wrap", lineHeight: 1.6 }}
          >
            {details.abstract_text}
          </Typography>
        </Box>
      )}

      {details.notes && (
        <Box sx={{ mb: 3 }}>
          <Typography variant="h6" gutterBottom>
            Notes
          </Typography>
          <Paper variant="outlined" sx={{ p: 2, bgcolor: "action.hover" }}>
            <Typography variant="body2" sx={{ whiteSpace: "pre-wrap" }}>
              {details.notes}
            </Typography>
          </Paper>
        </Box>
      )}

      <Box sx={{ mt: 4, pt: 2, borderTop: 1, borderColor: "divider" }}>
        <Typography variant="caption" display="block" color="text.secondary">
          ID: {details.id} | Citations: {details.citation_count || 0}
        </Typography>
      </Box>
    </Box>
  );
}
