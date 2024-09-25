use std::collections::HashMap;
use serde::Serialize;
use surrealdb::sql::Thing;

#[derive(Debug, Clone, Eq, PartialEq, Copy, Hash, Serialize)]
pub enum TB {
    Text,
    Image,
    Item,
    ImageFrame,
    AudioFrame,
    Audio,
    Video,
    Page,
    Web,
    Document,
    Payload,
}

impl TB {
    fn mapping() -> HashMap<&'static str, TB> {
        let mut mapping = HashMap::new();
        mapping.insert("text", TB::Text);
        mapping.insert("image", TB::Image);
        mapping.insert("item", TB::Item);
        mapping.insert("image_frame", TB::ImageFrame);
        mapping.insert("audio_frame", TB::AudioFrame);
        mapping.insert("audio", TB::Audio);
        mapping.insert("video", TB::Video);
        mapping.insert("page", TB::Page);
        mapping.insert("web", TB::Web);
        mapping.insert("document", TB::Document);
        mapping.insert("payload", TB::Payload);
        mapping
    }
}

impl From<&str> for TB {
    fn from(value: &str) -> Self {
        *TB::mapping().get(value).unwrap_or(&TB::Text)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize)]
pub struct ID {
    id: String,
    tb: TB,
}

impl From<&Thing> for ID {
    fn from(value: &Thing) -> Self {
        ID::new(value.id.to_raw(), value.tb.as_str().into())
    }
}

impl From<&str> for ID {
    fn from(value: &str) -> Self {
        let mut iter = value.split(':');
        let tb = iter.next().unwrap();
        let id = iter.next().unwrap();
        Self {
            id: id.into(),
            tb: tb.into(),
        }
    }
}

impl ID {
    pub fn new(id: String, tb: &str) -> Self {
        Self { id, tb: tb.into() }
    }

    pub fn table_name(&self) -> &str {
        TB::mapping()
            .iter()
            .find_map(|(&key, &value)| if value == self.tb { Some(key) } else { None })
            .unwrap_or("text")
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn id_with_table(&self) -> String {
        format!("{}:{}", self.table_name(), self.id)
    }

    pub fn tb(&self) -> &TB {
        &self.tb
    }
}
