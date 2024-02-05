// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;
use app::file_handler;
use qdrant_client::client::QdrantClientConfig;
use qdrant_client::qdrant::vectors_config::Config;
use qdrant_client::qdrant::CreateCollection;
use qdrant_client::qdrant::Distance;
use qdrant_client::qdrant::VectorParams;
use qdrant_client::qdrant::VectorsConfig;
use serde::Serialize;
use tauri::api::process::Command;
use tauri::api::process::CommandEvent;
use tauri::Manager;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let router = api_server::router::get_router();

    tauri::Builder::default()
        .plugin(rspc::integrations::tauri::plugin(router.into(), || ()))
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }

            // start qdrant
            let local_data_dir = app
                .app_handle()
                .path_resolver()
                .app_local_data_dir()
                .expect("failed to find local data dir");
            std::fs::create_dir_all(&local_data_dir).unwrap();
            let (mut rx, _) = Command::new_sidecar("qdrant")
                .expect("failed to create `qdrant` binary command")
                .current_dir(local_data_dir)
                .spawn()
                .expect("Failed to spawn sidecar");

            tauri::async_runtime::spawn(async move {
                // read events such as stdout
                while let Some(event) = rx.recv().await {
                    if let CommandEvent::Stdout(line) = event {
                        debug!("message: {}", line);
                    }
                }
            });

            // make sure collection is created
            tauri::async_runtime::spawn(async move {
                loop {
                    let client = QdrantClientConfig::from_url("http://0.0.0.0:6334")
                        .build()
                        .expect("");
                    let collection_info = client
                        .collection_info(file_handler::QDRANT_COLLECTION_NAME)
                        .await;

                    match collection_info {
                        Err(_) => {
                            debug!("collection does not exist, creating it");
                            // create collection
                            let _ = client
                                .create_collection(&CreateCollection {
                                    collection_name: file_handler::QDRANT_COLLECTION_NAME.into(),
                                    vectors_config: Some(VectorsConfig {
                                        config: Some(Config::Params(VectorParams {
                                            size: file_handler::EMBEDDING_DIM,
                                            distance: Distance::Cosine.into(),
                                            ..Default::default()
                                        })),
                                    }),
                                    ..Default::default()
                                })
                                .await;
                        }
                        _ => break,
                    }

                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            list_files,
            handle_video_file,
            get_frame_caption,
            handle_search
        ])
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

use tokio::join;
use tracing::debug;

#[tauri::command]
async fn handle_video_file(app_handle: tauri::AppHandle, video_path: &str) -> Result<(), ()> {
    let video_handler = file_handler::video::VideoHandler::new(
        video_path,
        app_handle
            .path_resolver()
            .app_local_data_dir()
            .expect("failed to find local data dir"),
        app_handle
            .path_resolver()
            .resolve_resource("resources")
            .expect("failed to find resources dir"),
    )
    .expect("failed to initialize video handler");

    let vh = video_handler.clone();

    let frames_fut = async {
        match vh.get_frames().await {
            Ok(_) => {
                let _ = vh.get_frame_content_embedding().await;
            }
            Err(e) => {
                debug!("failed to get frames: {}", e);
            }
        };
    };

    let vh = video_handler.clone();
    let audio_fut = async {
        match vh.get_audio().await {
            Ok(_) => {
                match vh.get_transcript().await {
                    Ok(_) => {
                        let _ = vh.get_transcript_embedding().await;
                    }
                    Err(e) => {
                        debug!("failed to get audio embedding: {}", e);
                    }
                };
            }
            Err(e) => {
                debug!("failed to get audio: {}", e);
            }
        };
    };

    join!(frames_fut, audio_fut);

    Ok(())
}

#[tauri::command]
async fn get_frame_caption(app_handle: tauri::AppHandle, video_path: &str) -> Result<(), ()> {
    let video_handler = file_handler::video::VideoHandler::new(
        video_path,
        app_handle
            .path_resolver()
            .app_local_data_dir()
            .expect("failed to find local data dir"),
        app_handle
            .path_resolver()
            .resolve_resource("resources")
            .expect("failed to find resources dir"),
    )
    .expect("failed to initialize video handler");

    let _ = video_handler.get_frames_caption().await;
    let _ = video_handler.get_frame_caption_embedding().await;

    Ok(())
}

#[tauri::command]
async fn handle_search(
    app_handle: tauri::AppHandle,
    text: String,
) -> Result<Vec<file_handler::SearchResult>, ()> {
    let resources_dir = app_handle
        .path_resolver()
        .resolve_resource("resources")
        .unwrap();

    Ok(
        file_handler::handle_search(&text, resources_dir, None, None)
            .await
            .map_err(|_| ())?,
    )
}
