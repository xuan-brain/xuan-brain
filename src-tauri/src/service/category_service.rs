use sea_orm::*;
use tracing::info;

use crate::{
    command::category_command::CategoryDto,
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
        // 即使 parent 没变，如果原本 path 不是以 ID 结尾（旧数据），这里也会统一规范化
        if new_ltree_path != dragged_node.ltree_path {
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

    // 工具：计算新建节点的 ltree_path
    // 已废弃：现在直接使用 ID 生成路径
}
