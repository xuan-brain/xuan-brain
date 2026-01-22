import { useState, useEffect } from 'react'
import { Box, Typography, IconButton, Chip } from '@mui/material'
import { Sync, CloudCircle, Search, Memory } from '@mui/icons-material'

export default function StatusBar() {
  const [currentTime, setCurrentTime] = useState(new Date())
  const [syncStatus, setSyncStatus] = useState<'Synced' | 'Syncing...' | 'Unsynced'>('Synced')
  const [isSyncing, setIsSyncing] = useState(false)
  const documentCount = 0 // TODO: Load from backend

  // Update time every second
  useEffect(() => {
    const intervalId = setInterval(() => {
      setCurrentTime(new Date())
    }, 1000)

    return () => clearInterval(intervalId)
  }, [])

  // Handle sync button click
  const handleSync = () => {
    setIsSyncing(true)
    setSyncStatus('Syncing...')
    setTimeout(() => {
      setIsSyncing(false)
      setSyncStatus('Synced')
    }, 2000)
  }

  return (
    <Box
      sx={{
        height: 32,
        bgcolor: 'background.paper',
        borderTop: 1,
        borderColor: 'divider',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        px: 2,
        userSelect: 'none',
      }}
    >
      {/* Left section */}
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
        {/* Sync status */}
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
          <IconButton
            size="small"
            onClick={handleSync}
            disabled={isSyncing}
            sx={{
              p: 0.5,
              animation: isSyncing ? 'pulse 1s infinite' : 'none',
              '@keyframes pulse': {
                '0%': { opacity: 1 },
                '50%': { opacity: 0.5 },
                '100%': { opacity: 1 },
              },
            }}
          >
            <Sync fontSize="small" />
          </IconButton>
          <Chip
            size="small"
            label={syncStatus}
            color={syncStatus === 'Synced' ? 'success' : syncStatus === 'Syncing...' ? 'warning' : 'default'}
            sx={{ height: 20, fontSize: '0.7rem' }}
          />
        </Box>

        {/* Document count */}
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
          <CloudCircle fontSize="small" sx={{ fontSize: 14 }} />
          <Typography variant="caption" sx={{ fontSize: '0.7rem' }}>
            文献: {documentCount}
          </Typography>
        </Box>

        {/* Search status */}
        <Box sx={{ display: { xs: 'none', md: 'flex' }, alignItems: 'center', gap: 0.5 }}>
          <Search fontSize="small" sx={{ fontSize: 14 }} />
          <Typography variant="caption" sx={{ fontSize: '0.7rem' }}>
            Ready
          </Typography>
        </Box>

        {/* Memory usage */}
        <Box sx={{ display: { xs: 'none', lg: 'flex' }, alignItems: 'center', gap: 0.5 }}>
          <Memory fontSize="small" sx={{ fontSize: 14 }} />
          <Typography variant="caption" sx={{ fontSize: '0.7rem' }}>
            {Math.floor(Math.random() * 100 + 50)} MB
          </Typography>
        </Box>
      </Box>

      {/* Right section */}
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
        <Typography variant="caption" sx={{ display: { xs: 'none', sm: 'block' }, fontSize: '0.7rem' }}>
          v0.1.0
        </Typography>
        <Box
          sx={{
            fontFamily: 'monospace',
            bgcolor: 'action.disabledBackground',
            px: 1,
            py: 0.5,
            borderRadius: 0.5,
          }}
        >
          <Typography variant="caption" sx={{ fontSize: '0.65rem' }}>
            {currentTime.toLocaleTimeString()}
          </Typography>
        </Box>
      </Box>
    </Box>
  )
}
