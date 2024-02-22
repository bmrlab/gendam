use std::path::PathBuf;

use content_library::Library;
use rspc::Router;
use serde_json::json;
// use prisma_lib::user;
// use prisma_lib::PrismaClient;
use crate::{Ctx, R};

pub fn get_routes() -> Router<Ctx> {
    R.router()
        .procedure(
            "list",
            R.query(|ctx, _input: ()| async move {
                let res = list_libraries(&ctx.local_data_root);
                serde_json::to_value::<Vec<String>>(res).unwrap()
            })
        )
        .procedure(
            "create",
            R.mutation(|ctx, title: String| async move {
                let library = create_library(&ctx.local_data_root, title).await;
                json!({
                    "id": library.id,
                    "dir": library.dir,
                    // "artifacts_dir": library.artifacts_dir,
                    // "index_dir": library.index_dir,
                    // "db_url": library.db_url,
                })
            })
        )
}

fn list_libraries(local_data_root: &PathBuf) -> Vec<String> {
    match local_data_root.join("libraries").read_dir() {
        Ok(entries) => {
            let mut res = vec![];
            for entry in entries {
                match entry.as_ref() {
                    Ok(entry) => {
                        let path = entry.path();
                        let file_name = entry.file_name();
                        if path.is_dir() {
                            let file_name = file_name.to_str().unwrap().to_string();
                            res.push(file_name);
                        }
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                        continue;
                    }
                };
            }
            res
        }
        Err(e) => {
            println!("Error: {:?}", e);
            vec![]
        }
    }
}

async fn create_library(local_data_root: &PathBuf, title: String) -> Library {
    let library = content_library::create_library_with_title(local_data_root, title).await;
    return library;
}
