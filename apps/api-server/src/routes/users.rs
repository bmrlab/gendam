use rspc::Router;
use prisma_lib::user;
use prisma_lib::PrismaClient;
use crate::{Ctx, R};

pub fn get_routes() -> Router<Ctx> {
    R.router()
        .procedure(
            "list",
            R.query(|_ctx, _input: ()| async move {
                let res = list_users().await;
                serde_json::to_value(res).unwrap()
            })
        )
}

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
