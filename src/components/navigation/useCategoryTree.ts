import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

// Backend data structure
export interface CategoryNode {
  id: number;
  path: string;
  name: string;
  parent_id: number | null;
  sort_order: number;
}

// Headless Tree data structure
export interface TreeDataItem {
  itemName: string;
  childrenIds: string[];
  isFolder: boolean;
}

export function useCategoryTree() {
  const [treeData, setTreeData] = useState<Record<string, TreeDataItem>>({});
  const [loading, setLoading] = useState(true);
  const [expandedItems, setExpandedItems] = useState<string[]>([]);

  // Transform backend data to headless-tree format
  const transformToHeadlessTreeData = useCallback(
    (categories: CategoryNode[]): Record<string, TreeDataItem> => {
      const dataMap: Record<string, TreeDataItem> = {};

      // Sort categories by sort_order
      const sortedCategories = [...categories].sort(
        (a, b) => a.sort_order - b.sort_order
      );

      // Build lookup map
      sortedCategories.forEach((cat) => {
        dataMap[cat.path] = {
          itemName: cat.name,
          childrenIds: [],
          isFolder: true,
        };
      });

      // Link children to parents
      sortedCategories.forEach((cat) => {
        if (cat.parent_id) {
          const parentPath = categories.find(
            (c) => c.id === cat.parent_id
          )?.path;
          if (parentPath && dataMap[parentPath]) {
            dataMap[parentPath].childrenIds.push(cat.path);
          }
        }
      });

      // Add root node
      const rootChildren = sortedCategories
        .filter((cat) => !cat.parent_id)
        .map((cat) => cat.path);

      dataMap["root"] = {
        itemName: "root",
        childrenIds: rootChildren,
        isFolder: true,
      };

      return dataMap;
    },
    []
  );

  // Load categories from backend
  const loadCategoriesData = useCallback(async () => {
    setLoading(true);
    try {
      const categories = await invoke<CategoryNode[]>("load_categories");
      const transformedData = transformToHeadlessTreeData(categories);
      setTreeData(transformedData);

      // Auto-expand logic could be improved or persisted
      // For now, expand if not already set or empty
      if (expandedItems.length === 0) {
        const itemsToExpand = categories
            .filter((cat) => {
            // Level 1: No parent
            if (!cat.parent_id) return true;

            // Level 2: Parent is a root-level item
            const parent = categories.find((p) => p.id === cat.parent_id);
            return parent && !parent.parent_id;
            })
            .map((cat) => cat.path);
        setExpandedItems(itemsToExpand);
      }
    } catch (err) {
      console.error("Failed to load categories:", err);
      // No demo data fallback anymore - real app behavior
    } finally {
      setLoading(false);
    }
  }, [transformToHeadlessTreeData, expandedItems.length]);

  useEffect(() => {
    loadCategoriesData();
  }, []); // Run once on mount

  // Map headless-tree drop event to backend API
  const mapDropToBackend = useCallback(
    (
      items: any[],
      target: any
    ): {
      draggedPath: string;
      targetPath: string | null;
      position: "child" | "above" | "below";
    } => {
      const draggedItem = items[0];
      const draggedPath = draggedItem.getId();

      // Check for reorder/insert (Drop Between)
      if (typeof target.childIndex === "number") {
        const targetItem = target.targetItem || target.item;

        if (!targetItem) {
          console.error(
            "Dropped between items but no parent item found:",
            target
          );
          return {
            draggedPath,
            targetPath: null,
            position: "child",
          };
        }

        const parentId = targetItem.getId();
        const targetChildrenIds = treeData[parentId]?.childrenIds || [];
        const { childIndex } = target;

        // If dropped at the end, make it a child of the parent
        if (childIndex === targetChildrenIds.length) {
          return {
            draggedPath,
            targetPath: parentId === "root" ? null : parentId,
            position: "child",
          };
        }

        // Otherwise, insert ABOVE the sibling at that index
        const siblingId = targetChildrenIds[childIndex];
        return {
          draggedPath,
          targetPath: siblingId,
          position: "above",
        };
      }

      // Check if dropped on item title (make child)
      if ("item" in target) {
        const targetId = target.item.getId();
        return {
          draggedPath,
          targetPath: targetId === "root" ? null : targetId,
          position: "child",
        };
      }

      return {
        draggedPath,
        targetPath: null,
        position: "child",
      };
    },
    [treeData]
  );

  return {
    treeData,
    loading,
    expandedItems,
    setExpandedItems,
    loadCategoriesData,
    mapDropToBackend,
  };
}
