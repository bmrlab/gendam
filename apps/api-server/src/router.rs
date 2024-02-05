use rspc::Router;

use prisma_lib::user;
use prisma_lib::PrismaClient;

async fn list_users() -> Vec<user::Data> {
    let client = PrismaClient::_builder().build().await.unwrap();
    let result: Vec<user::Data> = client
        .user()
        .find_many(vec![user::id::equals(1)])
        .exec()
        .await
        .unwrap();
    result
}

pub fn get_router() -> Router {
    let router = <Router>::new()
        .query("version", |t| {
            t(|_ctx, _input: ()| env!("CARGO_PKG_VERSION"))
        })
        .query("users", |t| {
            t(|_ctx, _input: ()| async move {
                let res = list_users().await;
                serde_json::to_value(res).unwrap()
            })
        })
        .build();
    return router;
}
