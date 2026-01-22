import { Box, Typography, Paper, Button } from '@mui/material'
import { Add, Search } from '@mui/icons-material'

export default function LibraryPage() {
  return (
    <Box sx={{ p: 3 }}>
      {/* 页面标题和操作栏 */}
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
        <Typography variant="h5" component="h1">
          文献库
        </Typography>
        <Button variant="contained" startIcon={<Add />}>
          导入文献
        </Button>
      </Box>

      {/* 搜索栏 */}
      <Paper sx={{ p: 2, mb: 3 }}>
        <Box sx={{ display: 'flex', gap: 2, alignItems: 'center' }}>
          <Box sx={{ flex: 1 }}>
            <input
              type="text"
              placeholder="搜索文献..."
              style={{
                width: '100%',
                padding: '8px 12px',
                border: '1px solid #ccc',
                borderRadius: '4px',
                fontSize: '14px',
              }}
            />
          </Box>
          <Button variant="outlined" startIcon={<Search />}>
            搜索
          </Button>
        </Box>
      </Paper>

      {/* 文献列表占位 */}
      <Paper sx={{ p: 4, textAlign: 'center' }}>
        <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
          暂无文献
        </Typography>
        <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
          点击"导入文献"按钮开始使用
        </Typography>
        <Button variant="outlined" startIcon={<Add />}>
          导入第一篇文献
        </Button>
      </Paper>

      {/* 统计信息 */}
      <Box sx={{ mt: 3, display: 'flex', gap: 3 }}>
        <Paper sx={{ p: 2, flex: 1, textAlign: 'center' }}>
          <Typography variant="h6" color="primary">
            0
          </Typography>
          <Typography variant="caption" color="text.secondary">
            总文献数
          </Typography>
        </Paper>
        <Paper sx={{ p: 2, flex: 1, textAlign: 'center' }}>
          <Typography variant="h6" color="secondary">
            0
          </Typography>
          <Typography variant="caption" color="text.secondary">
            分类数量
          </Typography>
        </Paper>
        <Paper sx={{ p: 2, flex: 1, textAlign: 'center' }}>
          <Typography variant="h6" color="info">
            0
          </Typography>
          <Typography variant="caption" color="text.secondary">
            标签数量
          </Typography>
        </Paper>
      </Box>
    </Box>
  )
}
