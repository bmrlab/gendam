use std::process::Command;
use serde::Serialize;
use rspc::Router;
use crate::{Ctx, R};

pub fn get_routes() -> Router<Ctx> {
    let router = R.router()
    // .procedure(
    //     "files",
    //     R.query(|_ctx, subpath: Option<String>| async move {
    //         // println!("subpath: {:?}", subpath);
    //         let res = list_files(subpath);
    //         serde_json::to_value(res).unwrap()
    //     })
    // )
    // .procedure(
    //     "folders",
    //     R.query(|_ctx, _input: ()| async move {
    //         let res = get_folders_tree();
    //         serde_json::to_value(res).unwrap()
    //     })
    // )
    .procedure("home_dir",
        R.query(|ctx, _input: ()| async move {
            ctx.library.files_dir.to_str().unwrap().to_string()
            // dirs::home_dir().unwrap()
        })
    )
    .procedure("ls",
        R.query(|_ctx, full_path: String| async move {
            let res = get_files_in_path(&full_path);
            serde_json::to_value(res).unwrap()
        })
    )
    .procedure("reveal",
        R.mutation(|_ctx, path: String| async move {
            let res = reveal_in_finder(&path);
            res.expect("failed reveal file in finder");
        })
    );
    return router;
}


#[derive(Serialize)]
struct File {
    name: String,
    is_dir: bool,
}

// fn list_files(subpath: Option<String>) -> Vec<File> {
//     let mut root_path = String::from("xxxxxx");
//     if let Some(subpath) = subpath {
//         root_path = format!("{}/{}", root_path, subpath);
//     }
//     let paths = std::fs::read_dir(root_path).unwrap();
//     let mut files = vec![];
//     for path in paths {
//         let file_name = path.as_ref().unwrap().file_name();
//         let file_path = path.as_ref().unwrap().path();
//         let file_path_str = file_name.to_str().unwrap().to_string();
//         let is_dir = file_path.is_dir();
//         let file = File {
//             name: file_path_str,
//             is_dir,
//         };
//         files.push(file);
//     }
//     files
// }

fn get_files_in_path(full_path: &str) -> Vec<File> {
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
        if !file.name.starts_with(".") {
            files.push(file);
        }
    }
    files
}

// fn get_folders_tree() -> Vec<File> {
//     vec![]
// }

fn reveal_in_finder(path: &str) -> std::io::Result<()> {
    Command::new("open")
        .arg("-R")
        .arg(path)
        .output()?;
    Ok(())
}
