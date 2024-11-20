use tracing::Instrument;

use crate::check_db_error_from_resp;
use crate::db::{
    model::{
        audio::AudioModel, document::DocumentModel, image::ImageModel, video::VideoModel,
        web_page::WebPageModel, ModelDelete,
    },
    DB,
};

// const CONTENT_TYPE_LOOKUP_QUERY: &'static str = r#"
// (
//     SELECT
//         <-with[0].in.id as id
//     FROM ONLY payload
//     WHERE file_identifier = $file_identifier LIMIT 1
// ).id;
// "#;
const CONTENT_TYPE_LOOKUP_QUERY: &'static str = r#"
(
    SELECT
        <-with[0].in.id as id
    FROM payload
    WHERE file_identifier = $file_identifier
).id
"#;

impl DB {
    #[tracing::instrument(skip(self))]
    pub async fn delete_by_file_identifier(&self, file_identifier: &str) -> anyhow::Result<()> {
        // https://github.com/bmrlab/gendam/issues/110
        // 可能并不一定是删除的时候有问题，但是至少 transaction 可以确保数据一致，不会出现有些数据没删除。
        // 上面的 issue 很可能是插入数据的时候产生的，应该让插入数据串行才行
        // TODO: 还有，如果遇到 index 出问题可以考虑 rebuild index, 已验证可以解决问题

        // 开启事务
        let mut resp = self.client.query("BEGIN TRANSACTION;").await?;
        check_db_error_from_resp!(resp)
            .map_err(|errors_map| anyhow::anyhow!("begin transaction error: {:?}", errors_map))?;

        let result: anyhow::Result<()> = async {
            let mut resp = self
                .client
                .query(CONTENT_TYPE_LOOKUP_QUERY)
                .bind(("file_identifier", file_identifier.to_string()))
                .await?;
            check_db_error_from_resp!(resp).map_err(|errors_map| {
                anyhow::anyhow!("content_type lookup error: {:?}", errors_map)
            })?;
            let records = resp.take::<Vec<surrealdb::sql::Thing>>(0)?;
            tracing::info!(
                "{} records found for file_identifier: {}",
                records.len(),
                file_identifier
            );
            for record in records {
                match record.tb.as_str() {
                    "image" => {
                        ImageModel::delete_cascade(&self.client, &record).await?;
                    }
                    "audio" => {
                        AudioModel::delete_cascade(&self.client, &record).await?;
                    }
                    "video" => {
                        VideoModel::delete_cascade(&self.client, &record).await?;
                    }
                    "document" => {
                        DocumentModel::delete_cascade(&self.client, &record).await?;
                    }
                    "web" => {
                        WebPageModel::delete_cascade(&self.client, &record).await?;
                    }
                    _ => {
                        tracing::warn!("unexpected content type: {}", record.tb.as_str());
                        return Ok(());
                    }
                };
            }
            Ok(())
        }
        .instrument(tracing::Span::current())
        .await;

        match result {
            Ok(_) => {
                // 提交事务
                let mut resp = self.client.query("COMMIT TRANSACTION;").await?;
                check_db_error_from_resp!(resp).map_err(|errors_map| {
                    anyhow::anyhow!("commit transaction error: {:?}", errors_map)
                })?;
                Ok(())
            }
            Err(e) => {
                // 回滚事务
                let mut resp = self.client.query("CANCEL TRANSACTION;").await?;
                check_db_error_from_resp!(resp).map_err(|errors_map| {
                    anyhow::anyhow!("cancel transaction error: {:?}", errors_map)
                })?;
                Err(e)
            }
        }
    }
}
