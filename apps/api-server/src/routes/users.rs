use crate::CtxWithLibrary;
use rspc::{Router, RouterBuilder};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Auth {
    pub id: String,
    pub name: String,
}

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new()
        .query("get", |t| {
            t(|ctx, _input: ()| async move {
                // let res = list_users().await;
                // serde_json::to_value::<Vec<String>>(vec![]).unwrap()
                let auth_file = ctx.get_local_data_root().join(".auth");
                if let Ok(reader) = std::fs::File::open(&auth_file) {
                    let auth = serde_json::from_reader::<_, Auth>(reader);
                    if let Ok(auth) = auth {
                        return Some(auth)
                    }
                }
                return None;
            })
        })
        .mutation("set", |t| {
            t(|ctx, input: Auth| async move {
                let auth_file = ctx.get_local_data_root().join(".auth");
                let writer = std::fs::File::create(auth_file)
                    .map_err(|e| rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("Failed to create .auth file: {}", e),
                    ))?;
                serde_json::to_writer::<_, Auth>(writer, &input)
                    .map_err(|e| rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("Failed to write .auth file: {}", e),
                    ))?;
                return Ok(input);
            })
        })
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
