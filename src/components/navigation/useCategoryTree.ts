import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

// Backend data structure from database
export interface CategoryNode {
  id: number;
  path: string;
  name: string;
  parent_id: number | null;
  sort_order: number;
}

// Ant Design Tree node structure
export interface AntTreeNode {
  title: string;
  key: string;
  children?: AntTreeNode[];
}

export function useCategoryTree() {
  const [treeData, setTreeData] = useState<AntTreeNode[]>([]);
  const [loading, setLoading] = useState(true);
  const [expandedKeys, setExpandedKeys] = useState<React.Key[]>([]);
  const [selectedKeys, setSelectedKeys] = useState<React.Key[]>([]);

  // Transform backend data to Ant Design Tree format
  const transformToAntTreeData = useCallback(
    (categories: CategoryNode[]): AntTreeNode[] => {
      // Build map for quick lookup
      const nodeMap = new Map<string, AntTreeNode>();

      // Sort by sort_order
      const sortedCategories = [...categories].sort(
        (a, b) => a.sort_order - b.sort_order,
      );

      // Initialize all nodes
      sortedCategories.forEach((cat) => {
        nodeMap.set(cat.path, {
          title: cat.name,
          key: cat.path,
          children: [],
        });
      });

      // Build tree hierarchy
      const rootNodes: AntTreeNode[] = [];
      sortedCategories.forEach((cat) => {
        const node = nodeMap.get(cat.path)!;
        if (cat.parent_id === null) {
          // Root level
          rootNodes.push(node);
        } else {
          // Find parent and add to children
          const parent = categories.find((c) => c.id === cat.parent_id);
          if (parent) {
            const parentNode = nodeMap.get(parent.path);
            if (parentNode) {
              parentNode.children = parentNode.children || [];
              parentNode.children.push(node);
            }
          }
        }
      });

      return rootNodes;
    },
    [],
  );

  // Load categories from backend
  const loadCategories = useCallback(async () => {
    setLoading(true);
    try {
      const categories = await invoke<CategoryNode[]>("load_categories");
      const transformedData = transformToAntTreeData(categories);
      setTreeData(transformedData);

      // Auto-expand root level nodes
      const rootKeys = categories
        .filter((cat) => cat.parent_id === null)
        .map((cat) => cat.path);
      setExpandedKeys(rootKeys);
    } catch (err) {
      console.error("Failed to load categories:", err);
      // No demo data fallback - real app behavior
    } finally {
      setLoading(false);
    }
  }, [transformToAntTreeData]);

  // Load on mount
  useEffect(() => {
    loadCategories();
  }, [loadCategories]);

  // Handle drop event for drag and drop
  const onDrop = useCallback(
    async (info: any) => {
      const { dragNode, dropNode, dropPosition } = info;

      if (!dropNode || dragNode.key === dropNode.key) {
        return;
      }

      const draggedPath = dragNode.key as string;
      const targetPath = dropNode.key as string;

      // Determine position: -1 (above), 0 (inside), 1 (below)
      let position: "child" | "above" | "below";
      if (dropPosition === 0) {
        position = "child";
      } else if (dropPosition === -1) {
        position = "above";
      } else {
        position = "below";
      }

      try {
        await invoke("move_category", {
          draggedPath,
          targetPath: targetPath === "root" ? null : targetPath,
          position,
        });
        console.info(
          "Successfully moved category:",
          draggedPath,
          "to",
          targetPath,
          "position:",
          position,
        );
        await loadCategories();
      } catch (err) {
        console.error("Failed to move category:", err);
      }
    },
    [loadCategories],
  );

  return {
    treeData,
    loading,
    expandedKeys,
    setExpandedKeys,
    selectedKeys,
    setSelectedKeys,
    loadCategories,
    onDrop,
  };
}
