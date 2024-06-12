use crate::store::Store;
use api_server::ctx::default::Ctx;
use api_server::{get_asset_object_location, CtxWithLibrary, DataLocationType};

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
