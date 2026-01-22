import { Box } from '@mui/material'
import Navigation from './Navigation'
import StatusBar from './StatusBar'

interface LayoutProps {
  isDark: boolean
  onToggleTheme: () => void
  children: React.ReactNode
}

export default function Layout({ isDark, onToggleTheme, children }: LayoutProps) {
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
        <Navigation isDark={isDark} onToggleTheme={onToggleTheme} />

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
