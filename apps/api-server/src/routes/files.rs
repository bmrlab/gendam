use std::{
    path::PathBuf,
    process::Command,
};
use serde::Serialize;
use rspc::{Rspc, Router};
use rspc::internal::middleware::MiddlewareContext;
// use crate::{Ctx, R};
use crate::CtxWithLibrary;

pub fn get_routes<TCtx>() -> Router<TCtx>
where TCtx: CtxWithLibrary + Clone + Send + Sync + 'static
{
    let router = Rspc::<TCtx>::new().router()
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
    .procedure(
        "home_dir",
        Rspc::<TCtx>::new()
        .with(|mw: MiddlewareContext, ctx| {
            // let local_data_root = ctx.local_data_root;
            async move {
                // let res = dirs::home_dir().unwrap();
                Ok(mw.next(ctx))
            }
        })
        .query(|ctx, _input: ()| async move {
            let library = ctx.load_library()?;
            Ok(library.files_dir.to_str().unwrap().to_string())
            // dirs::home_dir().unwrap()
        })
    )
    .procedure(
        "ls",
        Rspc::<TCtx>::new().query(|ctx, path: String| async move {
            let library = ctx.load_library()?;
            if !path.starts_with("/") {
                // let res = serde_json::to_value::<Vec<File>>(vec![]);
                // return res.map_err(|e| {
                //     rspc::Error::new(
                //         rspc::ErrorCode::BadRequest,
                //         String::from("path muse be start with /")
                //     )
                // });
                return Err(rspc::Error::new(
                    rspc::ErrorCode::BadRequest,
                    String::from("path muse be start with /")
                ));
            }
            let relative_path = format!(".{}", path);
            let files_dir = library.files_dir;
            let ls_dir = files_dir.join(relative_path);
            let res = get_files_in_path(&ls_dir);
            Ok(serde_json::to_value(res).unwrap())
        })
    )
    .procedure(
        "reveal",
        Rspc::<TCtx>::new().mutation(|ctx, path: String| async move {
            let library = ctx.load_library()?;
            let relative_path = format!(".{}", path);
            let files_dir = library.files_dir;
            let reveal_path = files_dir.join(relative_path).into_os_string().into_string().unwrap();
            match reveal_in_finder(&reveal_path) {
                Ok(_) => Ok(()),
                Err(e) => Err(rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("failed reveal file in finder: {}", e)
                ))
            }
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
    Command::new("open")
        .arg("-R")
        .arg(path)
        .output()?;
    Ok(())
}
