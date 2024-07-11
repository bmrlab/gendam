use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub bit_rate: usize,
    pub duration: f64,
}
