use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
};

pub struct TaskStore<TValue> {
    store: HashMap<String, TValue>,
    prefix_index: BTreeMap<String, Vec<String>>,
}

impl<TValue> TaskStore<TValue> {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            prefix_index: BTreeMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: TValue) {
        if !self.store.contains_key(key) {
            // TODO 实际上不用索引所有前缀，只需要按照分隔符进行分割就可以了
            for i in 1..=key.len() {
                let prefix = &key[0..i];
                self.prefix_index
                    .entry(prefix.to_string())
                    .or_insert_with(Vec::new)
                    .push(key.to_string());
            }
        }
        self.store.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<&TValue> {
        self.store.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut TValue> {
        self.store.get_mut(key)
    }

    pub fn get_all<'a>(
        &'a self,
        pattern: &'a str,
    ) -> impl Iterator<Item = (Cow<'a, str>, &'a TValue)> + 'a {
        let wildcard = pattern.strip_suffix('*').unwrap_or(pattern);

        self.prefix_index
            .get(wildcard)
            .into_iter()
            .flat_map(|keys| keys.iter())
            .filter_map(move |key| {
                self.store.get(key).map(|value| {
                    let cow_key = if key == wildcard {
                        Cow::Borrowed(key.as_str())
                    } else {
                        Cow::Owned(key.to_string())
                    };
                    (cow_key, value)
                })
            })
    }

    pub fn remove(&mut self, key: &str) -> Option<TValue> {
        let removed = self.store.remove(key);
        if removed.is_some() {
            // Remove key from all prefix indices
            for i in 1..=key.len() {
                let prefix = &key[0..i];
                if let Some(keys) = self.prefix_index.get_mut(prefix) {
                    keys.retain(|k| k != key);
                    if keys.is_empty() {
                        self.prefix_index.remove(prefix);
                    }
                }
            }
        }
        removed
    }
}
