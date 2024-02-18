// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use dotenvy::dotenv;
use qdrant_client::{
    client::QdrantClientConfig,
    qdrant::{
        vectors_config,
        CreateCollection,
        Distance,
        VectorParams,
        VectorsConfig,
    }
};
use std::time::Duration;
use tauri::api::process::Command;
use tauri::api::process::CommandEvent;
use tauri::Manager;
use tracing::{debug, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    match dotenv() {
        Ok(path) => println!(".env read successfully from {}", path.display()),
        Err(e) => println!("Could not load .env file: {e}"),
    };
    init_tracing();

    let router = api_server::router::get_router();

    tauri::Builder::default()
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

            // this will send stdout of qdrant to debug log
            tauri::async_runtime::spawn(async move {
                // read events such as stdout
                while let Some(event) = rx.recv().await {
                    if let CommandEvent::Stdout(line) = event {
                        debug!("message: {}", line);
                    }
                }
            });

            // make sure collection is created
            // query collection info every seconds, until it exists
            tauri::async_runtime::spawn(async move {
                loop {
                    let client = QdrantClientConfig::from_url("http://0.0.0.0:6334")
                        .build()
                        .expect("");
                    let collection_info = client
                        .collection_info(file_handler::QDRANT_COLLECTION_NAME)
                        .await;

                    match collection_info {
                        Err(err) => {
                            debug!("collection does not exist, creating it. {}", err);
                            // create collection
                            let _ = client
                                .create_collection(&CreateCollection {
                                    collection_name: file_handler::QDRANT_COLLECTION_NAME.into(),
                                    vectors_config: Some(VectorsConfig {
                                        config: Some(vectors_config::Config::Params(VectorParams {
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
        .plugin(rspc::integrations::tauri::plugin(router, |app| {
            let local_data_dir = app.app_handle()
                .path_resolver()
                .app_local_data_dir()
                .expect("failed to find local data dir");
            let resources_dir = app.app_handle()
                .path_resolver()
                .resolve_resource("resources")
                .expect("failed to find resources dir");
            let db_dir = local_data_dir.join("db/muse-v2.db");
            let db_url = format!("file:{}", db_dir.to_str().unwrap());
            api_server::router::Ctx {
                x_demo_header: None,
                local_data_dir,
                resources_dir,
                db_url,
            }
        }))
        .invoke_handler(tauri::generate_handler![
            greet,
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

#[tauri::command]
async fn handle_video_file(app_handle: tauri::AppHandle, video_path: &str) -> Result<(), String> {
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
    .await
    .expect("failed to initialize video handler");

    debug!("video handler initialized");

    let vh = video_handler.clone();
    let frame_handle = tokio::spawn(async move {
        match vh.get_frames().await {
            Ok(_) => match vh.get_frame_content_embedding().await {
                Ok(_) => Ok(()),
                Err(e) => {
                    error!("failed to get frame content embedding: {}", e);
                    Err(e)
                }
            },
            Err(e) => {
                debug!("failed to get frames: {}", e);
                Err(e)
            }
        }
    });

    let vh = video_handler.clone();
    let audio_handle = tokio::spawn(async move {
        match vh.get_audio().await {
            Ok(_) => match vh.get_transcript().await {
                Ok(_) => {
                    let res = vh.get_transcript_embedding().await;

                    if let Err(e) = res {
                        error!("failed to get transcript embedding: {}", e);
                        Err(e)
                    } else {
                        Ok(())
                    }
                }
                Err(e) => {
                    error!("failed to get audio embedding: {}", e);
                    Err(e)
                }
            },
            Err(e) => {
                error!("failed to get audio: {}", e);
                Err(e)
            }
        }
    });

    let frame_results = frame_handle.await;
    let audio_results = audio_handle.await;

    match frame_results {
        Ok(result) => {
            if let Err(frame_err) = result {
                error!("failed to get frames: {}", frame_err);
            }
        }
        Err(err) => {
            error!("failed to get frames: (JoinError){}", err);
        }
    }

    match audio_results {
        Ok(result) => {
            if let Err(audio_err) = result {
                error!("failed to get audio: {}", audio_err);
            }
        }
        Err(err) => {
            error!("failed to get audio: (JoinError){}", err);
        }
    }

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
    .await
    .expect("failed to initialize video handler");

    let _ = video_handler.get_frames_caption().await;
    let _ = video_handler.get_frame_caption_embedding().await;

    Ok(())
}

#[tauri::command]
async fn handle_search(
    app_handle: tauri::AppHandle,
    payload: file_handler::SearchRequest,
) -> Result<Vec<file_handler::SearchResult>, ()> {
    debug!("search payload: {:?}", payload);

    let resources_dir = app_handle
        .path_resolver()
        .resolve_resource("resources")
        .unwrap();

    Ok(file_handler::handle_search(payload, resources_dir)
        .await
        .map_err(|_| ())?)
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            // load filters from the `RUST_LOG` environment variable.
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "muse_desktop=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_ansi(true))
        .init();
}
