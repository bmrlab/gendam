use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use prisma_lib::{file_path, PrismaClient};

// 获取相对路径
pub fn get_suffix_path(path: &str, prefix: &str) -> String {
    let suffix = std::path::Path::new(path)
        .strip_prefix(prefix)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    tracing::debug!("path: {path}, prefix: {prefix}, suffix: {suffix}");
    return suffix;
}

// 获取通过完全路径或者materialized_path，上面的文件夹
pub fn get_folder_list(old_path: String, is_full_path: bool) -> Vec<String> {
    let mut folder_list = Vec::new();

    let path = Path::new(&old_path);
    let path_split = path.components().collect::<Vec<_>>();
    let len = if is_full_path {
        1..path_split.len() - 1
    } else {
        1..path_split.len()
    };

    for i in len {
        let mut folder_path = "".to_string();
        for j in 1..=i {
            folder_path.push_str("/");
            folder_path.push_str(path_split[j].as_os_str().to_str().unwrap());
        }
        folder_list.push(folder_path);
    }
    tracing::debug!("folder_list: {folder_list:?}");

    folder_list
}

// 通过文件夹路径获取路径信息 "/111" "/22/33"
pub async fn get_folder_info_by_path(
    prisma_client: Arc<PrismaClient>,
    folder_path_string: String,
) -> Result<(PathBuf, String, String, file_path::Data), anyhow::Error> {
    let folder_path = Path::new(&folder_path_string);
    let folder_path_split = folder_path.components().collect::<Vec<_>>();
    // 取第一个到倒数第二个
    let mut folder_materialized_path = folder_path_split[0]
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string();
    for i in 1..folder_path_split.len() - 1 {
        folder_materialized_path.push_str(folder_path_split[i].as_os_str().to_str().unwrap());
        folder_materialized_path.push_str("/");
    }
    let folder_name = folder_path_split
        .last()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap();

    let folder_data = prisma_client
        .file_path()
        .find_unique(file_path::UniqueWhereParam::MaterializedPathNameEquals(
            folder_materialized_path.clone(),
            folder_name.to_string(),
        ))
        .exec()
        .await
        .unwrap()
        .unwrap();

    Ok((
        folder_path.to_path_buf(),
        folder_materialized_path,
        folder_name.to_string(),
        folder_data,
    ))
}

// 通过老的路径和新的名字获取新的路径 /xx/aa bb => /xx/bb
pub fn get_new_path_by_old_path_and_name(old_path: String, new_name: String) -> String {
    let path = Path::new(&old_path);
    let path_split = path.components().collect::<Vec<_>>();
    let mut new_path = String::new();
    for i in 1..path_split.len() - 1 {
        new_path.push_str("/");
        new_path.push_str(path_split[i].as_os_str().to_str().unwrap());
    }
    new_path.push_str("/");
    new_path.push_str(&new_name);
    new_path
}

// 通full_path 获取 name materialized_path /xxx/aaa/bb -> /xxx/aaa/  bb
pub fn get_name_and_materialized_path_by_full_path(full_path: String) -> (String, String) {
    let path = Path::new(&full_path);
    let path_split = path.components().collect::<Vec<_>>();
    let name = path_split
        .last()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string();
    let mut materialized_path = format!("/");
    for i in 1..path_split.len() - 1 {
        materialized_path.push_str(path_split[i].as_os_str().to_str().unwrap());
        materialized_path.push_str("/");
    }
    (name, materialized_path)
}
