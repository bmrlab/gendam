// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use serde::Serialize;
use rspc::Router;

#[tokio::main]
async fn main() {
    let router = <Router>::new()
        .query("version", |t| t(|ctx, input: ()| env!("CARGO_PKG_VERSION")))
        .build();

    tauri::Builder::default()
        .plugin(rspc::integrations::tauri::plugin(router.into(), || ()))
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, list_files,list_users,])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn greet(name: &str) -> String {
    println!("Hello, {}, from Server!", name);
    format!("Hello, {}, in Client!", name)
}

#[derive(Serialize)]
struct File {
    name: String,
    is_dir: bool,
}

#[tauri::command]
fn list_files(subpath: Option<String>) -> Vec<File> {
    let mut root_path = String::from("/Users/xddotcom/Downloads/local dam files");
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

mod prisma;
use prisma::PrismaClient;
use prisma::user;

#[tauri::command]
async fn list_users() -> Vec<user::Data> {
    let client = PrismaClient::_builder().build().await.unwrap();
    let result: Vec<user::Data> = client.user().find_many(vec![user::id::equals(1)]).exec().await.unwrap();
    result
}
