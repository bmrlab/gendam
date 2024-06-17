use crate::store::Store;
use api_server::ctx::default::Ctx;
use api_server::{
    get_asset_object_location, get_library_settings, CtxWithLibrary, DataLocationType,
};
use storage::S3Config;

pub struct StorageState {
    pub ctx: Ctx<Store>,
    pub(crate) cache_map: std::collections::HashMap<String, DataLocationType>,
}

impl StorageState {
    pub fn new(ctx: Ctx<Store>) -> Self {
        Self {
            ctx,
            cache_map: std::collections::HashMap::new(),
        }
    }

    // TODO: replace read settings file with cache
    pub fn get_library_settings(&self) -> anyhow::Result<S3Config> {
        let library = self.ctx.library()?;
        let library_settings = get_library_settings(&library.dir);
        library_settings
            .s3_config
            .ok_or(anyhow::anyhow!("s3 config not found"))
    }

    pub async fn get_location(&mut self, hash: &str) -> anyhow::Result<DataLocationType> {
        // get or insert
        return match self.cache_map.entry(hash.to_string()) {
            std::collections::hash_map::Entry::Occupied(entry) => {
                let location = entry.get();
                Ok(location.clone())
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                let library = self.ctx.library()?;
                let location = get_asset_object_location(&library, hash.to_string()).await?;

                entry.insert(location.clone());
                Ok(location)
            }
        };
    }
}
