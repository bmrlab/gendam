#[allow(warnings, unused)]
pub mod prisma;

pub use prisma::*;

#[cfg(test)]
mod prisma_tests {
    use super::*;
    use futures::future::join_all;
    use std::sync::Arc;
    use prisma_client_rust::{
        // raw,
        QueryError,
    };
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    async fn exec(client: Arc<PrismaClient>, i: i32)
        -> Result<asset_object::Data, QueryError>
    {
        let start = std::time::Instant::now();
        // wait for random seconds
        // let millis = rand::random::<u64>() % 1000;
        let millis = 0;
        println!("executing {:<5}, wait for {:<5} ms", i, millis);
        tokio::time::sleep(tokio::time::Duration::from_millis(millis)).await;
        // let client = new_client().await.unwrap();
        let res = client
            .asset_object()
            .create(vec![
                asset_object::note::set(Some(i.to_string())),
            ])
            .exec()
            .await;
        let duration = start.elapsed();
        println!("executed {:<5}, wait for {:<5}, duration {:<5}", i, millis, duration.as_millis());
        return res;
    }

    #[tokio::test]
    async fn test_sqlite_write() {
        tracing_subscriber::registry()
            .with(
                // load filters from the `RUST_LOG` environment variable.
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info".into()),
            )
            .with(tracing_subscriber::fmt::layer().with_ansi(true))
            .init();

        let client = new_client().await.unwrap();
        client._db_push().await.expect("failed to push db");
        // clear asset objects
        client.asset_object().delete_many(vec![]).exec().await.unwrap();

        // let client = Arc::new(new_client().await.unwrap());
        let client = Arc::new(client);
        // match client._execute_raw(raw!("PRAGMA journal_mode = WAL;")).exec().await {
        //     Ok(res) => println!("res: {:?}", res),
        //     Err(err) => println!("err: {:?}", err),
        // };
        let mut x: Vec<_> = vec![];
        let start = std::time::Instant::now();
        let n = 10000;
        for i in 0..n {
            let item = exec(Arc::clone(&client), i);
            // item.await.unwrap();
            x.push(item);
        }
        join_all(x).await;
        let duration = start.elapsed();
        println!("\nSQL finished in {:?} millis\n", duration.as_millis());

        let res = client.asset_object().count(vec![]).exec().await.unwrap();
        println!("final count: {:?}", res);
        assert!(res as i32 == n);
        // let results = client.asset_object().find_many(vec![]).exec().await.unwrap();
        // println!("results:");
        // results.iter().for_each(|x| {
        //     println!("{:?}, {:?}", x.id, x.note);
        // });
    }
}
