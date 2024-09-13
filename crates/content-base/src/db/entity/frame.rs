use serde::Deserialize;
use surrealdb::sql::Thing;

use super::{ImageEntity, TextEntity};

#[derive(Debug, Deserialize)]
pub struct AudioFrameEntity {
    id: Thing,
    data: Vec<TextEntity>,
    start_timestamp: usize,
    end_timestamp: usize,
}

#[derive(Debug, Deserialize)]
pub struct ImageFrameEntity {
    id: Thing,
    data: Vec<ImageEntity>,
    start_timestamp: usize,
    end_timestamp: usize,
}
