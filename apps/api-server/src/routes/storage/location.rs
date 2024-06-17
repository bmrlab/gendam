use content_library::Library;
use prisma_lib::asset_object;
use specta::Type;
use std::fmt;

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

    dbg!(&asset_object);

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
