use std::{
    sync::Arc,
    path::PathBuf,
};
use rspc::{
    // Router,
    Rspc,
    BuiltRouter,
    ExportConfig,
};
use serde::Serialize;
use prisma_lib::user;
use prisma_lib::PrismaClient;

#[derive(Clone)]
pub struct Ctx {
    pub x_demo_header: Option<String>,
}
pub const R: Rspc<Ctx> = Rspc::new();

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

pub fn get_router() -> Arc<BuiltRouter<Ctx>> {
    let router = R.router()
        .procedure(
            "version",
            R.query(|_ctx, _input: ()| env!("CARGO_PKG_VERSION"))
        )
        .procedure(
            "users",
            R.query(|_ctx, _input: ()| async move {
                let res = list_users().await;
                serde_json::to_value(res).unwrap()
            })
        )
        .procedure(
            "files",
            R.query(|_ctx, subpath: Option<String>| async move {
                // println!("subpath: {:?}", subpath);
                let res = list_files(subpath);
                serde_json::to_value(res).unwrap()
            })
        )
        .procedure(
            "folders",
            R.query(|_ctx, _input: ()| async move {
                let res = get_folders_tree();
                serde_json::to_value(res).unwrap()
            })
        )
        .procedure("ls",
            R.query(|_ctx, full_path: String| async move {
                let res = get_files_in_path(full_path);
                serde_json::to_value(res).unwrap()
            })
        );
    let router = router.build().unwrap().arced();
    router
        .export_ts(ExportConfig::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/web/src/app/lib/bindings.ts"),
        ))
        .unwrap();
    return router;
}

#[derive(Serialize)]
struct File {
    name: String,
    is_dir: bool,
}

fn list_files(subpath: Option<String>) -> Vec<File> {
    let mut root_path = String::from("/Users/xddotcom/Downloads/local_dam_files");
    if let Some(subpath) = subpath {
        root_path = format!("{}/{}", root_path, subpath);
    }
    let paths = std::fs::read_dir(root_path).unwrap();
    let mut files = vec![];
    for path in paths {
        let file_name = path.as_ref().unwrap().file_name();
        let file_path = path.as_ref().unwrap().path();
        let file_path_str = file_name.to_str().unwrap().to_string();
        let is_dir = file_path.is_dir();
        let file = File {
            name: file_path_str,
            is_dir,
        };
        files.push(file);
    }
    files
}

fn get_files_in_path(full_path: String) -> Vec<File> {
    let result = std::fs::read_dir(full_path);
    if let Err(_) = result {
        return vec![];
    }
    let paths = result.unwrap();
    let mut files = vec![];
    for path in paths {
        let file_name = path.as_ref().unwrap().file_name();
        let file_path = path.as_ref().unwrap().path();
        let file_path_str = file_name.to_str().unwrap().to_string();
        let is_dir = file_path.is_dir();
        let file = File {
            name: file_path_str,
            is_dir,
        };
        files.push(file);
    }
    files
}

fn get_folders_tree() -> Vec<File> {
    vec![]
}
