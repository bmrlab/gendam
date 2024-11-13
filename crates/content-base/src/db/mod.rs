pub mod client;
#[allow(dead_code)]
mod constant;
#[allow(dead_code)]
pub mod model;
mod op;
mod rank;
pub mod search;
pub mod shared;
mod sql;
pub mod utils;

#[derive(Clone, Debug)]
pub struct DB {
    #[cfg(feature = "embedded-db")]
    pub client: surrealdb::Surreal<surrealdb::engine::local::Db>,

    // #[cfg(not(feature = "embedded-db"))]
    #[cfg(feature = "remote-db")]
    pub client: surrealdb::Surreal<surrealdb::engine::remote::ws::Client>,
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
