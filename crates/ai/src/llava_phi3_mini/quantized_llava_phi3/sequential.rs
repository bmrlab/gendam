use super::linear::QLinear;
use candle_core::{Module, Tensor};

/// forked from candle-nn/src/sequential.rs
/// Sequantial 原先使用的是 trait object，这样会让 Sequential 无法实现 Send,
/// 而 impl Model for LLaVAPhi3Mini 里面的 async fn process(&mut self) 放回一个 Future，所以需要实现 Send
/// 所以这里将 Sequential 的 layers 从 Vec<Box<dyn Module>> 改为 Vec<QSequentialLayer>
pub enum QSequentialLayer {
    QLinear(QLinear),
    Activation(candle_nn::Activation),
}

/// A sequential layer combining multiple other layers.
pub struct QSequential {
    layers: Vec<QSequentialLayer>,
}

/// Creates a new empty sequential layer.
pub fn seq() -> QSequential {
    QSequential { layers: vec![] }
}

impl Module for QSequential {
    fn forward(&self, xs: &Tensor) -> candle_core::Result<Tensor> {
        let mut xs = xs.clone();
        for layer in self.layers.iter() {
            match layer {
                QSequentialLayer::QLinear(layer) => xs = layer.forward(&xs)?,
                QSequentialLayer::Activation(layer) => xs = layer.forward(&xs)?,
            }
        }
        Ok(xs)
    }
}

impl QSequential {
    /// Appends a layer after all the current layers.
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, layer: QSequentialLayer) -> Self {
        self.layers.push(layer);
        self
    }
}
