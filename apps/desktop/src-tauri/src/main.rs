// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use api_server::{ctx::default::Ctx, CtxWithLibrary};
use dotenvy::dotenv;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::{
    http::{
        header::{HeaderValue, CONTENT_TYPE},
        Request, ResponseBuilder,
    },
    AppHandle, Manager,
};
use vector_db::kill_qdrant_server;
mod store;
use storage::Storage;
use store::Store;
mod protocol;

#[tokio::main]
async fn main() {
    match dotenv() {
        Ok(path) => println!(".env read successfully from {}", path.display()),
        Err(e) => println!("Could not load .env file: {e}"),
    };

    let app = tauri::Builder::default()
        .register_uri_scheme_protocol("storage", move |app: &AppHandle, request: &Request| {
            // 处理请求并返回响应
            dbg!(&request);

            let decode_path = percent_encoding::percent_decode(request.uri().as_bytes())
                .decode_utf8_lossy()
                .to_string();

            let path = decode_path.split("storage://localhost/").last().unwrap();
            dbg!(&path);
            let part_split: Vec<&str> = path.split('/').collect();

            // 找到 "artifacts" 的索引
            if let Some(index) = part_split
                .iter()
                .position(|&x| x == "artifacts" || x == "files")
            {
                let root_path = part_split[..index].join("/");
                dbg!(&root_path);
                // 提取从 "artifacts or files" 开始的部分
                let relative_path = part_split[index..].join("/");
                dbg!(&relative_path);

                if let Ok(storage) = Storage::new_fs(root_path) {
                    storage.read_blocking(relative_path).map_or_else(
                        |e| {
                            tracing::error!("failed to read storage");
                            return ResponseBuilder::new()
                                .status(404)
                                .body(format!("error: {}", e).as_bytes().to_vec())
                                .map_err(Into::into);
                        },
                        |data| {
                            return ResponseBuilder::new()
                                .header(
                                    CONTENT_TYPE,
                                    HeaderValue::try_from("video/quicktime")
                                        .expect("Invalid header value"),
                                )
                                .status(200)
                                .body(data.to_vec())
                                .map_err(Into::into);
                        },
                    )
                } else {
                    tracing::error!("failed to read storage");
                    return ResponseBuilder::new()
                        .status(500)
                        .body(Vec::new())
                        .map_err(Into::into);
                }
            } else {
                tracing::error!("failed to create storage");
                return ResponseBuilder::new()
                    .status(500)
                    .body(Vec::new())
                    .map_err(Into::into);
            }
        })
        .setup(|_app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = _app.get_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet,]);

    let app = app
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    #[cfg(not(debug_assertions))]
    {
        /*
         * macos log dir
         * ~/Library/Logs/ai.gendam.desktop/app.log
         */
        let log_dir = app.path_resolver().app_log_dir().unwrap();
        analytics_tracing::init_tracing_to_file(log_dir);
    }
    #[cfg(debug_assertions)]
    {
        analytics_tracing::init_tracing_to_stdout();
    }
    {
        // https://docs.rs/tracing/latest/tracing/struct.Span.html#in-asynchronous-code
        // Spans will be sent to the configured OpenTelemetry exporter
        // let root = tracing::span!(tracing::Level::INFO, "gendam-desktop", custom_field="custom value");
        // let _enter = root.enter();
        // tracing::error!("This event will be logged in the root span.");
    }

    // p2p
    let node = p2p::Node::new().expect("create node fail");

    let p2p = Arc::new(Mutex::new(node));

    // let p2p_clone = p2p.clone();

    // tauri::async_runtime::spawn(async move {
    //     let mut node = p2p_clone.lock().unwrap().clone();
    //     node.start_p2p().await.unwrap();
    // });

    let window = app.get_window("main").unwrap();
    let local_data_root = window
        .app_handle()
        .path_resolver()
        .app_local_data_dir()
        .expect("failed to find local data dir");
    std::fs::create_dir_all(&local_data_root).unwrap();

    let resources_dir = window
        .app_handle()
        .path_resolver()
        .resolve_resource("resources")
        .expect("failed to find resources dir");
    let temp_dir = std::env::temp_dir();
    let cache_dir = tauri::api::path::cache_dir().unwrap_or({
        tracing::error!("Failed to get cache dir");
        temp_dir.clone()
    });

    // app.app_handle()
    window
        .app_handle()
        .plugin(tauri_plugin_store::Builder::default().build())
        .expect("failed to add store plugin");
    // TODO: 需要确认下 tauri_plugin_store 这个 plugin 是不是需要，如果网页上不用，应该不需要

    // validate_app_version(window.app_handle(), &local_data_root);

    let mut tauri_store = tauri_plugin_store::StoreBuilder::new(
        window.app_handle(),
        "settings.json".parse().unwrap(),
    )
    .build();
    tauri_store.load().unwrap_or_else(|e| {
        tracing::warn!("Failed to load tauri store: {:?}", e);
    });

    // clipboard
    window
        .app_handle()
        .plugin(tauri_plugin_clipboard::init())
        .expect("fail to add clipboard plugin");

    let store = Arc::new(Mutex::new(Store::new(tauri_store)));
    let router = api_server::get_routes::<Ctx<Store>>();
    let ctx = Ctx::<Store>::new(local_data_root, resources_dir, temp_dir, cache_dir, store, p2p);

    window.on_window_event({
        let ctx = ctx.clone();
        move |e| {
            if let tauri::WindowEvent::Destroyed = e {
                // https://github.com/bmrlab/gendam/issues/10#issuecomment-2078827778
                tracing::info!("window destroyed");
                if let Ok(library) = ctx.library() {
                    // drop(library);
                    let pid = library.qdrant_server_info();
                    let _ = kill_qdrant_server(pid);
                }
                // tokio::runtime::Runtime::new().unwrap().block_on(async {
                //     let _ = ctx.unload_library().await;
                // });
            }
        }
    });

    window
        .app_handle()
        .plugin(rspc_tauri::plugin(router.arced(), move |_window| {
            // 不能每次 new 而应该是 clone，这样会保证 ctx 里面的每个元素每次只是新建了引用
            ctx.clone()
        }))
        .expect("failed to add rspc plugin");

    app.run(|_, _| {});
}

#[tauri::command]
fn greet(name: &str) -> String {
    println!("Hello, {}, from Server!", name);
    format!("Hello, {}, in Client!", name)
}

// #[allow(dead_code)]
// fn validate_app_version(app_handle: tauri::AppHandle, local_data_root: &std::path::PathBuf) {
//     // 目前先不需要调用这个, 这一次 release 不会用到旧的数据库
//     const VERSION_SHOULD_GTE: usize = 2;
//     let mut tauri_store =
//         tauri_plugin_store::StoreBuilder::new(app_handle, "settings.json".parse().unwrap()).build();
//     tauri_store.load().unwrap_or_else(|e| {
//         tracing::warn!("Failed to load tauri store: {:?}", e);
//     });
//     let version: usize = match tauri_store.get("version") {
//         Some(value) => value.as_str().unwrap_or("").to_string(),
//         None => "".to_string(),
//     }
//     .parse()
//     .unwrap_or(0);
//     if version < VERSION_SHOULD_GTE {
//         // check if libraries exists, if true, move it to archived/libraries
//         let libraries_dir = local_data_root.join("libraries");
//         if libraries_dir.exists() {
//             let archived_dir = local_data_root.join("archived");
//             std::fs::create_dir_all(&archived_dir).unwrap();
//             std::fs::rename(
//                 &libraries_dir,
//                 archived_dir.join(format!("libraries-{}", chrono::Utc::now().timestamp())),
//             )
//             .unwrap();
//         }
//         tauri_store.delete("current-library-id").unwrap();
//         tauri_store
//             .insert("version".to_string(), VERSION_SHOULD_GTE.to_string().into())
//             .unwrap_or_else(|e| tracing::warn!("Failed to insert version to tauri store: {:?}", e));
//         tauri_store.save().unwrap_or_else(|e| {
//             tracing::warn!("Failed to save tauri store: {:?}", e);
//         });
//     }
// }
