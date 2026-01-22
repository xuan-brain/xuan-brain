import { Box, Typography, CircularProgress } from '@mui/material'

export default function CategoryTree() {
  return (
    <Box sx={{ p: 1 }}>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 1, textAlign: 'center' }}>
        分类树（待迁移）
      </Typography>
      <Box sx={{ display: 'flex', justifyContent: 'center', p: 2 }}>
        <CircularProgress size={20} />
      </Box>
    </Box>
  )
}
