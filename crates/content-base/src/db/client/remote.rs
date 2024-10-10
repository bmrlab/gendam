use crate::db::constant::{DATABASE_HOST, DATABASE_NAME, DATABASE_NS, DATABASE_PORT};
use crate::db::sql::CREATE_TABLE;
use crate::db::DB;
use std::env;
use std::path::Path;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::Config;
use surrealdb::Surreal;

impl DB {
    pub async fn new() -> Self {
        Self {
            client: DB::init_db().await.expect("Failed to initialize database"),
        }
    }

    async fn init_db() -> anyhow::Result<Surreal<Client>> {
        let db = Surreal::new::<Ws>(format!(
            "{}:{}",
            env::var(DATABASE_HOST)?,
            env::var(DATABASE_PORT)?
        ))
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
