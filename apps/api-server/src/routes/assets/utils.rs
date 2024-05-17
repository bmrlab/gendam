use blake3::Hasher;
use content_library::Library;
use futures::stream::{self, StreamExt};
use prisma_lib::{file_path, sync_metadata};
use std::path::Path;
use tokio::{
    fs::{self, File},
    io::{self, AsyncReadExt, AsyncSeekExt, SeekFrom},
};

const SAMPLE_COUNT: u64 = 4;
const SAMPLE_SIZE: u64 = 1024 * 10;
const HEADER_OR_FOOTER_SIZE: u64 = 1024 * 8;
// minimum file size of 100KiB, to avoid sample hashing for small files as they can be smaller than the total sample size
const MINIMUM_FILE_SIZE: u64 = 1024 * 100;

// use static_assertions::const_assert;
// // Asserting that nobody messed up our consts
// const_assert!((HEADER_OR_FOOTER_SIZE * 2 + SAMPLE_COUNT * SAMPLE_SIZE) < MINIMUM_FILE_SIZE);
// // Asserting that the sample size is larger than header/footer size, as the same buffer is used for both
// const_assert!(SAMPLE_SIZE > HEADER_OR_FOOTER_SIZE);

pub async fn generate_file_hash(path: impl AsRef<Path>, size: u64) -> Result<String, io::Error> {
    let mut hasher = Hasher::new();
    hasher.update(&size.to_le_bytes());

    if size <= MINIMUM_FILE_SIZE {
        // For small files, we hash the whole file
        hasher.update(&fs::read(path).await?);
    } else {
        let mut file = File::open(path).await?;
        let mut buf = vec![0; SAMPLE_SIZE as usize].into_boxed_slice();

        // Hashing the header
        let mut current_pos = file
            .read_exact(&mut buf[..HEADER_OR_FOOTER_SIZE as usize])
            .await? as u64;
        hasher.update(&buf[..HEADER_OR_FOOTER_SIZE as usize]);

        // Sample hashing the inner content of the file
        let seek_jump = (size - HEADER_OR_FOOTER_SIZE * 2) / SAMPLE_COUNT;
        loop {
            file.read_exact(&mut buf).await?;
            hasher.update(&buf);

            if current_pos >= (HEADER_OR_FOOTER_SIZE + seek_jump * (SAMPLE_COUNT - 1)) {
                break;
            }

            current_pos = file.seek(SeekFrom::Start(current_pos + seek_jump)).await?;
        }

        // Hashing the footer
        file.seek(SeekFrom::End(-(HEADER_OR_FOOTER_SIZE as i64)))
            .await?;
        file.read_exact(&mut buf[..HEADER_OR_FOOTER_SIZE as usize])
            .await?;
        hasher.update(&buf[..HEADER_OR_FOOTER_SIZE as usize]);
    }

    Ok(hasher.finalize().to_hex()[..16].to_string())
}

// pub(crate) fn contains_invalid_chars(name: &str) -> bool {
//     name.chars().any(|c| match c {
//         '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => true,
//         _ => false,
//     })
// }

// pub(crate) fn normalized_materialized_path(path: &str) -> String {
//     if path.ends_with("/") {
//         path.to_string()
//     } else {
//         format!("{}/", path)
//     }
// }

/// Merge shared file or folder path
/// If relative_path empty skip the merge
/// Check if the record is a file or a folder
/// - If the path is a file, return `relative_path.join(materialized_path)`
/// - If the path is a folder
///    - Query all the file paths under the folder, replace relative_path with `root_path.join(materialized_path)`
pub async fn merge_shared_path(library: &Library, data: &mut Vec<file_path::Data>) {
    // (file_path_id, relative_path, materialized_path, sub_file_path_ids)
    let need_handle_tuple: Vec<(String, String, String, Vec<String>)> = stream::iter(data.clone())
        .filter_map(|d| async move {
            if d.relative_path.is_none() {
                return None;
            }
            let relative_path = d.relative_path.clone().unwrap();

            match library
                .prisma_client()
                .sync_metadata()
                .find_unique(sync_metadata::UniqueWhereParam::FilePathIdEquals(
                    d.id.clone(),
                ))
                .exec()
                .await
            {
                Ok(sync_metadata) => {
                    if let Some(sync_metadata) = sync_metadata {
                        Some((
                            d.id.clone(),
                            relative_path,
                            d.materialized_path,
                            serde_json::from_str(
                                &sync_metadata.sub_file_path_ids.unwrap_or("[]".into()),
                            )
                            .unwrap_or(vec![]),
                        ))
                    } else {
                        None
                    }
                }
                // 查询失败，跳过
                Err(_) => None,
            }
        })
        .collect()
        .await;

    _merge_shared_path(need_handle_tuple, data);
}

fn _merge_shared_path(
    need_handle_tuple: Vec<(String, String, String, Vec<String>)>,
    data: &mut Vec<file_path::Data>,
) {
    for tuple in need_handle_tuple {
        let (file_path_id, relative_path, materialized_path, sub_file_path_ids) = tuple;

        for d in data.iter_mut() {
            // 当前 id 不包含在 sub_file_path_ids 中，并且当前 ID 不等于 file_path_id
            // 当前实例不是子路径
            if !(sub_file_path_ids.contains(&d.id) || file_path_id.eq(&d.id))
                || !is_subpath(&materialized_path, &d.materialized_path)
            {
                continue;
            }

            d.materialized_path = format!(
                "{}{}",
                relative_path,
                d.materialized_path
                    .strip_prefix('/')
                    .unwrap_or(&d.materialized_path)
            );
        }
    }
}

fn is_subpath(parent: &str, child: &str) -> bool {
    child.strip_prefix(parent).is_some()
}

#[cfg(test)]
mod test {

    use crate::routes::assets::utils::{_merge_shared_path, merge_shared_path};
    use prisma_lib::file_path::Data;

    fn create_data(
        id: &str,
        is_dir: bool,
        materialized_path: &str,
        relative_path: Option<&str>,
        name: &str,
    ) -> Data {
        Data {
            id: id.to_string(),
            is_dir,
            materialized_path: materialized_path.to_string(),
            relative_path: relative_path.map(|s| s.to_string()),
            name: name.to_string(),
            description: None,
            asset_object_id: None,
            asset_object: None,
            created_at: Default::default(),
            updated_at: None,
            sync_metadata: None,
        }
    }

    #[tokio::test]
    async fn test_generate_file_hash() {
        let mut data = vec![
            create_data("", false, "/", None, ""),
            create_data("", true, "/", None, "common_dir"),
            create_data("", false, "/common_dir/", None, "file1"),
            create_data("", true, "/common_dir/", None, "sync_dir"),
            // sync file
            create_data("s_1", true, "/", Some("/common_dir/sync_dir/"), "dir1"),
            create_data("s_2", false, "/dir1/", None, "x1_file"),
            create_data("s_3", false, "/dir1/", None, "x2_file"),
            create_data("s_4", true, "/dir1/", None, "dir2"),
            create_data("s_5", true, "/dir1/dir2/", None, "xx1_file"),
            create_data("s_6", true, "/dir1/dir2/", None, "xx2_file"),
        ];
        // (relative_path, materialized_path, sub_file_path_ids)
        let need_handle_tuple = vec![(
            "s_1".to_string(),
            "/common_dir/sync_dir/".to_string(),
            "/".to_string(),
            vec!["s_2", "s_3", "s_4", "s_5", "s_6"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        )];

        let data = &mut data;
        _merge_shared_path(need_handle_tuple, data);

        data.iter()
            .map(|d| {
                (
                    d.id.clone(),
                    d.is_dir,
                    d.materialized_path.clone(),
                    d.name.clone(),
                )
            })
            .for_each(|d| println!("{:?}", d));
    }
}
