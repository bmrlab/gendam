#[allow(warnings, unused)]
pub mod prisma;

pub use prisma::*;

#[cfg(test)]
mod prisma_tests {
    use super::*;
    use futures::future::join_all;
    use std::sync::Arc;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    async fn exec(client: Arc<PrismaClient>, i: i32)
        // -> Result<asset_object::Data, prisma_client_rust::QueryError>
    {
        let start = std::time::Instant::now();
        // wait for random seconds
        let millis = rand::random::<u64>() % 1000;
        // let millis = 0;
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
        match res {
            Ok(_) => {
                println!("executed {:<5}, wait for {:<5}, duration {:<5}", i, millis, duration.as_millis());
            }
            Err(err) => {
                println!("executed {:<5}, wait for {:<5}, duration {:<5}, error: {:?}", i, millis, duration.as_millis(), err);
            }
        };
        // return res;
    }

    #[tokio::test]
    async fn test_sqlite_write() {
        tracing_subscriber::registry()
            .with(
                // load filters from the `RUST_LOG` environment variable.
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "debug".into()),
            )
            .with(tracing_subscriber::fmt::layer().with_ansi(false))
            .init();

        let client = new_client().await.unwrap();
        client._db_push().await.expect("failed to push db");
        // match client._execute_raw(prisma_client_rust::raw!("PRAGMA journal_mode = WAL;")).exec().await {
        //     Ok(res) => println!("journal_mode res: {:?}", res),
        //     Err(err) => println!("journal_mode err: {:?}", err),
        // };
        // clear asset objects
        client.asset_object().delete_many(vec![]).exec().await.unwrap();

        // let client = Arc::new(new_client().await.unwrap());
        let client = Arc::new(client);
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

        // let results = client.asset_object().find_many(vec![]).exec().await.unwrap();
        // println!("results:");
        // results.iter().for_each(|x| {
        //     let note = x.note.clone().unwrap();
        //     // println!("{:?}, {:?}", x.id, note);
        //     println!("{:>6}", note);
        // });

        let res = client.asset_object().count(vec![]).exec().await.unwrap();
        println!("final count: {:?}", res);
        assert!(res as i32 == n);
    }
}
