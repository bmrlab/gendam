use api_server::{CtxStore, StoreError};

pub struct  Store {
    pub store: tauri_plugin_store::Store<tauri::Wry>,
}

impl Store {
    pub fn new(store: tauri_plugin_store::Store<tauri::Wry>) -> Self {
        Self { store }
    }
}

impl CtxStore for Store {
    fn load(&mut self) -> Result<(), StoreError> {
        match self.store.load() {
            Ok(_) => Ok(()),
            Err(e) => Err(StoreError(e.to_string()))
        }
    }
    fn save(&self) -> Result<(), StoreError> {
        match self.store.save() {
            Ok(_) => Ok(()),
            Err(e) => Err(StoreError(e.to_string()))
        }
    }
    fn insert(&mut self, key: &str, value: &str) -> Result<(), StoreError> {
        self.store.insert(String::from(key), value.into())
            .map_err(|e| StoreError(e.to_string()))?;
        Ok(())
    }
    fn get(&self, key: &str) -> Option<String> {
        let value = self.store.get(String::from(key));
        match value {
            Some(value) => {
                match value.as_str() {
                    Some(value) => Some(value.to_string()),
                    None => None,
                }
            },
            None => None,
        }
    }
}
