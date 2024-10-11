use surrealdb::Surreal;

pub mod client;
mod constant;
pub mod entity;
pub mod model;
mod op;
pub mod search;
pub mod shared;
mod sql;
pub mod utils;

#[derive(Clone, Debug)]
pub struct DB {
    #[cfg(feature = "embedded-db")]
    pub client: Surreal<surrealdb::engine::local::Db>,

    #[cfg(not(feature = "embedded-db"))]
    pub client: Surreal<surrealdb::engine::remote::ws::Client>,
}

#[allow(unused_imports, dead_code)]
mod test {
    use crate::db::shared::test::setup;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_init_db() {
        let db = setup(None).await;
        println!("{:?}", db.client.query("INFO FOR DB;").await.unwrap());
    }
}
