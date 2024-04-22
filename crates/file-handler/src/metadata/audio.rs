use serde::{Deserialize, Serialize};

use crate::traits::FileMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub bit_rate: usize,
    pub duration: f64,
}

impl FileMetadata for AudioMetadata {}
