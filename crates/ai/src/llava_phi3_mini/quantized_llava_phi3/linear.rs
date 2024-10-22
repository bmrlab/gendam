use candle_core::{quantized::QMatMul, Module, Tensor};
use candle_transformers::quantized_var_builder;

#[derive(Debug, Clone)]
pub struct QLinear {
    inner: QMatMul,
    bias: Tensor,
}

impl QLinear {
    pub fn load(
        in_dim: usize,
        out_dim: usize,
        vb: quantized_var_builder::VarBuilder,
    ) -> candle_core::Result<Self> {
        let weight = vb.get((out_dim, in_dim), "weight")?;
        let bias = vb.get(out_dim, "bias")?;
        let inner = QMatMul::from_arc(weight)?;
        let bias = bias.dequantize(vb.device())?;
        Ok(Self { inner, bias })
    }
}

impl Module for QLinear {
    #[tracing::instrument(level = "info", name="QLinear" skip_all)]
    fn forward(&self, xs: &Tensor) -> candle_core::Result<Tensor> {
        self.inner.forward(xs)?.broadcast_add(&self.bias)
    }
}
