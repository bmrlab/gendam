use crate::collect_async_results;
use crate::db::constant::{
    DATABASE_HOST, DATABASE_NAME, DATABASE_NS, DATABASE_PASSWORD, DATABASE_PORT, DATABASE_USER,
};
use crate::db::model::audio::{AudioFrameModel, AudioModel};
use crate::db::model::id::ID;
use crate::db::model::payload::PayloadModel;
use crate::db::model::video::{ImageFrameModel, VideoModel};
use crate::db::model::{ImageModel, TextModel};
use crate::db::sql::CREATE_TABLE;
use crate::query::payload::SearchPayload;
use anyhow::bail;
use std::env;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use tracing::{debug, error};

mod constant;
mod model;
mod sql;
mod utils;

#[derive(Clone)]
pub struct DB {
    pub client: Surreal<Client>,
}

/// init db
impl DB {
    pub async fn new() -> Self {
        Self {
            client: DB::init_db().await.expect("Failed to initialize database"),
        }
    }

    // TODO: read from local, and later change to embedded database.
    async fn init_db() -> anyhow::Result<Surreal<Client>> {
        let db = Surreal::new::<Ws>(format!(
            "{}:{}",
            env::var(DATABASE_HOST)?,
            env::var(DATABASE_PORT)?
        ))
        .await?;
        db.signin(Root {
            username: &env::var(DATABASE_USER)?,
            password: &env::var(DATABASE_PASSWORD)?,
        })
        .await?;
        db.use_ns(env::var(DATABASE_NS)?)
            .use_db(env::var(DATABASE_NAME)?)
            .await?;
        DB::init_table(&db).await?;
        Ok(db)
    }

    async fn init_table(db: &Surreal<Client>) -> anyhow::Result<()> {
        db.query(CREATE_TABLE).await?;
        Ok(())
    }
}

// 插入
impl DB {
    async fn insert_text(&self, text: TextModel) -> anyhow::Result<ID> {
        self.client
            .query(
                "
            (CREATE ONLY text CONTENT {
                data: $data,
                vector: $vector,
                en_data: $en_data,
                en_vector: $en_vector
            }).id",
            )
            .bind(text)
            .await?
            .take::<Option<Thing>>(0)?
            .map(|x| Ok(x.into()))
            .unwrap_or_else(|| Err(anyhow::anyhow!("Failed to insert text")))
    }

    async fn batch_insert_text(&self, texts: Vec<TextModel>) -> anyhow::Result<Vec<ID>> {
        let futures = texts
            .into_iter()
            .map(|text| self.insert_text(text))
            .collect::<Vec<_>>();
        collect_async_results!(futures)
    }

    pub async fn insert_image(
        &self,
        image_model: ImageModel,
        payload: Option<SearchPayload>,
    ) -> anyhow::Result<ID> {
        let mut res = self
            .client
            .query(
                "
                (CREATE ONLY image CONTENT {
                    prompt: $prompt,
                    vector: $vector,
                    prompt_vector: $prompt_vector
                }).id",
            )
            .bind(image_model)
            .await?;

        let id: Option<ID> = res.take::<Option<Thing>>(0)?.map(|x| x.into());

        match id {
            Some(id) => {
                if let Some(payload) = payload {
                    let payload_id = self.create_payload(payload.into()).await?;
                    self.create_with_relation(&id, &payload_id).await?;
                }
                Ok(id)
            }
            None => {
                bail!("Failed to insert image");
            }
        }
    }

    async fn batch_insert_image(&self, images: Vec<ImageModel>) -> anyhow::Result<Vec<ID>> {
        let futures = images
            .into_iter()
            .map(|image| self.insert_image(image, None))
            .collect::<Vec<_>>();
        collect_async_results!(futures)
    }

    pub async fn insert_image_frame(&self, frame: ImageFrameModel) -> anyhow::Result<ID> {
        let ids = self
            .batch_insert_image(frame.data)
            .await?
            .into_iter()
            .map(|id| id.id_with_table())
            .collect::<Vec<String>>();
        if ids.is_empty() {
            bail!("Failed to insert image frame");
        }
        let create_image_frame_sql = format!(
            "(CREATE ONLY image_frame CONTENT {{ data: [{}], start_timestamp: {}, end_timestamp: {} }}).id",
            ids.join(", "),
            frame.start_timestamp,
            frame.end_timestamp
        );
        let mut res = self.client.query(create_image_frame_sql).await?;
        match res.take::<Option<Thing>>(0)? {
            Some(id) => {
                let id: ID = id.into();
                self.create_contain_relation(
                    &id.id_with_table(),
                    ids.iter().map(|id| id.as_str()).collect(),
                )
                .await?;
                Ok(id.into())
            }
            None => Err(anyhow::anyhow!("Failed to insert image frame")),
        }
    }

    async fn batch_insert_image_frame(
        &self,
        frames: Vec<ImageFrameModel>,
    ) -> anyhow::Result<Vec<ID>> {
        let futures = frames
            .into_iter()
            .map(|frame| self.insert_image_frame(frame))
            .collect::<Vec<_>>();

        collect_async_results!(futures)
    }

    async fn create_payload(&self, payload: PayloadModel) -> anyhow::Result<ID> {
        let mut res = self
            .client
            .query(
                "
                (CREATE ONLY payload CONTENT {
                    file_identifier: $file_identifier,
                    url: $url
                }).id",
            )
            .bind(payload)
            .await?;

        match res.take::<Option<Thing>>(0)? {
            Some(id) => Ok(id.into()),
            None => Err(anyhow::anyhow!("Failed to create payload")),
        }
    }

    async fn insert_audio_frame(&self, frame: AudioFrameModel) -> anyhow::Result<ID> {
        let ids = self
            .batch_insert_text(frame.data)
            .await?
            .into_iter()
            .map(|id| id.id_with_table())
            .collect::<Vec<_>>();
        debug!("insert text ids: {:?}", ids);
        if ids.is_empty() {
            bail!("Failed to insert audio frame");
        }
        let create_audio_frame_sql = format!(
            "(CREATE ONLY audio_frame CONTENT {{ data: [{}], start_timestamp: {}, end_timestamp: {} }}).id",
            ids.join(", "),
            frame.start_timestamp,
            frame.end_timestamp
        );
        let mut res = self.client.query(create_audio_frame_sql).await?;
        match res.take::<Option<Thing>>(0)? {
            Some(id) => {
                let id: ID = id.into();
                self.create_contain_relation(
                    &id.id_with_table(),
                    ids.iter().map(|id| id.as_str()).collect(),
                )
                .await?;
                Ok(id.into())
            }
            None => Err(anyhow::anyhow!("Failed to insert audio frame")),
        }
    }

    async fn batch_insert_audio_frame(
        &self,
        frames: Vec<AudioFrameModel>,
    ) -> anyhow::Result<Vec<ID>> {
        let futures = frames
            .into_iter()
            .map(|frame| self.insert_audio_frame(frame))
            .collect::<Vec<_>>();

        collect_async_results!(futures)
    }

    pub async fn insert_audio(
        &self,
        audio: AudioModel,
        payload: SearchPayload,
    ) -> anyhow::Result<ID> {
        let ids = self
            .batch_insert_audio_frame(audio.audio_frame)
            .await?
            .into_iter()
            .map(|id| id.id_with_table())
            .collect::<Vec<String>>();
        if ids.is_empty() {
            error!("audio frame is empty");
            bail!("Failed to insert audio");
        }
        let create_audio_sql = format!(
            "(CREATE ONLY audio CONTENT {{ audio_frame: [{}] }}).id",
            ids.join(", ")
        );
        let mut res = self.client.query(create_audio_sql).await?;
        match res.take::<Option<Thing>>(0)? {
            Some(id) => {
                let id: ID = id.into();
                self.create_contain_relation(
                    &id.id_with_table(),
                    ids.iter().map(|id| id.as_str()).collect(),
                )
                .await?;
                let payload_id = self.create_payload(payload.into()).await?;
                self.create_with_relation(&id, &payload_id).await?;
                Ok(id)
            }
            None => Err(anyhow::anyhow!("Failed to insert audio")),
        }
    }

    pub async fn insert_video(
        &self,
        video: VideoModel,
        payload: SearchPayload,
    ) -> anyhow::Result<ID> {
        let image_frame_ids = self
            .batch_insert_image_frame(video.image_frame)
            .await?
            .into_iter()
            .map(|id| id.id_with_table())
            .collect::<Vec<String>>();

        let audio_frame_ids = self
            .batch_insert_audio_frame(video.audio_frame)
            .await?
            .into_iter()
            .map(|id| id.id_with_table())
            .collect::<Vec<String>>();

        let image_frame = if image_frame_ids.is_empty() {
            "image_frame: []".to_string()
        } else {
            format!("image_frame: [{}]", image_frame_ids.join(", "))
        };

        let audio_frame = if audio_frame_ids.is_empty() {
            "audio_frame: []".to_string()
        } else {
            format!("audio_frame: [{}]", audio_frame_ids.join(", "))
        };

        let sql = format!(
            "(CREATE ONLY video CONTENT {{ {}, {} }}).id",
            image_frame, audio_frame
        );

        let mut res = self.client.query(&sql).await?;
        match res.take::<Option<Thing>>(0)? {
            Some(id) => {
                let id: ID = id.into();
                self.create_contain_relation(
                    &id.id_with_table(),
                    image_frame_ids.iter().map(|id| id.as_str()).collect(),
                )
                .await?;
                let payload = self.create_payload(payload.into()).await?;
                self.create_with_relation(&id, &payload).await?;
                Ok(id)
            }
            None => Err(anyhow::anyhow!("Failed to insert video")),
        }
    }

    pub async fn insert_document(&self) {}

    pub async fn insert_web_page(&self) {}
}

// 关系
impl DB {
    async fn create_with_relation(
        &self,
        relation_in: &ID,
        relation_out: &ID,
    ) -> anyhow::Result<()> {
        let sql = format!(
            "RELATE {} -> with -> {};",
            relation_in.id_with_table(),
            relation_out.id_with_table(),
        );
        self.client.query(&sql).await?;
        Ok(())
    }

    async fn create_contain_relation(
        &self,
        relation_in: &str,
        relation_outs: Vec<&str>,
    ) -> anyhow::Result<()> {
        let sql = format!(
            "RELATE {} -> contains -> [{}];",
            relation_in,
            relation_outs.join(", "),
        );
        self.client.query(&sql).await?;
        Ok(())
    }
}

mod test {
    use crate::db::model::id::TB;
    use crate::db::model::{ImageModel, TextModel};
    use crate::db::DB;
    use crate::query::payload::image::ImageSearchMetadata;
    use crate::query::payload::{SearchMetadata, SearchPayload};
    use content_base_task::audio::trans_chunk::AudioTransChunkTask;
    use content_base_task::image::desc_embed::ImageDescEmbedTask;
    use content_base_task::image::ImageTaskType;
    use content_base_task::ContentTaskType;
    use rand::Rng;
    use test_log::test;
    use content_base_task::video::trans_chunk::VideoTransChunkTask;
    use content_base_task::video::VideoTaskType;
    use crate::query::payload::video::VideoSearchMetadata;

    async fn setup() -> DB {
        dotenvy::dotenv().ok();

        DB::new().await
    }

    fn gen_vector() -> Vec<f32> {
        (0..512)
            .map(|_| rand::thread_rng().gen_range(0.0..1.0))
            .collect()
    }

    #[test(tokio::test)]
    async fn test_init_db() {
        let db = setup().await;
        println!("{:?}", db.client.query("INFO FOR DB;").await.unwrap());
    }

    #[test(tokio::test)]
    async fn test_insert_text() {
        let id = setup()
            .await
            .insert_text(TextModel {
                data: "data".to_string(),
                vector: gen_vector(),
                en_data: "en_data".to_string(),
                en_vector: gen_vector(),
            })
            .await
            .unwrap();
        println!("{:?}", id);
        assert_eq!(id.tb(), &TB::Text);
    }

    #[test(tokio::test)]
    async fn test_insert_image() {
        let db = setup().await;
        let _ = db
            .insert_image(
                ImageModel {
                    prompt: "p3".to_string(),
                    vector: gen_vector(),
                    prompt_vector: gen_vector(),
                },
                Some(SearchPayload {
                    file_identifier: "file_identifier".to_string(),
                    task_type: ContentTaskType::Image(ImageTaskType::DescEmbed(
                        ImageDescEmbedTask {},
                    )),
                    metadata: SearchMetadata::Image(ImageSearchMetadata {}),
                }),
            )
            .await;
    }

    #[test(tokio::test)]
    async fn test_insert_audio() {
        let db = setup().await;
        let id = db
            .insert_audio(
                crate::db::model::audio::AudioModel {
                    audio_frame: vec![crate::db::model::audio::AudioFrameModel {
                        data: vec![
                            TextModel {
                                data: "data".to_string(),
                                vector: gen_vector(),
                                en_data: "en_data".to_string(),
                                en_vector: gen_vector(),
                            },
                            TextModel {
                                data: "data2".to_string(),
                                vector: gen_vector(),
                                en_data: "en_data2".to_string(),
                                en_vector: gen_vector(),
                            },
                        ],
                        start_timestamp: 0.0,
                        end_timestamp: 1.0,
                    }],
                },
                SearchPayload {
                    file_identifier: "file_identifier_audio".to_string(),
                    task_type: ContentTaskType::Audio(crate::audio::AudioTaskType::TransChunk(
                        AudioTransChunkTask {},
                    )),
                    metadata: SearchMetadata::Audio(
                        crate::query::payload::audio::AudioSearchMetadata {
                            start_timestamp: 0,
                            end_timestamp: 1,
                        },
                    ),
                },
            )
            .await
            .unwrap();
        assert_eq!(id.tb(), &TB::Audio);
    }
    
    #[test(tokio::test)]
    async fn test_insert_video() {
        let db = setup().await;
        let id = db
            .insert_video(
                crate::db::model::video::VideoModel {
                    image_frame: vec![crate::db::model::video::ImageFrameModel {
                        data: vec![
                            ImageModel {
                                prompt: "p3".to_string(),
                                vector: gen_vector(),
                                prompt_vector: gen_vector(),
                            },
                            ImageModel {
                                prompt: "p4".to_string(),
                                vector: gen_vector(),
                                prompt_vector: gen_vector(),
                            },
                        ],
                        start_timestamp: 0.0,
                        end_timestamp: 1.0,
                    }],
                    audio_frame: vec![crate::db::model::audio::AudioFrameModel {
                        data: vec![
                            TextModel {
                                data: "data".to_string(),
                                vector: gen_vector(),
                                en_data: "en_data".to_string(),
                                en_vector: gen_vector(),
                            },
                            TextModel {
                                data: "data2".to_string(),
                                vector: gen_vector(),
                                en_data: "en_data2".to_string(),
                                en_vector: gen_vector(),
                            },
                        ],
                        start_timestamp: 0.0,
                        end_timestamp: 1.0,
                    }],
                },
                SearchPayload {
                    file_identifier: "file_identifier_video".to_string(),
                    task_type: ContentTaskType::Video(VideoTaskType::TransChunk(VideoTransChunkTask {})),
                    metadata: SearchMetadata::Video(VideoSearchMetadata { 
                        start_timestamp: 0,
                        end_timestamp: 1,
                    }),
                },
            )
            .await
            .unwrap();
        assert_eq!(id.tb(), &TB::Video);
    }
}
