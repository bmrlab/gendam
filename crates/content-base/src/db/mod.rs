use std::env;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use crate::db::constant::{DATABASE_HOST, DATABASE_NAME, DATABASE_NS, DATABASE_PASSWORD, DATABASE_PORT, DATABASE_USER};
use crate::db::model::id::ID;
use crate::db::model::ImageModel;
use crate::db::model::payload::PayloadModel;
use crate::db::sql::CREATE_TABLE;
use crate::query::payload::SearchPayload;

mod constant;
mod model;
mod sql;

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
    pub async fn insert_image(&self, image_model: ImageModel, payload: Option<SearchPayload>) -> anyhow::Result<()> {
        let mut res = self.client
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
        
        if let (Some(id), Some(payload)) = (id, payload) {
            let payload_id = self.create_payload(payload.into()).await?;
            self.create_with_relation(id, payload_id).await?;
        }

        Ok(())
    }
    
    async fn create_payload(&self, payload: PayloadModel) -> anyhow::Result<ID> {
        let mut res = self.client
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
            None => Err(anyhow::anyhow!("Failed to create payload"))
        }
    }

    pub async fn insert_audio(&self) {}

    pub async fn insert_video(&self) {}

    pub async fn insert_document(&self) {}

    pub async fn insert_web_page(&self) {}
}

// 关系
impl DB {
    async fn create_with_relation(&self, relation_in: ID, relation_out: ID) -> anyhow::Result<()> {
        let sql = format!(
            "RELATE {} -> with -> {};",
            relation_in.id_with_table(),
            relation_out.id_with_table(),
        );
        self.client.query(&sql).await?;
        Ok(())
    }

    async fn create_contain_relation(&self, relation_in: &str, relation_outs: Vec<&str>) -> anyhow::Result<()> {
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
    use rand::Rng;
    use content_base_task::ContentTaskType;
    use content_base_task::image::desc_embed::ImageDescEmbedTask;
    use content_base_task::image::ImageTaskType;
    use crate::db::DB;
    use crate::db::model::ImageModel;
    use crate::query::payload::{SearchMetadata, SearchPayload};
    use crate::query::payload::image::ImageSearchMetadata;

    async fn setup() -> DB {
        dotenvy::dotenv().ok();
        DB::new().await
    }

    fn gen_vector() -> Vec<f32> {
        (0..512)
            .map(|_| rand::thread_rng().gen_range(0.0..1.0))
            .collect()
    }

    #[tokio::test]
    async fn test_init_db() {
        let db = setup().await;
        println!("{:?}", db.client.query("INFO FOR DB;").await.unwrap());
    }

    #[tokio::test]
    async fn test_insert_image() {
        let db = setup().await;
        let _ = db.insert_image(ImageModel {
            prompt: "p3".to_string(),
            vector: gen_vector(),
            prompt_vector: gen_vector(),
        }, Some(SearchPayload {
            file_identifier: "file_identifier".to_string(),
            task_type: ContentTaskType::Image(ImageTaskType::DescEmbed(ImageDescEmbedTask {})),
            metadata: SearchMetadata::Image(ImageSearchMetadata {}),
        })).await;
    }
}
