use crate::CtxWithLibrary;
use rspc::{Router, RouterBuilder};
use serde::Serialize;
use std::{path::PathBuf, process::Command};

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new()
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
        .query("home_dir", |t| {
            t(|ctx, _input: ()| async move {
                let library = ctx.library()?;
                Ok(library.files_dir.to_str().unwrap().to_string())
                // dirs::home_dir().unwrap()
            })
        })
        .query("ls", |t| {
            t(|ctx, path: String| async move {
                let library = ctx.library()?;
                if !path.starts_with("/") {
                    return Err(rspc::Error::new(
                        rspc::ErrorCode::BadRequest,
                        String::from("path muse be start with /"),
                    ));
                }
                let relative_path = format!(".{}", path);
                let files_dir = library.files_dir;
                let ls_dir = files_dir.join(relative_path);
                let res = get_files_in_path(&ls_dir);
                Ok(serde_json::to_value(res).unwrap())
            })
        })
        .mutation("reveal", |t| {
            t(|ctx, path: String| async move {
                let library = ctx.library()?;
                let relative_path = format!(".{}", path);
                let files_dir = library.files_dir;
                let reveal_path = files_dir
                    .join(relative_path)
                    .into_os_string()
                    .into_string()
                    .unwrap();
                match reveal_in_finder(&reveal_path) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("failed reveal file in finder: {}", e),
                    )),
                }
            })
        })
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

fn get_files_in_path(ls_dir: &PathBuf) -> Vec<File> {
    let result = ls_dir.read_dir();
    // let result = std::fs::read_dir(full_path);
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
    Command::new("open").arg("-R").arg(path).output()?;
    Ok(())
}
