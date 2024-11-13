use crate::check_db_error_from_resp;
use crate::db::entity::relation::RelationEntity;
use crate::db::model::id::{ID, TB};
use crate::db::DB;
use anyhow::bail;
use async_recursion::async_recursion;
use tracing::error;

impl DB {
    pub async fn delete_by_file_identifier(&self, file_identifier: &str) -> anyhow::Result<()> {
        let records = self
            .select_record_by_file_identifier(file_identifier)
            .await?;
        for record in records {
            self.delete(&record).await?;
        }
        Ok(())
    }

    /// - 传入顶层元素，比如 video，则会递归删除子元素
    ///
    /// - 如果传入底层元素，比如 text，则不会向上删除，只会删除 text 本身
    ///
    /// - 如果有 contain 关系将被删除
    ///
    /// - 如果有 with 关系也会被删除，对应的 payload 也会被同时删除
    ///
    /// 删除流程
    /// 1. 查询是否有 with 关系
    ///     - 如果有，删除 with 关系
    ///     - 删除 payload
    /// 2. 删除元素
    ///
    /// tips：级联删除暂不支持
    /// https://github.com/surrealdb/surrealdb/issues/1374
    pub async fn delete(&self, id: &ID) -> anyhow::Result<()> {
        match id.tb() {
            TB::Text => self.delete_text(id).await,
            TB::Image => self.delete_image(id).await,
            TB::ImageFrame => self.delete_image_frame(id).await,
            TB::AudioFrame => self.delete_audio_frame(id).await,
            TB::Audio => self.delete_audio(id).await,
            TB::Video => self.delete_video(id).await,
            TB::Page => self.delete_page(id).await,
            TB::Web => self.delete_web(id).await,
            TB::Document => self.delete_document(id).await,
            TB::Payload => self.delete_payload(id).await,
            _ => bail!("not implemented"),
        }
    }

    /// - `with` 关系会被删除
    ///     - 如果有 `with` 关系，对应的 payload 也会被删除（payload 关系是一对一的）
    /// - `contain` 关系会被删除，并且子元素也将被递归删除（如果子元素有 `contain` 也会被删除）
    async fn delete_text(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_with_relation_and_payload_by_id(id).await?;
        self.delete_by_id_with_table(id.id_with_table().as_str())
            .await
    }

    async fn delete_image(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_with_relation_and_payload_by_id(id).await?;
        self.delete_by_id_with_table(id.id_with_table().as_str())
            .await
    }

    /// 1. 查找 image frame 的 contain 关系
    /// 2. 删除 contain 关系
    /// 3. 删除子 record 记录
    /// 4. 删除 image frame
    #[async_recursion]
    async fn delete_image_frame(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_contain_relation_and_subrecord_by_id(id).await?;
        self.delete_by_id_with_table(id.id_with_table().as_str())
            .await
    }

    #[async_recursion]
    async fn delete_audio_frame(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_contain_relation_and_subrecord_by_id(id).await?;
        self.delete_by_id_with_table(id.id_with_table().as_str())
            .await
    }

    async fn delete_audio(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_contain_relation_and_subrecord_by_id(id).await?;
        self.delete_with_relation_and_payload_by_id(id).await?;
        self.delete_by_id_with_table(id.id_with_table().as_str())
            .await
    }

    async fn delete_video(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_contain_relation_and_subrecord_by_id(id).await?;
        self.delete_with_relation_and_payload_by_id(id).await?;
        self.delete_by_id_with_table(id.id_with_table().as_str())
            .await
    }

    #[async_recursion]
    async fn delete_page(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_contain_relation_and_subrecord_by_id(id).await?;
        self.delete_by_id_with_table(id.id_with_table().as_str())
            .await
    }

    async fn delete_web(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_contain_relation_and_subrecord_by_id(id).await?;
        self.delete_with_relation_and_payload_by_id(id).await?;
        self.delete_by_id_with_table(id.id_with_table().as_str())
            .await
    }

    async fn delete_document(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_contain_relation_and_subrecord_by_id(id).await?;
        self.delete_with_relation_and_payload_by_id(id).await?;
        self.delete_by_id_with_table(id.id_with_table().as_str())
            .await
    }

    /// id: payload id
    async fn delete_payload(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_by_id_with_table(id.id_with_table().as_str())
            .await
    }

    async fn batch_delete_payload(&self, ids: Vec<&ID>) -> anyhow::Result<()> {
        for id in ids {
            if let Err(e) = self.delete_payload(id).await {
                error!("failed to delete payload {id:?}: {e}");
            }
        }
        Ok(())
    }
}

/// 删除 relation
impl DB {
    async fn delete_contain_relation(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_by_id_with_table(id.id_with_table().as_str())
            .await
    }

    async fn delete_with_relation(&self, id: &ID) -> anyhow::Result<()> {
        self.delete_by_id_with_table(id.id_with_table().as_str())
            .await
    }

    /// 通用的删除
    /// 删除 relation 返回的 id 是 Id 类型，而不是 Thing 类型
    async fn delete_by_id_with_table(&self, id: &str) -> anyhow::Result<()> {
        let mut resp = self.client.query(format!("DELETE {};", id)).await?;
        check_db_error_from_resp!(resp).map_err(|errors_map| {
            error!(
                "delete_by_id_with_table id: {} errors: {:?}",
                id, errors_map
            );
            anyhow::anyhow!("Failed to delete id: {} errors: {:?}", id, errors_map)
        })
    }

    async fn batch_delete_with_relation(&self, ids: Vec<ID>) -> anyhow::Result<()> {
        for id in ids {
            if let Err(e) = self.delete_with_relation(&id).await {
                error!("failed to delete with relation {id:?}: {e}");
            }
        }
        Ok(())
    }

    /// - 删除 with relation 和 payload
    ///
    /// id: video、audio、document、web、text、image
    async fn delete_with_relation_and_payload_by_id(&self, id: &ID) -> anyhow::Result<()> {
        let relations = self.select_with_relation_by_in(id).await?;
        let with_ids = relations
            .iter()
            .map(|x| ID::from(x.id().as_str()))
            .collect::<Vec<ID>>();
        let payload_ids = relations
            .iter()
            .map(|x| ID::from(x.out_id().as_str()))
            .collect::<Vec<ID>>();
        self.batch_delete_with_relation(with_ids).await?;
        self.batch_delete_payload(payload_ids.iter().collect())
            .await?;
        Ok(())
    }

    /// - 删除 contain relation 和 subrecord
    ///
    /// id: video、audio、document、web、page、image_frame、audio_frame
    async fn delete_contain_relation_and_subrecord_by_id(&self, id: &ID) -> anyhow::Result<()> {
        let relations = self.select_contains_relation_by_in(id).await?;
        for relation in relations {
            self.delete_contain_relation_and_subrecord_by_relation(&relation)
                .await?;
        }
        Ok(())
    }

    /// 根据 relation 来删除 contain 和 subrecord
    ///
    /// subrecord 不可能是 video、audio、document、web 这些顶层元素
    async fn delete_contain_relation_and_subrecord_by_relation(
        &self,
        relation: &RelationEntity,
    ) -> anyhow::Result<()> {
        let subrecord_id = ID::from(relation.out_id().as_str());

        // 删除 contain relation
        self.delete_contain_relation(&ID::from(relation.id().as_str()))
            .await?;

        // 处理 subrecord
        match subrecord_id.tb() {
            TB::Text => {
                self.delete_text(&subrecord_id).await?;
            }
            TB::Image => {
                self.delete_image(&subrecord_id).await?;
            }
            TB::ImageFrame => {
                self.delete_image_frame(&subrecord_id).await?;
            }
            TB::AudioFrame => {
                self.delete_audio_frame(&subrecord_id).await?;
            }
            TB::Page => {
                self.delete_page(&subrecord_id).await?;
            }
            _ => bail!("can not reach here"),
        }
        Ok(())
    }
}

#[allow(unused)]
mod test {
    use crate::db::model::audio::AudioModel;
    use crate::db::model::id::{ID, TB};
    use crate::db::shared::test::{
        fake_audio_frame_model, fake_audio_model, fake_file_identifier, fake_image_model,
        fake_text_model, fake_video_model, setup,
    };
    use crate::db::DB;

    async fn local_db() -> DB {
        setup(Some(r#"/Users/zingerbee/Library/Application Support/ai.gendam.desktop/libraries/9058216e-7361-48eb-9385-fd5b2f9d044a/surreal"#.as_ref())).await
    }

    #[tokio::test]
    async fn test_delete_contains_relation() {
        let db = local_db().await;
        let res = db
            .delete_contain_relation(&ID::from("contains:x7bb2jx7oqw6aj7fbpm2"))
            .await;
        assert!(res.is_ok())
    }

    #[tokio::test]
    async fn test_delete_text() {
        let db = local_db().await;
        let text = db.insert_text(None, fake_text_model()).await.unwrap();
        db.delete(&text).await.expect("delete text");
        let select_text_res = db.select_text(vec![&text.id_with_table()]).await.unwrap();
        assert!(select_text_res.is_empty());
    }

    #[tokio::test]
    async fn test_delete_image() {
        let db = local_db().await;
        let image = db
            .insert_image(Some(fake_file_identifier()), fake_image_model())
            .await
            .unwrap();
        db.delete(&image).await.expect("delete image");
        let select_image_res = db.select_image(vec![&image.id_with_table()]).await.unwrap();
        assert!(select_image_res.is_empty());
    }

    #[tokio::test]
    async fn test_delete_audio() {
        let db = local_db().await;
        let audio = db
            .insert_audio(fake_file_identifier(), fake_audio_model())
            .await
            .unwrap();
        db.delete(&audio).await.expect("delete audio");
        let select_audio_res = db.select_audio(vec![&audio.id_with_table()]).await.unwrap();
        assert!(select_audio_res.is_empty());
    }

    #[tokio::test]
    async fn test_delete_video() {
        let db = local_db().await;
        let video_id = db
            .insert_video(fake_file_identifier(), fake_video_model())
            .await
            .unwrap();
        println!("video_id: {:?}", video_id);
        let payload = db.select_payload_by_id(video_id.clone()).await.unwrap();
        println!("payload: {:?}", payload);
        // image_frame audio_frame, relation_id
        let subrecords = db
            .select_contains_relation_by_in(&video_id)
            .await
            .unwrap()
            .into_iter()
            .map(|x| (x.out_id(), x.id()))
            .collect::<Vec<(String, String)>>();
        println!("subrecords: {:?}", subrecords);

        let subrecord_ids = subrecords
            .iter()
            .map(|x| ID::from(x.0.as_str()))
            .collect::<Vec<ID>>();
        let subrecord_relation_ids = subrecords
            .iter()
            .map(|x| ID::from(x.1.as_str()))
            .collect::<Vec<ID>>();
        println!("subrecord_ids: {:?}", subrecord_ids);
        println!("subrecord_relation_ids: {:?}", subrecord_relation_ids);

        // text image
        let mut base_records = vec![];
        for subrecord_id in subrecord_ids.clone() {
            db.select_contains_relation_by_in(&subrecord_id)
                .await
                .unwrap()
                .into_iter()
                .for_each(|x| base_records.push(x.out_id()));
        }
        println!("base_records: {:?}", base_records);
        let base_record_ids = base_records
            .iter()
            .map(|x| ID::from(x.as_str()))
            .collect::<Vec<ID>>();
        println!("base_record_ids: {:?}", base_record_ids);
        let base_relation_ids = base_records
            .iter()
            .map(|x| ID::from(x.as_str()))
            .collect::<Vec<ID>>();
        println!("base_relation_ids: {:?}", base_relation_ids);

        // delete video
        db.delete(&video_id).await.unwrap();

        // check record exist
        // - video
        // - payload
        // - subrecords
        // - subrecord relations
        // - base records
        // - base relations
        let video_id_string = video_id.id_with_table();
        let video_res = db.select_video(vec![video_id_string]).await.unwrap();
        assert!(video_res.is_empty());

        let payload_id_string = payload.id.expect("payload id").id_with_table();
        let payload_res = db.select_payload(vec![payload_id_string]).await.unwrap();
        assert!(payload_res.is_empty());
    }
}
