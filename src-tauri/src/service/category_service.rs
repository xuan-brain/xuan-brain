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

    // 移动节点:核心逻辑是根据 targetPath + position 重新计算 ltree_path,并更新整棵子树
    pub async fn move_node(
        &self,
        dragged_path: &str,
        target_path: Option<&str>,
        position: &str, // "above" | "below" | "child"
    ) -> Result<String, DbErr> {
        println!("=== MOVE NODE START ===");
        println!("Dragged path: {}", dragged_path);
        println!("Target path: {:?}", target_path);
        println!("Position: {}", position);

        // 1. 加载所有节点,内存中构建 path -> Model 映射
        let all = Category::find()
            .order_by_asc(entities::category::Column::LtreePath)
            .all(self.db)
            .await?;
        let by_path: HashMap<String, entities::category::Model> =
            all.into_iter().map(|m| (m.ltree_path.clone(), m)).collect();

        let dragged = by_path
            .get(dragged_path)
            .ok_or_else(|| DbErr::Custom("Dragged node not found".into()))?;

        println!("Current dragged node ltree_path: {}", dragged.ltree_path);

        let new_path =
            self.calc_new_path_after_move(&by_path, dragged_path, target_path, position)?;

        println!("Calculated new path: {}", new_path);

        // 更新 dragged 及所有子孙的 ltree_path
        // 查找所有以 dragged_path 开头的节点(包括 dragged_path 本身)
        let all_for_update: Vec<entities::category::Model> = Category::find()
            .filter(
                Expr::col(entities::category::Column::LtreePath).like(format!("{}%", dragged_path)),
            )
            .all(self.db)
            .await?;

        println!(
            "Found {} nodes to update (including dragged)",
            all_for_update.len()
        );

        for mut model in all_for_update {
            let old_path = model.ltree_path.clone();
            let suffix = model.ltree_path.strip_prefix(dragged_path).unwrap_or("");
            let new_node_path = if suffix.is_empty() {
                new_path.clone()
            } else {
                format!("{}.{}", new_path, &suffix[1..]) // skip the leading dot
            };
            println!("Updating: {} -> {}", old_path, new_node_path);
            model.ltree_path = new_node_path.clone();

            // 更新到数据库
            let mut am: entities::category::ActiveModel = model.into();
            am.ltree_path = ActiveValue::Set(new_node_path);
            am.update(self.db).await?;
        }

        // 重新更新 parent_id
        let segments: Vec<&str> = new_path.split('.').collect();
        let new_parent_path = if segments.len() > 1 {
            Some(segments[..segments.len() - 1].join("."))
        } else {
            None
        };

        println!("New parent path: {:?}", new_parent_path);

        // 从数据库中查找新的父节点
        let new_parent_id = if let Some(ref parent_path) = new_parent_path {
            Category::find()
                .filter(entities::category::Column::LtreePath.eq(parent_path.as_str()))
                .one(self.db)
                .await?
                .map(|p| {
                    println!("Found parent node: {} (id: {})", p.ltree_path, p.id);
                    p.id
                })
        } else {
            println!("No parent (root node)");
            None
        };

        // 保存 dragged 本身的 parent_id 字段
        let mut am: entities::category::ActiveModel = dragged.clone().into();
        am.parent_id = ActiveValue::Set(new_parent_id);
        // 注意:不要更新 ltree_path,因为已经在上面更新过了
        am.update(self.db).await?;

        println!("Updated parent_id to: {:?}", new_parent_id);
        println!("=== MOVE NODE COMPLETE ===");

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

    // 工具:根据拖拽位置计算新的 ltree_path
    fn calc_new_path_after_move(
        &self,
        by_path: &HashMap<String, entities::category::Model>,
        _dragged_path: &str,
        target_path: Option<&str>,
        position: &str,
    ) -> Result<String, DbErr> {
        if let Some(target) = target_path {
            let target_node = by_path
                .get(target)
                .ok_or_else(|| DbErr::Custom("Target not found".into()))?;

            match position {
                "child" => {
                    // 作为 target 的子节点,找 target 下子节点的最大序号
                    let target_prefix = format!("{}.", target);
                    let max_seq = by_path
                        .keys()
                        .filter(|k| k.starts_with(&target_prefix) && k.as_str() != target)
                        .filter_map(|k| {
                            k.trim_start_matches(&target_prefix)
                                .split('.')
                                .next()
                                .and_then(|s| s.parse::<i64>().ok())
                        })
                        .max()
                        .unwrap_or(0);

                    Ok(format!("{}.{}", target, max_seq + 1))
                }
                "above" | "below" => {
                    // 获取 target 的父路径
                    let (parent_path, target_seq) = target_node
                        .ltree_path
                        .rsplit_once('.')
                        .unwrap_or(("", target_node.ltree_path.as_str()));

                    // 找到同级所有节点,确定新序号
                    let _prefix = if parent_path.is_empty() {
                        String::new()
                    } else {
                        format!("{}.", parent_path)
                    };

                    // 解析 target 的序号
                    let target_seq_num = target_seq
                        .parse::<i64>()
                        .map_err(|_| DbErr::Custom("Invalid target path format".into()))?;

                    // position == "above" 使用 target 的序号
                    // position == "below" 使用 target 的序号 + 1
                    let new_seq = if position == "above" {
                        target_seq_num
                    } else {
                        target_seq_num + 1
                    };

                    // 构建新路径
                    if parent_path.is_empty() {
                        Ok(new_seq.to_string())
                    } else {
                        Ok(format!("{}.{}", parent_path, new_seq))
                    }
                }
                _ => Err(DbErr::Custom(format!("Unsupported position: {}", position))),
            }
        } else {
            // 拖到根区域:找最大根节点 id + 1
            let max_root_id = by_path
                .keys()
                .filter(|k| !k.contains('.'))
                .filter_map(|k| k.parse::<i64>().ok())
                .max()
                .unwrap_or(0);

            Ok((max_root_id + 1).to_string())
        }
    }
}
