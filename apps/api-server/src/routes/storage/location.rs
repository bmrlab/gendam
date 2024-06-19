use content_library::Library;
use global_variable::{get_current_s3_storage, get_or_insert_fs_storage};
use prisma_lib::asset_object;
use specta::Type;
use std::fmt;
use storage::Storage;

use crate::get_library_settings;

#[derive(Type, Eq, PartialEq, Clone, Debug)]
pub enum DataLocationType {
    Fs,
    S3,
}

impl Default for DataLocationType {
    fn default() -> Self {
        DataLocationType::Fs
    }
}

impl From<String> for DataLocationType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "fs" => DataLocationType::Fs,
            "s3" => DataLocationType::S3,
            _ => DataLocationType::Fs,
        }
    }
}

impl fmt::Display for DataLocationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataLocationType::Fs => write!(f, "fs"),
            DataLocationType::S3 => write!(f, "s3"),
        }
    }
}

pub async fn get_asset_object_location(
    library: &Library,
    hash: String,
) -> Result<DataLocationType, rspc::Error> {
    let asset_object = library
        .prisma_client()
        .asset_object()
        .find_unique(asset_object::hash::equals(hash))
        .with(asset_object::data_location::fetch(vec![]))
        .exec()
        .await?;

    if let Some(asset_object) = asset_object {
        if let Some(data_location) = asset_object.data_location {
            let res = data_location
                .iter()
                .find(|d| DataLocationType::from(d.medium.clone()) == DataLocationType::S3);

            if let Some(_) = res {
                return Ok(DataLocationType::S3);
            }
        }
    }

    return Ok(DataLocationType::Fs);
}

pub fn get_hash_from_url(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    if let Some(artifacts_index) = parts.iter().position(|&r| r == "artifacts" || r == "files") {
        if let Some(hash) = parts.get(artifacts_index + 2) {
            return Some(hash.to_string());
        } else {
            None
        }
    } else {
        return None;
    }
}

pub async fn get_storage_by_location(
    library: &Library,
    relative_path: String,
    library_dir: Option<String>,
) -> Result<Box<dyn Storage>, rspc::Error> {
    let root_path = library_dir.unwrap_or(
        library
            .dir
            .clone()
            .to_str()
            .expect("library dir")
            .to_string(),
    );

    let local_exist =
        std::path::Path::new(format!("{}/{}", root_path, relative_path).as_str()).exists();
    let mut location = DataLocationType::Fs;
    if !local_exist {
        let hash = get_hash_from_url(&relative_path);
        if hash.is_some() {
            location = get_asset_object_location(&library, hash.unwrap().to_string()).await?;
        }
    }
    let storage: Box<dyn Storage> = match location {
        DataLocationType::Fs => Box::new(get_or_insert_fs_storage!(root_path).map_err(|_| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                "Current FS config not found".to_string(),
            )
        })?),
        DataLocationType::S3 => {
            let s3_config =
                get_library_settings(&library.dir)
                    .s3_config
                    .ok_or(rspc::Error::new(
                        rspc::ErrorCode::PreconditionFailed,
                        "S3 config not set".to_string(),
                    ))?;
            Box::new(get_current_s3_storage!(s3_config).map_err(|_| {
                rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    "Current S3 config not found".to_string(),
                )
            })?)
        }
    };
    Ok(storage)
}
