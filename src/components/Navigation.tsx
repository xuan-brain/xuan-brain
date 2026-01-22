import { useState } from 'react'
import { useNavigate, useLocation } from 'react-router-dom'
import {
  Box,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  IconButton,
  Divider,
  Typography,
  Collapse,
} from '@mui/material'
import {
  MenuBook as LibraryIcon,
  Star,
  Trash2,
  LocalOffer as TagIcon,
  Add,
  ExpandLess,
  ExpandMore,
} from '@mui/icons-material'
import CategoryTree from './CategoryTree'
import TagsSection from './TagsSection'
import AddCategoryDialog from './AddCategoryDialog'
import AddTagDialog from './AddTagDialog'

interface NavigationProps {
  isDark: boolean
  onToggleTheme: () => void
}

export default function Navigation({ isDark, onToggleTheme }: NavigationProps) {
  const navigate = useNavigate()
  const location = useLocation()
  const [activeItem, setActiveItem] = useState('library')
  const [showAddCategoryDialog, setShowAddCategoryDialog] = useState(false)
  const [showAddTagDialog, setShowAddTagDialog] = useState(false)
  const [categoriesExpanded, setCategoriesExpanded] = useState(true)
  const [tagsExpanded, setTagsExpanded] = useState(true)

  const handleNavClick = (itemId: string) => {
    setActiveItem(itemId)
    if (itemId === 'library') {
      navigate('/')
    }
  }

  const handleAddCategory = () => {
    setShowAddCategoryDialog(true)
  }

  const handleAddTag = () => {
    setShowAddTagDialog(true)
  }

  const isActive = (item: string) => activeItem === item

  return (
    <Box
      sx={{
        width: 280,
        bgcolor: 'background.paper',
        borderRight: 1,
        borderColor: 'divider',
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      {/* 导航列表 */}
      <List sx={{ flex: 1, overflow: 'auto', py: 1 }}>
        {/* 文献库 */}
        <ListItem disablePadding>
          <ListItemButton
            selected={isActive('library')}
            onClick={() => handleNavClick('library')}
            sx={{ pl: 2 }}
          >
            <ListItemIcon sx={{ minWidth: 32 }}>
              <LibraryIcon fontSize="small" />
            </ListItemIcon>
            <ListItemText primary="文献库" />
            <IconButton
              size="small"
              onClick={(e) => {
                e.stopPropagation()
                handleAddCategory()
              }}
              sx={{ ml: 1 }}
              edge="end"
            >
              <Add fontSize="small" />
            </IconButton>
          </ListItemButton>
        </ListItem>

        {/* 分类树 */}
        <ListItem disablePadding>
          <ListItemButton
            onClick={() => setCategoriesExpanded(!categoriesExpanded)}
            sx={{ pl: 2 }}
          >
            <ListItemIcon sx={{ minWidth: 32 }}>
              {categoriesExpanded ? <ExpandMore fontSize="small" /> : <ExpandLess fontSize="small" />}
            </ListItemIcon>
            <ListItemText primary="分类" />
          </ListItemButton>
        </ListItem>
        <Collapse in={categoriesExpanded} timeout="auto" unmountOnExit>
          <Box sx={{ pl: 3 }}>
            <CategoryTree />
          </Box>
        </Collapse>

        <Divider sx={{ my: 1 }} />

        {/* 标签 */}
        <ListItem disablePadding>
          <ListItemButton
            onClick={() => setTagsExpanded(!tagsExpanded)}
            sx={{ pl: 2 }}
          >
            <ListItemIcon sx={{ minWidth: 32 }}>
              <TagIcon fontSize="small" />
            </ListItemIcon>
            <ListItemText primary="标签" />
            {tagsExpanded ? <ExpandMore fontSize="small" /> : <ExpandLess fontSize="small" />}
          </ListItemButton>
        </ListItem>
        <Collapse in={tagsExpanded} timeout="auto" unmountOnExit>
          <Box sx={{ pl: 2 }}>
            <TagsSection onAddTag={handleAddTag} />
          </Box>
        </Collapse>

        <Divider sx={{ my: 1 }} />

        {/* 收藏夹 */}
        <ListItem disablePadding>
          <ListItemButton onClick={() => handleNavClick('favorites')} sx={{ pl: 2 }}>
            <ListItemIcon sx={{ minWidth: 32 }}>
              <Star fontSize="small" />
            </ListItemIcon>
            <ListItemText primary="收藏夹" />
          </ListItemButton>
        </ListItem>

        {/* 回收站 */}
        <ListItem disablePadding>
          <ListItemButton onClick={() => handleNavClick('trash')} sx={{ pl: 2 }}>
            <ListItemIcon sx={{ minWidth: 32 }}>
              <Trash2 fontSize="small" />
            </ListItemIcon>
            <ListItemText primary="回收站" />
          </ListItemButton>
        </ListItem>
      </List>

      {/* 对话框 */}
      <AddCategoryDialog
        open={showAddCategoryDialog}
        onClose={() => setShowAddCategoryDialog(false)}
        onCategoryCreated={() => {
          setShowAddCategoryDialog(false)
          // TODO: Refresh CategoryTree
        }}
      />

      <AddTagDialog
        open={showAddTagDialog}
        onClose={() => setShowAddTagDialog(false)}
        onTagCreated={() => {
          setShowAddTagDialog(false)
          // TODO: Refresh TagsSection
        }}
      />
    </Box>
  )
}
