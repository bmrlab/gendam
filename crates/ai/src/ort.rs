use std::path::Path;

use ort::{ExecutionProvider, GraphOptimizationLevel, Session};
use tracing::warn;

pub(crate) struct ONNXModelConfig {
    pub num_intra_thread: usize,
    pub optimization_level: GraphOptimizationLevel,
}

impl Default for ONNXModelConfig {
    fn default() -> Self {
        Self {
            num_intra_thread: 16,
            optimization_level: GraphOptimizationLevel::Level3,
        }
    }
}

/// Load ONNX model from file with some predefined config
///
/// Maybe it's better to use global default, which can be referred from:
/// https://ort.pyke.io/perf/execution-providers#global-defaults
pub(crate) fn load_onnx_model(
    model_path: impl AsRef<Path>,
    config: Option<ONNXModelConfig>,
) -> anyhow::Result<Session> {
    let builder = Session::builder()?;

    // let coreml = CoreMLExecutionProvider::default();
    // if coreml.register(&builder).is_err() {
    //     warn!("failed to register CoreMLExecutionProvider");
    // }

    let config = config.unwrap_or(Default::default());

    let session = builder
        .with_intra_threads(config.num_intra_thread)?
        .with_optimization_level(config.optimization_level)?
        .commit_from_file(model_path)?;

    Ok(session)
}
