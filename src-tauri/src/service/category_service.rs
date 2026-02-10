use sea_orm::*;
use tracing::info;

use crate::{
    command::category_command::{CategoryDto, TreeNodeDto},
    database::entities::prelude::Category,
    database::entities::{self},
};

pub struct CategoryService<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> CategoryService<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    // 加载整棵树，转为 CategoryDto[]（含 ltree_path）
    pub async fn load_tree(&self) -> Result<Vec<CategoryDto>, DbErr> {
        let all = Category::find()
            .order_by_asc(entities::category::Column::LtreePath)
            .all(self.db)
            .await?;

        let dtos: Vec<CategoryDto> = all
            .into_iter()
            .map(|m| CategoryDto {
                id: m.id,
                path: m.ltree_path,
                name: m.name,
                parent_id: m.parent_id,
                sort_order: m.sort_order,
            })
            .collect();

        Ok(dtos)
    }

    // 新建分类：parentPath 为 NULL 表示根节点
    pub async fn create(&self, name: &str, parent_path: Option<&str>) -> Result<(), DbErr> {
        // 1. 找父节点（如果有）
        let parent_opt = match parent_path {
            Some(path) => Some(
                Category::find()
                    .filter(entities::category::Column::LtreePath.eq(path))
                    .one(self.db)
                    .await?
                    .ok_or_else(|| DbErr::Custom("Parent not found".into()))?,
            ),
            None => None,
        };

        // 2. 插入新记录，使用临时路径 "tmp" (只要符合 ltree 格式即可，字母数字)
        // 随后立即更新为基于 ID 的路径，确保唯一性和一致性
        let active = entities::category::ActiveModel {
            id: ActiveValue::NotSet,
            name: ActiveValue::Set(name.into()),
            parent_id: ActiveValue::Set(parent_opt.as_ref().map(|p| p.id)),
            ltree_path: ActiveValue::Set("tmp".to_owned()),
            sort_order: ActiveValue::Set(0),
            created_at: ActiveValue::Set(chrono::Local::now().to_rfc3339()),
        };

        let inserted = active.insert(self.db).await?;
        let new_id = inserted.id;

        // 3. 更新为真正的 ltree_path (基于 ID)
        let real_path = if let Some(parent) = parent_opt {
            format!("{}.{}", parent.ltree_path, new_id)
        } else {
            new_id.to_string()
        };

        let mut active_update: entities::category::ActiveModel = inserted.into();
        active_update.ltree_path = ActiveValue::Set(real_path);
        active_update.update(self.db).await?;

        Ok(())
    }

    // 根据 ltree_path 删除节点及所有子孙
    pub async fn delete_by_path(&self, path: &str) -> Result<DeleteResult, DbErr> {
        Category::delete_many()
            .filter(entities::category::Column::LtreePath.starts_with(path))
            .exec(self.db)
            .await
    }

    // 根据 ltree_path 更新分类名称
    pub async fn update_by_path(&self, path: &str, name: &str) -> Result<(), DbErr> {
        let category = Category::find()
            .filter(entities::category::Column::LtreePath.eq(path))
            .one(self.db)
            .await?
            .ok_or_else(|| DbErr::Custom("Category not found".into()))?;

        let mut active: entities::category::ActiveModel = category.into();
        active.name = ActiveValue::Set(name.to_string());
        active.update(self.db).await?;
        Ok(())
    }

    // 移动节点: 核心逻辑是根据 targetPath + position 更新 parent_id 和 sort_order
    pub async fn move_node(
        &self,
        dragged_path: &str,
        target_path: Option<&str>,
        position: &str, // "above" | "below" | "child"
    ) -> Result<String, DbErr> {
        info!(
            "move_node called: dragged={}, target={:?}, pos={}",
            dragged_path, target_path, position
        );

        // 1. 获取 dragged node
        let dragged_node = Category::find()
            .filter(entities::category::Column::LtreePath.eq(dragged_path))
            .one(self.db)
            .await?
            .ok_or_else(|| DbErr::Custom("Dragged node not found".into()))?;

        // 2. 确定新的 parent 信息
        let (new_parent_id, new_parent_path) = if let Some(t_path) = target_path {
            let target_node = Category::find()
                .filter(entities::category::Column::LtreePath.eq(t_path))
                .one(self.db)
                .await?
                .ok_or_else(|| DbErr::Custom("Target node not found".into()))?;

            if position == "child" {
                (Some(target_node.id), Some(target_node.ltree_path))
            } else {
                // above or below: parent is target's parent
                if let Some(pid) = target_node.parent_id {
                    let parent = Category::find_by_id(pid)
                        .one(self.db)
                        .await?
                        .ok_or_else(|| DbErr::Custom("Target parent not found".into()))?;
                    (Some(pid), Some(parent.ltree_path))
                } else {
                    (None, None) // Root level
                }
            }
        } else {
            (None, None) // Move to root
        };

        // 3. 计算新路径 (仅当 parent 改变或 path 需要刷新时)
        // 使用 ID 保证唯一性: new_parent_path.id OR id (if root)
        let new_ltree_path = if let Some(ref pp) = new_parent_path {
            format!("{}.{}", pp, dragged_node.id)
        } else {
            dragged_node.id.to_string()
        };

        // 4. 更新子树路径 (如果路径改变了)
        // 如果从有parent变为root，或者从root变为有parent，或者在根级别重新排序，需要更新所有其他root节点的路径
        let need_reindex_roots = (dragged_node.parent_id.is_some() && new_parent_id.is_none())
            || (dragged_node.parent_id.is_none() && new_parent_id.is_some())
            || (dragged_node.parent_id.is_none() && new_parent_id.is_none()); // 始终在根级别移动时重新索引

        if new_ltree_path != dragged_node.ltree_path || need_reindex_roots {
            // 查找所有以 dragged_path 开头的节点
            let all_descendants = Category::find()
                .filter(entities::category::Column::LtreePath.starts_with(dragged_path))
                .all(self.db)
                .await?;

            for descendant in all_descendants {
                let suffix_str = descendant
                    .ltree_path
                    .strip_prefix(dragged_path)
                    .unwrap_or("")
                    .to_string(); // Own the string

                let new_desc_path = format!("{}{}", new_ltree_path, suffix_str);

                let mut am: entities::category::ActiveModel = descendant.into();
                am.ltree_path = ActiveValue::Set(new_desc_path);
                // 如果是 dragged node 本身，顺便更新 parent_id
                if suffix_str.is_empty() {
                    am.parent_id = ActiveValue::Set(new_parent_id);
                }
                am.update(self.db).await?;
            }

            // 如果是在根级别之间移动，需要重新索引所有根节点
            if need_reindex_roots {
                // 获取所有根节点（按 sort_order 排序）
                let all_roots = Category::find()
                    .filter(entities::category::Column::ParentId.is_null())
                    .order_by_asc(entities::category::Column::SortOrder)
                    .order_by_asc(entities::category::Column::LtreePath)
                    .all(self.db)
                    .await?;

                // 重新分配所有根节点的 ltree_path
                for root in all_roots.iter() {
                    let new_path = root.id.to_string();
                    if root.ltree_path != new_path {
                        let mut am: entities::category::ActiveModel = root.clone().into();
                        am.ltree_path = ActiveValue::Set(new_path.clone());

                        // 更新此根节点下所有子节点的路径
                        let children = Category::find()
                            .filter(
                                entities::category::Column::LtreePath.starts_with(&root.ltree_path),
                            )
                            .filter(entities::category::Column::LtreePath.ne(&root.ltree_path))
                            .all(self.db)
                            .await?;

                        for child in children {
                            let suffix = child
                                .ltree_path
                                .strip_prefix(&root.ltree_path)
                                .unwrap_or("")
                                .to_string();
                            let new_child_path = format!("{}{}", new_path, suffix);
                            let mut child_am: entities::category::ActiveModel = child.into();
                            child_am.ltree_path = ActiveValue::Set(new_child_path);
                            child_am.update(self.db).await?;
                        }

                        am.update(self.db).await?;
                    }
                }
            }
        } else {
            // 路径没变（同级移动且 parent 没变），但可能需要更新 parent_id
            let mut am: entities::category::ActiveModel = dragged_node.clone().into();
            am.parent_id = ActiveValue::Set(new_parent_id);
            am.update(self.db).await?;
        }

        // 5. 处理排序 (Sort Order)
        // 获取新 Parent 下的所有子节点（包括刚移动过来的 dragged_node）
        // 注意：这里需要重新从 DB 查一次，确保拿到最新的状态
        let mut siblings = if let Some(pid) = new_parent_id {
            Category::find()
                .filter(entities::category::Column::ParentId.eq(pid))
                .order_by_asc(entities::category::Column::SortOrder)
                .order_by_asc(entities::category::Column::LtreePath)
                .all(self.db)
                .await?
        } else {
            Category::find()
                .filter(entities::category::Column::ParentId.is_null())
                .order_by_asc(entities::category::Column::SortOrder)
                .order_by_asc(entities::category::Column::LtreePath)
                .all(self.db)
                .await?
        };

        // 从列表中临时移除 dragged_node (通过 ID 识别)
        let dragged_idx = siblings.iter().position(|c| c.id == dragged_node.id);
        if let Some(idx) = dragged_idx {
            siblings.remove(idx);
        }

        // 找到插入位置
        let insert_index = if let Some(t_path) = target_path {
            if position == "child" {
                // 插入到末尾
                siblings.len()
            } else {
                // above or below
                // 找到 target 在 siblings 中的位置
                let target_node = Category::find()
                    .filter(entities::category::Column::LtreePath.eq(t_path))
                    .one(self.db)
                    .await?
                    .ok_or_else(|| DbErr::Custom("Target node not found for sorting".into()))?; // Should exist

                let t_idx = siblings.iter().position(|c| c.id == target_node.id);

                if let Some(idx) = t_idx {
                    if position == "above" {
                        idx
                    } else {
                        idx + 1
                    }
                } else {
                    siblings.len()
                }
            }
        } else {
            siblings.len() // Root end
        };

        // 插入 dragged_node
        let current_dragged = Category::find_by_id(dragged_node.id)
            .one(self.db)
            .await?
            .ok_or_else(|| DbErr::Custom("Dragged node lost".into()))?;

        let safe_insert_index = insert_index.min(siblings.len());
        siblings.insert(safe_insert_index, current_dragged);

        // 6. 批量更新 sort_order
        for (index, node) in siblings.iter().enumerate() {
            if node.sort_order != index as i64 {
                let mut am: entities::category::ActiveModel = node.clone().into();
                am.sort_order = ActiveValue::Set(index as i64);
                am.update(self.db).await?;
            }
        }

        Ok(new_ltree_path)
    }

    // 根据新的树结构重建整个数据库
    // 前端拖拽后调用此方法，基于新的树结构刷新所有节点的路径
    pub async fn rebuild_tree_from_structure(
        &self,
        new_structure: &[TreeNodeDto],
    ) -> Result<(), DbErr> {
        info!(
            "Rebuilding tree from structure with {} root nodes",
            new_structure.len()
        );

        // 递归处理整个树结构
        self.rebuild_tree_recursive(new_structure, None, None, 0)
            .await?;

        info!("Tree rebuild completed");
        Ok(())
    }

    // 递归重建树结构
    async fn rebuild_tree_recursive(
        &self,
        nodes: &[TreeNodeDto],
        parent_path: Option<&str>,
        parent_id: Option<i64>,
        sort_order: i64,
    ) -> Result<(), DbErr> {
        for (index, node) in nodes.iter().enumerate() {
            let current_sort_order = sort_order + index as i64;
            let current_index = (index + 1) as i64; // ltree_path 编号从 1 开始

            // 计算新的 ltree_path
            let new_path = if let Some(pp) = parent_path {
                // 子节点：parent_path.index
                format!("{}.{}", pp, current_index)
            } else {
                // 根节点：直接用 index
                current_index.to_string()
            };

            // 更新当前节点
            let category = Category::find_by_id(node.id)
                .one(self.db)
                .await?
                .ok_or_else(|| DbErr::Custom(format!("Category {} not found", node.id)))?;

            let mut am: entities::category::ActiveModel = category.into();
            am.ltree_path = ActiveValue::Set(new_path.clone());
            am.parent_id = ActiveValue::Set(parent_id);
            am.sort_order = ActiveValue::Set(current_sort_order);
            am.update(self.db).await?;

            info!(
                "Updated category {}: path={}, parent_id={:?}, sort_order={}",
                node.id, new_path, parent_id, current_sort_order
            );

            // 递归处理子节点
            if let Some(children) = &node.children {
                if !children.is_empty() {
                    Box::pin(self.rebuild_tree_recursive(
                        children,
                        Some(new_path.as_str()),
                        Some(node.id),
                        0,
                    ))
                    .await?;
                }
            }
        }
        Ok(())
    }

    // 工具：计算新建节点的 ltree_path
    // 已废弃：现在直接使用 ID 生成路径
}
