use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ArtifactsResult {
    pub dir: PathBuf,
    pub files: Option<Vec<PathBuf>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
/// The models and results settings of artifacts.
pub struct ArtifactsSettings {
    /// Record the task type and corresponding models,
    /// if a task type not exists in hashmap,
    /// the model should be taken according to library settings.
    pub models: HashMap<String, String>,
    /// Record the task type and corresponding results.
    ///
    /// A task type may have many results, so the results are stored in a hashmap.
    /// The key of the inner results hashmap is "{model_name}:{input_path}",
    /// and the value is the output path of the model with the corresponding input path.
    ///
    /// When reading results from this hashmap, the model should be determined by the `models` hashmap,
    /// and the input path should be determined recursively (e.g., the input path of image caption embedding
    /// is determined by the output path of image caption, so the input path should refer to the image caption
    /// in `models` hashmap firstly, and then find its output path in `results` hashmap).
    pub results: HashMap<String, HashMap<String, ArtifactsResult>>,
}
