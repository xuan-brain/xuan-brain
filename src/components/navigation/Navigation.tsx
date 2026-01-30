import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button, Divider } from "antd";
import {
  BookOutlined as LibraryIcon,
  StarOutlined,
  DeleteOutlined,
  TagOutlined,
  PlusOutlined,
} from "@ant-design/icons";
import { useI18n } from "../../lib/i18n";
import CategoryTree from "./CategoryTree";
import TagsSection from "./TagsSection";
import AddCategoryDialog from "../dialogs/AddCategoryDialog";
import AddTagDialog from "../dialogs/AddTagDialog";
import { useAppStore } from "../../stores/useAppStore";

interface NavigationProps {
  onCategorySelect?: (categoryId: string | null) => void;
}

export default function Navigation({ onCategorySelect }: NavigationProps) {
  const { t } = useI18n();
  const { accentColor } = useAppStore();
  const navigate = useNavigate();
  const [activeItem, setActiveItem] = useState("library");
  const [showAddCategoryDialog, setShowAddCategoryDialog] = useState(false);
  const [showAddTagDialog, setShowAddTagDialog] = useState(false);
  const [categoryTreeKey, setCategoryTreeKey] = useState(0);
  const [tagsSectionKey, setTagsSectionKey] = useState(0);

  const handleNavClick = (itemId: string) => {
    setActiveItem(itemId);
    if (itemId === "library") {
      navigate("/");
      if (onCategorySelect) onCategorySelect(null);
    } else if (itemId === "trash") {
      if (onCategorySelect) onCategorySelect("trash");
    }
  };

  const handleCategorySelect = (categoryId: string | null) => {
    setActiveItem("library");
    if (onCategorySelect) {
      onCategorySelect(categoryId);
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
    <div
      style={{
        borderRight: "1px solid var(--ant-color-border)",
        display: "flex",
        flexDirection: "column",
        height: "100%",
      }}
    >
      {/* 上部：文献库和分类树 */}
      <div style={{ flex: 1, overflow: "auto", padding: "4px 0" }}>
        {/* 文献库 */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            padding: "4px 12px",
            cursor: "pointer",
            backgroundColor: isActive("library")
              ? `${accentColor}26`
              : "transparent",
            transition: "background-color 0.2s",
          }}
          onClick={() => handleNavClick("library")}
        >
          <LibraryIcon
            style={{
              marginRight: 8,
              minWidth: 20,
              color: isActive("library")
                ? accentColor
                : "var(--ant-color-text)",
            }}
          />
          <span
            style={{
              flex: 1,
              color: isActive("library")
                ? accentColor
                : "var(--ant-color-text)",
              fontWeight: isActive("library") ? 500 : 400,
            }}
          >
            {t("navigation.library")}
          </span>
          <Button
            type="text"
            size="small"
            icon={<PlusOutlined style={{ color: "var(--ant-color-text)" }} />}
            onClick={(e) => {
              e.stopPropagation();
              handleAddCategory();
            }}
            style={{ marginLeft: 8 }}
          />
        </div>

        {/* 分类树 */}
        <div style={{ padding: "4px 2px" }}>
          <CategoryTree
            key={categoryTreeKey}
            onCategorySelect={handleCategorySelect}
          />
        </div>
      </div>

      {/* 下部：停靠区域（标签、收藏夹、回收站） */}
      <div
        style={{
          borderTop: "1px solid var(--ant-color-border)",
        }}
      >
        <div style={{ padding: "4px 0" }}>
          {/* 标签 */}
          <div
            style={{
              display: "flex",
              alignItems: "center",
              padding: "4px 12px",
            }}
          >
            <TagOutlined
              style={{
                marginRight: 8,
                minWidth: 20,
                color: "var(--ant-color-text)",
              }}
            />
            <span style={{ flex: 1, color: "var(--ant-color-text)" }}>
              {t("navigation.tags")}
            </span>
            <Button
              type="text"
              size="small"
              icon={<PlusOutlined style={{ color: "var(--ant-color-text)" }} />}
              onClick={(e) => {
                e.stopPropagation();
                handleAddTag();
              }}
              style={{ marginLeft: 8 }}
            />
          </div>
          <div style={{ padding: "4px 4px" }}>
            <TagsSection key={tagsSectionKey} onAddTag={handleAddTag} />
          </div>

          <Divider style={{ margin: "4px 0" }} />

          {/* 收藏夹 */}
          <div
            style={{
              display: "flex",
              alignItems: "center",
              padding: "4px 12px",
              cursor: "pointer",
              transition: "background-color 0.2s",
              backgroundColor: isActive("favorites")
                ? `${accentColor}26`
                : "transparent",
            }}
            onClick={() => handleNavClick("favorites")}
          >
            <StarOutlined
              style={{
                marginRight: 8,
                minWidth: 20,
                color: isActive("favorites")
                  ? accentColor
                  : "var(--ant-color-text)",
              }}
            />
            <span
              style={{
                color: isActive("favorites")
                  ? accentColor
                  : "var(--ant-color-text)",
                fontWeight: isActive("favorites") ? 500 : 400,
              }}
            >
              {t("navigation.favorites")}
            </span>
          </div>

          <Divider style={{ margin: "4px 0" }} />

          {/* 回收站 */}
          <div
            style={{
              display: "flex",
              alignItems: "center",
              padding: "4px 12px",
              cursor: "pointer",
              backgroundColor: isActive("trash")
                ? `${accentColor}26`
                : "transparent",
              transition: "background-color 0.2s",
            }}
            onClick={() => handleNavClick("trash")}
          >
            <DeleteOutlined
              style={{
                marginRight: 8,
                minWidth: 20,
                color: isActive("trash")
                  ? accentColor
                  : "var(--ant-color-text)",
              }}
            />
            <span
              style={{
                color: isActive("trash")
                  ? accentColor
                  : "var(--ant-color-text)",
                fontWeight: isActive("trash") ? 500 : 400,
              }}
            >
              {t("navigation.trash")}
            </span>
          </div>
        </div>
      </div>

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
          setTagsSectionKey((prev) => prev + 1);
        }}
      />
    </div>
  );
}
