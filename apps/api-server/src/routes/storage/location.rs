use content_library::Library;
use prisma_lib::data_location;

pub enum DataLocationType {
    Fs,
    S3,
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

pub async fn get_asset_object_location(
    library: &Library,
    asset_object_id: i32,
) -> Result<DataLocationType, rspc::Error> {
    let data_location = library
        .prisma_client()
        .data_location()
        .find_unique(data_location::UniqueWhereParam::IdEquals(asset_object_id))
        .exec()
        .await?;

    match data_location {
        Some(data_location) => Ok(DataLocationType::from(data_location.medium)),
        None => Ok(DataLocationType::Fs),
    }
}
