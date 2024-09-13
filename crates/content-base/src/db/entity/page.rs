use serde::Deserialize;
use surrealdb::sql::Thing;

use super::{ImageEntity, TextEntity};

#[derive(Debug, Deserialize)]
pub struct PageEntity {
    id: Thing,
    text: Vec<TextEntity>,
    image: Vec<ImageEntity>,
    start_index: usize,
    end_index: usize,
}
