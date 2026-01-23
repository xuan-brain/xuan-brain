import { Box, Typography } from "@mui/material";

interface DocumentDetailsProps {
  document?: any;
}

export default function DocumentDetails({ document: _ }: DocumentDetailsProps) {
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
        sx={{ textAlign: "center", fontStyle: "italic" }}
      >
        文档详情面板（占位）
      </Typography>
    </Box>
  );
}
