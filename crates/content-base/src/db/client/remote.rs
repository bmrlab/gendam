use crate::db::constant::{
    DATABASE_HOST, DATABASE_NAME, DATABASE_NS, DATABASE_PASSWORD, DATABASE_PORT, DATABASE_USER,
};
use crate::db::sql::CREATE_TABLE;
use crate::db::DB;
use std::env;
// use std::path::Path;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
// use surrealdb::opt::Config;
use surrealdb::Surreal;

impl DB {
    pub async fn new() -> anyhow::Result<Self> {
        let client = Self::init_db().await.map_err(|e| {
            tracing::error!("Failed to initialize surrealdb: {}", e);
            e
        })?;
        Ok(Self { client })
    }

    async fn init_db() -> anyhow::Result<Surreal<Client>> {
        let db = Surreal::new::<Ws>(format!(
            "{}:{}",
            env::var(DATABASE_HOST)?,
            env::var(DATABASE_PORT)?
        ))
        .await?;
        db.signin(Root {
            username: env::var(DATABASE_USER)?.as_str(),
            password: env::var(DATABASE_PASSWORD)?.as_str(),
        })
        .await?;

        db.use_ns(env::var(DATABASE_NS)?)
            .use_db(env::var(DATABASE_NAME)?)
            .await?;
        Self::init_table(&db).await?;
        Ok(db)
    }

    async fn init_table(db: &Surreal<Client>) -> anyhow::Result<()> {
        db.query(CREATE_TABLE).await?;
        Ok(())
    }
}
