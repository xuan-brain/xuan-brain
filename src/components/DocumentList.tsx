import { Box, Typography } from "@mui/material";

interface LibraryPageProps {
  onDocumentSelect: (document: any) => void;
}

export default function LibraryPage({ onDocumentSelect: _ }: LibraryPageProps) {
  return (
    <Box
      sx={{
        p: 4,
        height: "100%",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <Typography
        variant="h4"
        color="text.secondary"
        sx={{ textAlign: "center", fontStyle: "italic" }}
      >
        文献列表页面（占位）
      </Typography>
    </Box>
  );
}
