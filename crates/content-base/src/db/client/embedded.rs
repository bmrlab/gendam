use crate::db::constant::{DATABASE_NAME, DATABASE_NS};
use crate::db::sql::CREATE_TABLE;
use crate::db::DB;
use std::env;
use std::path::Path;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::opt::Config;
use surrealdb::Surreal;

impl DB {
    pub async fn new(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let client = DB::init_db(path).await.map_err(|e| {
            tracing::error!("Failed to initialize surrealdb: {}", e);
            e
        })?;
        Ok(Self { client })
    }

    async fn init_db(path: impl AsRef<Path>) -> anyhow::Result<Surreal<Db>> {
        let config = Config::default();
        let db = Surreal::new::<RocksDb>((path.as_ref(), config)).await?;
        db.use_ns(env::var(DATABASE_NS)?)
            .use_db(env::var(DATABASE_NAME)?)
            .await?;
        DB::init_table(&db).await?;
        Ok(db)
    }

    async fn init_table(db: &Surreal<Db>) -> anyhow::Result<()> {
        db.query(CREATE_TABLE).await?;
        Ok(())
    }
}
