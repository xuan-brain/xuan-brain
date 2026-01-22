import { Box } from '@mui/material'
import Navigation from './Navigation'
import StatusBar from './StatusBar'

interface LayoutProps {
  children?: React.ReactNode
}

export default function Layout({ children }: LayoutProps) {
  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        height: '100vh',
        overflow: 'hidden',
      }}
    >
      <Box
        sx={{
          display: 'flex',
          flex: 1,
          overflow: 'hidden',
        }}
      >
        {/* 侧边导航栏 */}
        <Navigation />

        {/* 主内容区域 */}
        <Box
          component="main"
          sx={{
            flex: 1,
            overflow: 'auto',
            bgcolor: 'background.default',
          }}
        >
          {children}
        </Box>
      </Box>

      {/* 底部状态栏 */}
      <StatusBar />
    </Box>
  )
}
