// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tauri::Manager;
use serde::Serialize;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, list_files,])
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
fn list_files() -> Vec<File> {
    // list files form current directory, mark folder as folder
    let paths = std::fs::read_dir("./").unwrap();
    let mut files = vec![];
    for path in paths {
        let file_name = path.as_ref().unwrap().file_name();
        let file_path = path.as_ref().unwrap().path();
        let file_path_str = file_name.to_str().unwrap().to_string();
        let is_dir = file_path.is_dir();
        let file = File {
            name: file_path_str,
            is_dir: is_dir,
        };
        files.push(file);
    }
    files
}
