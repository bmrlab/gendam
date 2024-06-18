use crate::store::Store;
use api_server::ctx::default::Ctx;
use api_server::{
    get_asset_object_location, get_library_settings, CtxWithLibrary, DataLocationType,
};
use storage::S3Config;

pub struct StorageState {
    pub ctx: Ctx<Store>,
    pub(crate) cache_map: std::collections::HashMap<String, DataLocationType>,
    pub(crate) s3_config_cache_map: std::collections::HashMap<String, S3Config>,
}

impl StorageState {
    pub fn new(ctx: Ctx<Store>) -> Self {
        Self {
            ctx,
            cache_map: std::collections::HashMap::new(),
            s3_config_cache_map: std::collections::HashMap::new(),
        }
    }

    fn get_s3_config_from_local(&self) -> anyhow::Result<S3Config> {
        let library = self.ctx.library()?;
        get_library_settings(&library.dir)
            .s3_config
            .ok_or(anyhow::anyhow!("s3 config not found"))
    }

    pub fn get_s3_config(&mut self) -> anyhow::Result<S3Config> {
        let library_id = self.ctx.library()?.id;
        return match self.s3_config_cache_map.entry(library_id.clone()) {
            std::collections::hash_map::Entry::Occupied(entry) => {
                let config = entry.get();
                Ok(config.clone())
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                drop(entry);
                let s3_config = self.get_s3_config_from_local();
                s3_config.map(|config| {
                    self.s3_config_cache_map.insert(library_id, config.clone());
                    config
                })
            }
        };
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
