use futures::{stream, StreamExt};
use itertools::Itertools;

use crate::db::{
    entity::{relation::RelationEntity, ImageEntity, SelectResultEntity, TextEntity},
    model::id::{ID, TB},
    DB,
};

/// audio -> audio_frame -> text
/// 当 TB 在 MIDDLE_LAYER_LIST 中时，需要继续向上查询一层，才是最终的结果
const MIDDLE_LAYER_LIST: [TB; 3] = [TB::AudioFrame, TB::ImageFrame, TB::Page];

// 数据查询
impl DB {
    /// ids: 只包含 text 和 image 表的 ID
    /// ids 是去重的
    /// 查询出的结果顺序是和 ids 一致的
    pub async fn select_by_id(&self, ids: Vec<ID>) -> anyhow::Result<Vec<SelectResultEntity>> {
        let mut backtrack = vec![];
        stream::iter(ids)
            .then(|id| async move {
                let mut res = vec![];
                let relation = self
                    .select_relation_by_out(vec![id.id_with_table()])
                    .await?
                    .into_iter()
                    .map(|r| r.in_id())
                    .collect::<Vec<_>>();
                if !relation.is_empty() {
                    // TODO: 有 contain 关系的情况
                    // let item = self.select_item(deduplicate(relation)).await?;
                    // res.push(
                    //     item.into_iter()
                    //         .map(SelectResultEntity::Item)
                    //         .collect::<Vec<SelectResultEntity>>(),
                    // );
                } else {
                    // 没有 contain 关系的情况
                    match id.tb() {
                        TB::Text => {
                            let text = self.select_text(vec![id.id()]).await?;
                            res.push(
                                text.into_iter()
                                    .map(SelectResultEntity::Text)
                                    .collect::<Vec<SelectResultEntity>>(),
                            );
                        }
                        TB::Image => {
                            let image = self.select_image(vec![id.id()]).await?;
                            res.push(
                                image
                                    .into_iter()
                                    .map(SelectResultEntity::Image)
                                    .collect::<Vec<SelectResultEntity>>(),
                            );
                        }
                        _ => {}
                    }
                }
                Ok::<Vec<SelectResultEntity>, anyhow::Error>(res.into_iter().flatten().collect())
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .for_each(|res| match res {
                Ok(res) => {
                    backtrack.push(res);
                }
                _ => {}
            });

        Ok(backtrack.into_iter().flatten().collect())
    }

    /// 检查 id 是否存在 contain 关系
    /// 如果是多层 contain，则返回最顶层的 id
    /// page、audio_frame、image_frame 都是中间层，还需要向上查询
    async fn select_relation_by_out(
        &self,
        ids: Vec<impl AsRef<str>>,
    ) -> anyhow::Result<Vec<RelationEntity>> {
        let mut result: Vec<Vec<RelationEntity>> = vec![];
        stream::iter(ids)
            .then(|id| async move {
                let mut resp = self
                    .client
                    .query(format!(
                        "SELECT * from contains where out = {};",
                        id.as_ref()
                    ))
                    .await?;
                let result = resp.take::<Vec<RelationEntity>>(0)?;
                Ok::<_, anyhow::Error>(result)
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .for_each(|res| match res {
                Ok(relations) => {
                    result.push(relations);
                }
                _ => {}
            });

        // 目前可以确定只有一层 contain 关系
        // 如果以后有多层 contain 关系，可以递归
        let futures = result
            .into_iter()
            .flatten()
            .map(|r| async move {
                match r.in_table() {
                    tb if MIDDLE_LAYER_LIST.contains(&tb) => {
                        let mut resp = self
                            .client
                            .query(format!("SELECT * from contains where out = {};", r.in_id()))
                            .await?;
                        let result = resp.take::<Vec<RelationEntity>>(0)?;
                        Ok::<_, anyhow::Error>(result)
                    }
                    _ => Ok::<_, anyhow::Error>(vec![r]),
                }
            })
            .collect::<Vec<_>>();

        let futures_result: Vec<Vec<RelationEntity>> = stream::iter(futures)
            .buffered(1)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .filter(Result::is_ok)
            .try_collect()?;

        Ok(futures_result
            .into_iter()
            .flatten()
            .collect::<Vec<RelationEntity>>())
    }

    async fn select_text(&self, ids: Vec<impl AsRef<str>>) -> anyhow::Result<Vec<TextEntity>> {
        let mut result = vec![];

        stream::iter(ids)
            .then(|id| async move {
                let mut resp = self
                    .client
                    .query(format!("SELECT * FROM {};", id.as_ref()))
                    .await?;
                let result = resp.take::<Vec<TextEntity>>(0)?;
                Ok::<_, anyhow::Error>(result)
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .for_each(|res| match res {
                Ok(image) => {
                    result.push(image);
                }
                _ => {}
            });
        Ok(result.into_iter().flatten().collect())
    }

    async fn select_image(&self, ids: Vec<impl AsRef<str>>) -> anyhow::Result<Vec<ImageEntity>> {
        let mut result = vec![];

        stream::iter(ids)
            .then(|id| async move {
                let mut resp = self
                    .client
                    .query(format!("SELECT * FROM {};", id.as_ref()))
                    .await?;
                let result = resp.take::<Vec<ImageEntity>>(0)?;
                Ok::<_, anyhow::Error>(result)
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .for_each(|res| match res {
                Ok(image) => {
                    result.push(image);
                }
                _ => {}
            });
        Ok(result.into_iter().flatten().collect())
    }
}
