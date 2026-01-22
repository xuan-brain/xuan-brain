use sea_orm::prelude::Expr;
use sea_orm::*;
use std::collections::HashMap;

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

        // 2. 决定新的 ltree_path
        //  - 如果没有父节点，作为新的根：找 max(id) + 1，path = id.toString
        //  - 如果有父节点，找父节点的所有子节点中 max(路径最后一段) + 1，path = parent_path + "." + newSeq
        let new_path = self.calc_new_path(parent_opt.as_ref()).await?;

        // 3. 插入新记录
        let active = entities::category::ActiveModel {
            id: ActiveValue::NotSet,
            name: ActiveValue::Set(name.into()),
            parent_id: ActiveValue::Set(parent_opt.map(|p| p.id)),
            ltree_path: ActiveValue::Set(new_path.clone()),
            sort_order: ActiveValue::Set(0),
            created_at: ActiveValue::Set(chrono::Local::now().to_rfc3339()), // 或其他时间格式
        };
        active.insert(self.db).await?;
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

    // 移动节点：核心逻辑是根据 targetPath + position 重新计算 ltree_path，并更新整棵子树
    pub async fn move_node(
        &self,
        dragged_path: &str,
        target_path: Option<&str>,
        position: &str, // "above" | "below" | "child"
    ) -> Result<String, DbErr> {
        // 1. 加载所有节点，内存中构建 path -> Model 映射
        let all = Category::find()
            .order_by_asc(entities::category::Column::LtreePath)
            .all(self.db)
            .await?;
        let by_path: HashMap<String, entities::category::Model> =
            all.into_iter().map(|m| (m.ltree_path.clone(), m)).collect();

        let dragged = by_path
            .get(dragged_path)
            .ok_or_else(|| DbErr::Custom("Dragged node not found".into()))?;

        let new_path =
            self.calc_new_path_after_move(&by_path, dragged_path, target_path, position)?;

        // 更新 dragged 及所有子孙的 ltree_path
        let prefix_to_update = dragged_path.to_string() + "."; // 避免匹配到 "1.10" 这种
        let all_for_update: Vec<entities::category::Model> = Category::find()
            .filter(
                Expr::col(entities::category::Column::LtreePath)
                    .like(format!("{}%", prefix_to_update)),
            )
            .all(self.db)
            .await?;

        for mut model in all_for_update {
            let suffix = model.ltree_path.strip_prefix(dragged_path).unwrap_or("");
            let new_node_path = if suffix.is_empty() {
                new_path.clone()
            } else {
                format!("{}.{}", new_path, suffix)
            };
            model.ltree_path = new_node_path;
            // 更新到数据库
            let mut am: entities::category::ActiveModel = model.into();
            am.ltree_path = ActiveValue::Set(am.ltree_path.unwrap());
            am.update(self.db).await?;
        }

        // 重新更新 parent_id（可选，根据新 path 的前段）
        let segments: Vec<&str> = new_path.split('.').collect();
        let new_parent_path = if segments.len() > 1 {
            Some(segments[..segments.len() - 1].join("."))
        } else {
            None
        };

        // 从数据库中查找新的父节点（因为 by_path 包含的是旧路径，新父路径可能不在其中）
        let new_parent_id = if let Some(ref parent_path) = new_parent_path {
            Category::find()
                .filter(entities::category::Column::LtreePath.eq(parent_path.as_str()))
                .one(self.db)
                .await?
                .map(|p| p.id)
        } else {
            None
        };

        // 保存 dragged 本身（parent_id 字段）
        let mut am: entities::category::ActiveModel = dragged.clone().into();
        am.parent_id = ActiveValue::Set(new_parent_id);
        am.update(self.db).await?;

        Ok(new_path)
    }

    // 工具：计算新建节点的 ltree_path
    async fn calc_new_path(
        &self,
        parent_opt: Option<&entities::category::Model>,
    ) -> Result<String, DbErr> {
        if let Some(parent) = parent_opt {
            // 找到 parent 下所有子节点，解析 path 最后一段，取 max + 1
            let children = Category::find()
                .filter(entities::category::Column::ParentId.eq(parent.id))
                .all(self.db)
                .await?;

            let max_seq = children
                .iter()
                .filter_map(|c| c.ltree_path.rsplit('.').next())
                .filter_map(|s| s.parse::<i64>().ok())
                .max()
                .unwrap_or(0);

            Ok(format!("{}.{}", parent.ltree_path, max_seq + 1))
        } else {
            // 没有父节点：找最大 id + 1
            let max_id = Category::find()
                .order_by_desc(entities::category::Column::Id)
                .one(self.db)
                .await?
                .map(|m| m.id)
                .unwrap_or(0);

            Ok((max_id + 1).to_string())
        }
    }

    // 工具：根据拖拽位置计算新的 ltree_path（简化示例）
    fn calc_new_path_after_move(
        &self,
        by_path: &HashMap<String, entities::category::Model>,
        _dragged_path: &str,
        target_path: Option<&str>,
        position: &str,
    ) -> Result<String, DbErr> {
        // 这里给一个简化版本：
        // - position == "child"：放到 target 之下（没有同级排序逻辑）
        // - position == "above"/"below"：放到 target 的父节点下，同级调整（暂不精确排序）

        if let Some(target) = target_path {
            let target_node = by_path
                .get(target)
                .ok_or_else(|| DbErr::Custom("Target not found".into()))?;

            match position {
                "child" => {
                    // 简化：作为 target 的最后一个子节点
                    let last_child_seq = 0; // 实际应查询 target 的子节点最大 path 段
                    Ok(format!("{}.{}", target, last_child_seq + 1))
                }
                "above" | "below" => {
                    // 插入到 target 的父节点之下，同级里排在 target 之前或之后
                    // 简化：这里直接用 target 的父路径作为前缀，再根据你自己的业务逻辑计算 seq
                    // 真实项目中需要考虑排序字段，对同级节点做重新编号
                    let parent_path = target_node.ltree_path.rsplit_once('.').map(|(p, _)| p);
                    let prefix = parent_path.unwrap_or("");
                    // 这里只是示例，实际上你要算出一个新的 seq 号，避免冲突
                    Ok(format!("{}.{}", prefix, 999)) // 占位，仅演示结构
                }
                _ => Err(DbErr::Custom(format!("Unsupported position: {}", position))),
            }
        } else {
            // 拖到根区域：作为根节点
            Ok((by_path.len() + 1).to_string()) // 极简示例
        }
    }
}
