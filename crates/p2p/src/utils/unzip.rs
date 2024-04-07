use std::path::PathBuf;

use crate::P2PError;

pub async fn unzip_artifact(artifact_path: PathBuf) -> Result<(), P2PError<()>> {
    let artifact_path = PathBuf::from(artifact_path.clone());
    let zip_dir = artifact_path.parent().expect("artifact_path parent error");
    let zip_name = artifact_path
        .file_stem()
        .expect("no found zip")
        .to_str()
        .expect("zip_name to_str error");
    let extract_dir = zip_dir.join(zip_name);

    tracing::debug!("artifact_path: '{artifact_path:?}', zip_dir: '{zip_dir:?}', zip_name: '{zip_name:?}', extract_dir: '{extract_dir:?}'");

    #[cfg(unix)]
    {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&zip_dir, fs::Permissions::from_mode(0o777))
            .expect("set zip_dir permission error");
    }

    if !extract_dir.exists() {
        tokio::fs::create_dir_all(&extract_dir)
            .await
            .map_err(|err| {
                tracing::error!("error creating extract directory '{extract_dir:?}': '{err:?}'");
            })
            .unwrap();
    }

    let artifact_zip = std::fs::File::open(&artifact_path)
        .map_err(|err| {
            tracing::error!("error opening artifact zip at '{artifact_path:?}': '{err:?}'");
        })
        .unwrap();

    let mut archive = zip::ZipArchive::new(artifact_zip).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();

        let outpath = extract_dir.join(file.name());

        tracing::debug!("File {i} extracted to \"{}\"", outpath.display());

        {
            let comment = file.comment();
            if !comment.is_empty() {
                tracing::debug!("File {i} comment: {comment}");
            }
        }

        if (*file.name()).ends_with('/') {
            tracing::debug!("File {} extracted to \"{}\"", i, outpath.display());
            std::fs::create_dir_all(&outpath).unwrap();
        } else {
            tracing::debug!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p).unwrap();
                }
            }
            let mut outfile = std::fs::File::create(&outpath).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }
    // 再删除 zip 文件
    tokio::fs::remove_file(&artifact_path)
        .await
        .map_err(|err| {
            tracing::error!("error removing artifact zip at '{artifact_path:?}': '{err:?}'");
            err
        })
        .unwrap();

    Ok(())
}
