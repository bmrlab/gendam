use crate::db::{
    entity::SelectResultEntity,
    model::{
        id::{ID, TB},
        SelectResultModel,
    },
    DB,
};
use futures::{stream, StreamExt};
use std::convert::Into;

#[derive(Debug)]
pub struct BacktrackResult {
    /// 只包含 text 和 image 表的 ID
    pub origin_id: ID,
    /// 命中的 id
    /// - 如果 origin_id 没有 relation，则是 origin_id
    /// - 如果 origin_id 有 relation
    ///     - video 类型，则是 audio_frame、image_frame
    ///     - web 类型，则是 page
    ///     - document 类型，则是 page
    pub hit_id: Vec<ID>,
    pub result: SelectResultModel,
}

impl DB {
    /// 从 text 和 image 表中根据 id 回溯关联的实体
    /// 查询出的结果顺序是和 ids（已去重）一致的
    ///
    /// 支持的情况:
    /// 1. ids 包含了 text 和 image 表中的一条记录，没有关联关系
    ///     - 直接返回对应记录
    /// 2. ids 包含了一条关联到 video、web_page、document 的 text 或 image 记录
    ///     - video: 返回包含了 audio_frame 和 image_frame 的 video 数据
    ///     - web_page: 返回包含了 text 和 image 列表的 web_page 数据
    ///     - document: 返回包含了 text 和 image 列表的 document 数据
    ///
    /// # 参数
    /// * `ids` - 只包含 text 和 image 表的 ID 列表（已去重）
    ///
    /// # 返回
    /// * `Vec<BacktrackResult>` - 按传入的 ids 顺序返回回溯结果列表
    pub async fn backtrace_by_ids(&self, ids: Vec<ID>) -> anyhow::Result<Vec<BacktrackResult>> {
        let backtrace = stream::iter(ids)
            .then(|id| async move {
                let mut res: Vec<BacktrackResult> = vec![];
                let has_relation = self.has_contains_relation(&id).await?;
                if has_relation {
                    let backtrace_relation =
                        self.backtrace_relation(vec![id.id_with_table()]).await?;

                    for br in backtrace_relation {
                        let entity = self.select_entity_by_relation(&br.result).await?;
                        for select_entity in entity {
                            res.push(BacktrackResult {
                                origin_id: id.clone(),
                                hit_id: br.hit_id.clone(),
                                result: select_entity.into(),
                            });
                        }
                    }
                } else {
                    // 没有 contain 关系的情况
                    let data = match id.tb() {
                        TB::Text => self
                            .select_text(vec![id.id_with_table()])
                            .await?
                            .into_iter()
                            .map(SelectResultEntity::Text)
                            .collect::<Vec<SelectResultEntity>>()
                            .pop(),
                        TB::Image => self
                            .select_image(vec![id.id_with_table()])
                            .await?
                            .into_iter()
                            .map(SelectResultEntity::Image)
                            .collect::<Vec<SelectResultEntity>>()
                            .pop(),
                        _ => {
                            tracing::error!("should not be here: {:?}", id);
                            None
                        }
                    };
                    if let Some(entity) = data {
                        res.push(BacktrackResult {
                            origin_id: id.clone(),
                            hit_id: vec![id.clone()],
                            result: entity.into(),
                        });
                    }
                }
                Ok::<Vec<BacktrackResult>, anyhow::Error>(res)
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .filter_map(Result::ok)
            .flatten()
            .collect::<Vec<BacktrackResult>>();

        Ok(backtrace)
    }
}
