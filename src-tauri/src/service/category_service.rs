use sea_orm::*;
use tracing::info;

use crate::{
    command::category_command::{CategoryDto, TreeNodeDto},
    database::entities,
    database::entities::prelude::Category,
};

pub struct CategoryService<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> CategoryService<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    // 加载整棵树，转为 CategoryDto[]
    pub async fn load_tree(&self) -> Result<Vec<CategoryDto>, DbErr> {
        let all = Category::find()
            .order_by_asc(entities::category::Column::SortOrder)
            .all(self.db)
            .await?;

        let dtos: Vec<CategoryDto> = all
            .into_iter()
            .map(|m| CategoryDto {
                id: m.id,
                name: m.name,
                parent_id: m.parent_id,
                sort_order: m.sort_order,
            })
            .collect();

        Ok(dtos)
    }

    // 新建分类：parent_id 为 NULL 表示根节点
    pub async fn create(&self, name: &str, parent_id: Option<i64>) -> Result<(), DbErr> {
        info!(
            "Creating category '{}' with parent_id: {:?}",
            name, parent_id
        );

        // 计算 sort_order（同级节点数量）
        let new_sort_order = if let Some(pid) = parent_id {
            // 有父节点：计算同级节点数量
            Category::find()
                .filter(entities::category::Column::ParentId.eq(pid))
                .count(self.db)
                .await? as i64
        } else {
            // 根节点：计算所有根节点数量
            Category::find()
                .filter(entities::category::Column::ParentId.is_null())
                .count(self.db)
                .await? as i64
        };

        // 插入新记录
        let active = entities::category::ActiveModel {
            id: ActiveValue::NotSet,
            name: ActiveValue::Set(name.into()),
            parent_id: ActiveValue::Set(parent_id),
            sort_order: ActiveValue::Set(new_sort_order),
            created_at: ActiveValue::Set(chrono::Local::now().to_rfc3339()),
        };

        active.insert(self.db).await?;

        info!(
            "Category created successfully: name={}, sort_order={}",
            name, new_sort_order
        );
        Ok(())
    }

    // 递归删除节点及所有子孙（通过 ID）
    pub async fn delete_by_id(&self, id: i64) -> Result<DeleteResult, DbErr> {
        info!("Deleting category and all descendants with id={}", id);

        // 先找到所有子孙节点（递归查找）
        let descendants = self.find_all_descendants(id).await?;

        // 删除所有子孙节点和当前节点
        let all_ids_to_delete: Vec<i64> = descendants.into_iter().map(|m| m.id).collect();

        let result = Category::delete_many()
            .filter(entities::category::Column::Id.is_in(all_ids_to_delete))
            .exec(self.db)
            .await?;

        info!("Deleted {} categories", result.rows_affected);
        Ok(result)
    }

    // 辅助方法：递归查找所有子孙节点
    async fn find_all_descendants(
        &self,
        parent_id: i64,
    ) -> Result<Vec<entities::category::Model>, DbErr> {
        let mut all_descendants = Vec::new();
        let mut to_process = vec![parent_id];

        while let Some(current_id) = to_process.pop() {
            // 查找直接子节点
            let children = Category::find()
                .filter(entities::category::Column::ParentId.eq(current_id))
                .all(self.db)
                .await?;

            for child in children {
                to_process.push(child.id);
                all_descendants.push(child);
            }
        }

        Ok(all_descendants)
    }

    // 根据 ID 更新分类名称
    pub async fn update_by_id(&self, id: i64, name: &str) -> Result<(), DbErr> {
        info!("Updating category id={} to name '{}'", id, name);

        let category = Category::find_by_id(id)
            .one(self.db)
            .await?
            .ok_or_else(|| DbErr::Custom("Category not found".into()))?;

        let mut active: entities::category::ActiveModel = category.into();
        active.name = ActiveValue::Set(name.to_string());
        active.update(self.db).await?;

        info!("Category updated successfully");
        Ok(())
    }

    // 移动节点：更新 parent_id 和重新计算 sort_order
    pub async fn move_node(
        &self,
        dragged_id: i64,
        target_id: Option<i64>,
        position: &str, // "above" | "below" | "child"
    ) -> Result<(), DbErr> {
        info!(
            "move_node called: dragged_id={}, target_id={:?}, pos={}",
            dragged_id, target_id, position
        );

        // 1. 获取 dragged node
        let dragged_node = Category::find_by_id(dragged_id)
            .one(self.db)
            .await?
            .ok_or_else(|| DbErr::Custom("Dragged node not found".into()))?;

        // 2. 确定新的 parent_id
        let new_parent_id = if let Some(tid) = target_id {
            if position == "child" {
                Some(tid)
            } else {
                // above or below: parent is target's parent
                let target_node = Category::find_by_id(tid)
                    .one(self.db)
                    .await?
                    .ok_or_else(|| DbErr::Custom("Target node not found".into()))?;
                target_node.parent_id
            }
        } else {
            None // Move to root
        };

        // 3. 更新 dragged_node 的 parent_id
        let mut dragged_active: entities::category::ActiveModel = dragged_node.into();
        dragged_active.parent_id = ActiveValue::Set(new_parent_id);
        dragged_active.update(self.db).await?;

        // 4. 重新计算所有同级节点的 sort_order
        self.reorder_siblings(new_parent_id).await?;

        info!("Category moved successfully");
        Ok(())
    }

    // 重新排序同级节点
    async fn reorder_siblings(&self, parent_id: Option<i64>) -> Result<(), DbErr> {
        let siblings = if let Some(pid) = parent_id {
            Category::find()
                .filter(entities::category::Column::ParentId.eq(pid))
                .order_by_asc(entities::category::Column::SortOrder)
                .all(self.db)
                .await?
        } else {
            Category::find()
                .filter(entities::category::Column::ParentId.is_null())
                .order_by_asc(entities::category::Column::SortOrder)
                .all(self.db)
                .await?
        };

        for (index, node) in siblings.iter().enumerate() {
            if node.sort_order != index as i64 {
                let mut am: entities::category::ActiveModel = node.clone().into();
                am.sort_order = ActiveValue::Set(index as i64);
                am.update(self.db).await?;
            }
        }

        Ok(())
    }

    // 根据新的树结构重建整个数据库（用于拖拽后保存）
    pub async fn rebuild_tree_from_structure(
        &self,
        new_structure: &[TreeNodeDto],
    ) -> Result<(), DbErr> {
        info!(
            "Rebuilding tree from structure with {} root nodes",
            new_structure.len()
        );

        self.rebuild_tree_recursive(new_structure, None, 0).await?;

        info!("Tree rebuild completed");
        Ok(())
    }

    // 递归重建树结构
    async fn rebuild_tree_recursive(
        &self,
        nodes: &[TreeNodeDto],
        parent_id: Option<i64>,
        sort_order: i64,
    ) -> Result<(), DbErr> {
        for (index, node) in nodes.iter().enumerate() {
            let current_sort_order = sort_order + index as i64;

            // 更新当前节点
            let category = Category::find_by_id(node.id)
                .one(self.db)
                .await?
                .ok_or_else(|| DbErr::Custom(format!("Category {} not found", node.id)))?;

            let mut am: entities::category::ActiveModel = category.into();
            am.parent_id = ActiveValue::Set(parent_id);
            am.sort_order = ActiveValue::Set(current_sort_order);
            am.update(self.db).await?;

            info!(
                "Updated category {}: parent_id={:?}, sort_order={}",
                node.id, parent_id, current_sort_order
            );

            // 递归处理子节点
            if let Some(children) = &node.children {
                if !children.is_empty() {
                    Box::pin(self.rebuild_tree_recursive(children, Some(node.id), 0)).await?;
                }
            }
        }
        Ok(())
    }
}
