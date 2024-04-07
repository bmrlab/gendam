use futures::future::join_all;
use p2p_block::{Range, SpaceblockRequest};
use std::io::Write;
use std::{path::PathBuf, sync::Arc};
use tokio::fs::File;

pub async fn create_requests(
    files: Vec<(String, String, PathBuf, PathBuf, String, String)>,
) -> Result<(Vec<(PathBuf, File, String)>, Vec<SpaceblockRequest>), rspc::Error> {
    let res = join_all(files.into_iter().map(|item| async move {
        let name = item.0.clone();
        let hash = item.1.clone();
        let path = item.2;
        let materialized_path = item.5.clone();
        let file = File::open(&path).await?;
        let metadata = file.metadata().await?;
        let size = metadata.len();

        // 打包 对应的 artifacts_dir
        let artifacts_dir = item.3;
        // 判断是否是文件夹
        if !artifacts_dir.is_dir() {
            panic!("artifacts_dir: {:?} is not dir", artifacts_dir)
        }

        // 存放zip数据
        let artifacts_dir_zip_path = item.4;

        // spawn blocking
        let zip_future = tokio::task::spawn_blocking(move || async move {
            use std::os::unix::fs::PermissionsExt;
            use tokio::io::AsyncReadExt;

            let zip_path = artifacts_dir_zip_path.clone();

            // 添加权限
            std::fs::set_permissions(
                artifacts_dir.clone(),
                std::fs::Permissions::from_mode(0o755),
            )
            .unwrap();

            // 为什么用 std的，因为 zip 不支持tokio
            let zip_file = std::fs::File::create(zip_path.clone()).unwrap();

            let zip_file_arc = Arc::new(zip_file);
            // 读取文件夹下的所有文件
            // 用walkdir遍历文件夹
            let walkdir = walkdir::WalkDir::new(&artifacts_dir);
            let it = walkdir.into_iter();
            tracing::debug!("artifacts_dir: {:#?}", it);

            // 写到内存的zip文件
            let mut zip = zip::ZipWriter::new(zip_file_arc.clone());
            let mut buffer = Vec::new();
            for entry in it {
                let entry = entry.unwrap();
                tracing::debug!("entry: {:#?}", entry);
                let path = entry.path();
                tracing::debug!("path: {:#?}", path);
                if path.is_file() {
                    let name = path
                        .strip_prefix(artifacts_dir.clone())
                        .expect("strip prefix fail");
                    zip.start_file(name.to_str().unwrap(), Default::default())
                        .expect("start file fail");
                    let mut file = tokio::fs::File::open(path).await.expect("open file fail");
                    file.read_to_end(&mut buffer)
                        .await
                        .expect("read to end fail");
                    zip.write_all(&buffer).expect("write all fail");
                    buffer.clear();
                }
            }

            // zip 结束
            zip.finish().expect("zip finish fail");
            tracing::debug!("zip finish");
            (
                zip_file_arc.clone().metadata().unwrap().len(),
                zip_path.clone(),
            )
        })
        .await
        .unwrap();

        // 等待zip结束
        let (artifact_size, zip_path) = zip_future.await;

        Ok((
            (path, file, zip_path),
            SpaceblockRequest {
                name,
                hash,
                path: materialized_path,
                size,
                range: Range::Full,
                artifact_size,
            },
        ))
    }))
    .await
    .into_iter()
    .collect::<Result<Vec<_>, std::io::Error>>()
    .map_err(|err| {
        tracing::error!("error opening file: '{err:?}'");
        return rspc::Error::new(
            rspc::ErrorCode::InternalServerError,
            format!("error opening file: {err:?}"),
        );
    });

    match res {
        Ok(res) => Ok(res.into_iter().unzip()),
        Err(error) => {
            return Err(rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("create_requests error: {error}"),
            ))
        }
    }
}
