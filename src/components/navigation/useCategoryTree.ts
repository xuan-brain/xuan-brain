import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

// Backend data structure from database
export interface CategoryNode {
  id: number;
  path: string;
  name: string;
  parent_id?: number | null;
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
  const [rawCount, setRawCount] = useState<number>(0);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

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
        // Check for null or undefined (backend might skip serializing null fields)
        if (cat.parent_id === null || cat.parent_id === undefined) {
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
            } else {
              // Should not happen if map is built correctly
              console.warn(
                `Parent mapped but missing in nodeMap: ${parent.path}`,
              );
              rootNodes.push(node);
            }
          } else {
            // Parent not found in the list (Orphan node), treat as root
            console.warn(
              `Parent ID ${cat.parent_id} not found for category ${cat.name} (${cat.path}), treating as root`,
            );
            rootNodes.push(node);
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
      console.log("Loaded categories:", categories);
      setRawCount(categories.length);
      setErrorMsg(null);

      const transformedData = transformToAntTreeData(categories);
      setTreeData(transformedData);

      // Auto-expand root level (Level 1) and second level (Level 2) nodes
      const rootLevelNodes = categories.filter(
        (cat) => cat.parent_id === null || cat.parent_id === undefined,
      );
      const rootLevelIds = new Set(rootLevelNodes.map((n) => n.id));

      const secondLevelNodes = categories.filter(
        (cat) =>
          cat.parent_id !== null &&
          cat.parent_id !== undefined &&
          rootLevelIds.has(cat.parent_id),
      );

      const keysToExpand = [
        ...rootLevelNodes.map((n) => n.path),
        ...secondLevelNodes.map((n) => n.path),
      ];
      setExpandedKeys(keysToExpand);
    } catch (err) {
      console.error("Failed to load categories:", err);
      setErrorMsg(String(err));
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
      const { dragNode, node } = info;
      const dropNode = node;

      if (!dropNode || !dragNode || dragNode.key === dropNode.key) {
        return;
      }

      // Calculate drop position relative to the target node
      const dropPos = dropNode.pos.split("-");
      const dropPosition =
        info.dropPosition - Number(dropPos[dropPos.length - 1]);

      const draggedPath = dragNode.key as string;
      const targetPath = dropNode.key as string;

      // Determine position based on dropToGap and calculated relative position
      let position: "child" | "above" | "below";

      if (!info.dropToGap) {
        position = "child";
      } else {
        if (dropPosition === -1) {
          position = "above";
        } else {
          position = "below";
        }
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
    rawCount,
    errorMsg,
  };
}
