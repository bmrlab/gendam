use rspc::{Rspc, Router};
// use crate::{Ctx, R};
use crate::CtxWithLibrary;

pub fn get_routes<TCtx>() -> Router<TCtx>
where TCtx: CtxWithLibrary + Clone + Send + Sync + 'static
{
    Rspc::<TCtx>::new().router()
        .procedure(
            "list",
            Rspc::<TCtx>::new().query(|_ctx, _input: ()| async move {
                // let res = list_users().await;
                serde_json::to_value::<Vec<String>>(vec![]).unwrap()
            })
        )
}

// async fn list_users() -> Vec<user::Data> {
//     let client = PrismaClient::_builder().build().await.unwrap();
//     let result: Vec<user::Data> = client
//         .user()
//         .find_many(vec![user::id::equals(1)])
//         .exec()
//         .await
//         .unwrap();
//     result
// }
