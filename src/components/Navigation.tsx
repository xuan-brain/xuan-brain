import { useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  Box,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  IconButton,
  Divider,
} from "@mui/material";
import {
  MenuBook as LibraryIcon,
  Star,
  Delete,
  LocalOffer as TagIcon,
  Add,
} from "@mui/icons-material";
import { useI18n } from "../lib/i18n";
import CategoryTree from "./CategoryTree";
import TagsSection from "./TagsSection";
import AddCategoryDialog from "./AddCategoryDialog";
import AddTagDialog from "./AddTagDialog";

export default function Navigation() {
  const { t } = useI18n();
  const navigate = useNavigate();
  const [activeItem, setActiveItem] = useState("library");
  const [showAddCategoryDialog, setShowAddCategoryDialog] = useState(false);
  const [showAddTagDialog, setShowAddTagDialog] = useState(false);
  const [categoryTreeKey, setCategoryTreeKey] = useState(0);

  const handleNavClick = (itemId: string) => {
    setActiveItem(itemId);
    if (itemId === "library") {
      navigate("/");
    }
  };

  const handleAddCategory = () => {
    setShowAddCategoryDialog(true);
  };

  const handleAddTag = () => {
    setShowAddTagDialog(true);
  };

  const isActive = (item: string) => activeItem === item;

  return (
    <Box
      sx={{
        width: 240,
        bgcolor: "background.paper",
        borderRight: 1,
        borderColor: "divider",
        display: "flex",
        flexDirection: "column",
      }}
    >
      {/* 导航列表 */}
      <List sx={{ flex: 1, overflow: "auto", py: 0.5 }}>
        {/* 文献库 */}
        <ListItem disablePadding>
          <ListItemButton
            selected={isActive("library")}
            onClick={() => handleNavClick("library")}
            sx={{ pl: 1.5, py: 0.5 }}
          >
            <ListItemIcon sx={{ minWidth: 32 }}>
              <LibraryIcon fontSize="small" />
            </ListItemIcon>
            <ListItemText primary={t("navigation.library")} />
            <IconButton
              size="small"
              onClick={(e) => {
                e.stopPropagation();
                handleAddCategory();
              }}
              sx={{ ml: 1 }}
              edge="end"
            >
              <Add fontSize="small" />
            </IconButton>
          </ListItemButton>
        </ListItem>

        {/* 分类树 */}
        <Box sx={{ pl: 0.5, py: 0.5, pr: 0.5 }}>
          <CategoryTree key={categoryTreeKey} />
        </Box>

        <Divider sx={{ my: 0.5 }} />

        {/* 标签 */}
        <ListItem disablePadding>
          <ListItemButton sx={{ pl: 1.5, py: 0.5 }}>
            <ListItemIcon sx={{ minWidth: 32 }}>
              <TagIcon fontSize="small" />
            </ListItemIcon>
            <ListItemText primary={t("navigation.tags")} />
            <IconButton
              size="small"
              onClick={(e) => {
                e.stopPropagation();
                handleAddTag();
              }}
              sx={{ ml: 1 }}
              edge="end"
            >
              <Add fontSize="small" />
            </IconButton>
          </ListItemButton>
        </ListItem>
        <Box sx={{ pl: 1, py: 0.5 }}>
          <TagsSection onAddTag={handleAddTag} />
        </Box>

        <Divider sx={{ my: 0.5 }} />

        {/* 收藏夹 */}
        <ListItem disablePadding>
          <ListItemButton
            onClick={() => handleNavClick("favorites")}
            sx={{ pl: 1.5, py: 0.5 }}
          >
            <ListItemIcon sx={{ minWidth: 32 }}>
              <Star fontSize="small" />
            </ListItemIcon>
            <ListItemText primary={t("navigation.favorites")} />
          </ListItemButton>
        </ListItem>

        {/* 回收站 */}
        <ListItem disablePadding>
          <ListItemButton
            onClick={() => handleNavClick("trash")}
            sx={{ pl: 1.5, py: 0.5 }}
          >
            <ListItemIcon sx={{ minWidth: 32 }}>
              <Delete fontSize="small" />
            </ListItemIcon>
            <ListItemText primary={t("navigation.trash")} />
          </ListItemButton>
        </ListItem>
      </List>

      {/* 对话框 */}
      <AddCategoryDialog
        open={showAddCategoryDialog}
        onClose={() => setShowAddCategoryDialog(false)}
        onCategoryCreated={() => {
          setShowAddCategoryDialog(false);
          setCategoryTreeKey((prev) => prev + 1);
        }}
      />

      <AddTagDialog
        open={showAddTagDialog}
        onClose={() => setShowAddTagDialog(false)}
        onTagCreated={() => {
          setShowAddTagDialog(false);
          // TODO: Refresh TagsSection
        }}
      />
    </Box>
  );
}
