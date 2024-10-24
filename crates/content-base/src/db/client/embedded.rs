// use crate::db::constant::{DATABASE_NAME, DATABASE_NS};
// use std::env;
use crate::db::sql::CREATE_TABLE;
use crate::db::DB;
use std::path::Path;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::opt::Config;
use surrealdb::Surreal;

// database namespace and name in embedded mode are fixed
// namespace 使用 gendam-library 和 sqlite 的名字 gendam-library.db 保持命名方式一致
// database name 使用 search-store 表示存储的是用于搜索的数据
const DATABASE_NS_VALUE: &'static str = "gendam-library";
const DATABASE_NAME_VALUE: &'static str = "search-store";

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
        db.use_ns(DATABASE_NS_VALUE)
            .use_db(DATABASE_NAME_VALUE)
            .await?;
        // .use_ns(env::var(DATABASE_NS)?)
        // .use_db(env::var(DATABASE_NAME)?)
        DB::init_table(&db).await?;
        Ok(db)
    }

    async fn init_table(db: &Surreal<Db>) -> anyhow::Result<()> {
        db.query(CREATE_TABLE).await?;
        Ok(())
    }
}
