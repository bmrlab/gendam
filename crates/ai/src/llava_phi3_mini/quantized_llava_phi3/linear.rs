use candle_core::{quantized::QMatMul, Module, Tensor};
use candle_transformers::quantized_var_builder;

#[derive(Debug, Clone)]
pub struct QLinear {
    inner: QMatMul,
    bias: Tensor,
    span: tracing::Span,
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
        let span = tracing::span!(tracing::Level::TRACE, "qmatmul");
        Ok(Self { inner, bias, span })
    }
}

impl Module for QLinear {
    fn forward(&self, xs: &Tensor) -> candle_core::Result<Tensor> {
        let _enter = self.span.enter();
        self.inner.forward(xs)?.broadcast_add(&self.bias)
    }
}
