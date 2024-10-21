use super::linear::QLinear;
use candle_core::{shape::D, DType, IndexOp, Module, Result, Shape, Tensor};
use candle_nn::Conv2dConfig;
use candle_transformers::quantized_var_builder;

#[derive(Debug, Clone, Copy)]
pub enum Activation {
    QuickGelu,
}

impl Module for Activation {
    fn forward(&self, xs: &Tensor) -> candle_core::Result<Tensor> {
        match self {
            Activation::QuickGelu => xs * candle_nn::ops::sigmoid(&(xs * 1.702f64)?),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClipVisionConfig {
    pub activation: Activation,
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    #[allow(dead_code)]
    pub projection_dim: usize,
    pub num_channels: usize,
    pub image_size: usize,
    pub patch_size: usize,
}

fn layer_norm(
    size: usize,
    config: candle_nn::LayerNormConfig,
    vb: quantized_var_builder::VarBuilder,
) -> Result<candle_nn::LayerNorm> {
    let weight = vb.get(size, "weight")?;
    let weight = weight.dequantize(vb.device())?;
    if config.affine {
        let bias = vb.get(size, "bias")?;
        let bias = bias.dequantize(vb.device())?;
        Ok(candle_nn::LayerNorm::new(weight, bias, config.eps))
    } else {
        Ok(candle_nn::LayerNorm::new_no_bias(weight, config.eps))
    }
}

// https://github.com/huggingface/transformers/blob/f6fa0f0bf0796ac66f201f23bdb8585de1609add/src/transformers/models/clip/modeling_clip.py#L112
#[derive(Clone, Debug)]
struct ClipVisionEmbeddings {
    patch_embedding: candle_nn::Conv2d,
    position_ids: Tensor,
    class_embedding: Tensor,
    position_embedding: candle_nn::Embedding,
}

impl ClipVisionEmbeddings {
    fn new(vs: quantized_var_builder::VarBuilder, c: &ClipVisionConfig) -> Result<Self> {
        // originally nn.Parameter
        let class_embedding = vs.get(c.hidden_size, "class_embd")?;
        let class_embedding = class_embedding.dequantize(vs.device())?;

        let num_patches = (c.image_size / c.patch_size).pow(2);
        let num_positions = num_patches + 1;
        let position_ids = Tensor::arange(0, num_positions as i64, vs.device())?;

        let conv2dconfig = Conv2dConfig {
            stride: c.patch_size,
            ..Default::default()
        };

        let position_embedding = {
            // candle_nn::embedding(num_positions, c.embed_dim, vs.pp("position_embedding"))?;
            let (in_size, out_size) = (num_positions, c.hidden_size);
            let embeddings = vs.get((in_size, out_size), "position_embd.weight")?;
            let embeddings = embeddings.dequantize(vs.device())?;
            candle_nn::Embedding::new(embeddings, out_size)
        };

        let patch_embedding = {
            // let patch_embedding = candle_nn::conv2d_no_bias(c.num_channels, c.embed_dim, c.patch_size, conv2dconfig, vs.pp("patch_embedding"))?;
            let (in_channels, out_channels, kernel_size) =
                (c.num_channels, c.hidden_size, c.patch_size);
            let ws = vs.get(
                (
                    out_channels,
                    in_channels / conv2dconfig.groups,
                    kernel_size,
                    kernel_size,
                ),
                "patch_embd.weight",
            )?;
            let ws = ws.dequantize(vs.device())?;
            candle_nn::Conv2d::new(ws, None, conv2dconfig)
        };

        Ok(Self {
            patch_embedding,
            position_ids,
            class_embedding,
            position_embedding,
        })
    }
}

impl Module for ClipVisionEmbeddings {
    fn forward(&self, pixel_values: &Tensor) -> Result<Tensor> {
        let batch_size = pixel_values.shape().dims();
        let patch_embeds = self
            .patch_embedding
            .forward(pixel_values)?
            .flatten_from(2)?
            .transpose(1, 2)?;
        let shape = Shape::from((batch_size[0], 1, self.class_embedding.dim(D::Minus1)?));
        let class_embeds = self.class_embedding.expand(shape)?;
        let embeddings = Tensor::cat(&[class_embeds, patch_embeds], 1)?;
        let position_embedding = self.position_embedding.forward(&self.position_ids)?;
        embeddings.broadcast_add(&position_embedding)
    }
}

#[derive(Clone, Debug)]
struct ClipAttention {
    k_proj: QLinear,
    v_proj: QLinear,
    q_proj: QLinear,
    out_proj: QLinear,
    head_dim: usize,
    scale: f64,
    num_attention_heads: usize,
}

impl ClipAttention {
    fn new(vs: quantized_var_builder::VarBuilder, c: &ClipVisionConfig) -> Result<Self> {
        let embed_dim = c.hidden_size;
        let num_attention_heads = c.num_attention_heads;
        let k_proj = QLinear::load(embed_dim, embed_dim, vs.pp("attn_k"))?;
        let v_proj = QLinear::load(embed_dim, embed_dim, vs.pp("attn_v"))?;
        let q_proj = QLinear::load(embed_dim, embed_dim, vs.pp("attn_q"))?;
        let out_proj = QLinear::load(embed_dim, embed_dim, vs.pp("attn_out"))?;
        let head_dim = embed_dim / num_attention_heads;
        let scale = (head_dim as f64).powf(-0.5);

        Ok(ClipAttention {
            k_proj,
            v_proj,
            q_proj,
            out_proj,
            head_dim,
            scale,
            num_attention_heads,
        })
    }

    fn shape(&self, xs: &Tensor, seq_len: usize, bsz: usize) -> Result<Tensor> {
        xs.reshape((bsz, seq_len, self.num_attention_heads, self.head_dim))?
            .transpose(1, 2)?
            .contiguous()
    }

    fn forward(&self, xs: &Tensor, causal_attention_mask: Option<&Tensor>) -> Result<Tensor> {
        let in_dtype = xs.dtype();
        let (bsz, seq_len, embed_dim) = xs.dims3()?;

        let query_states = (self.q_proj.forward(xs)? * self.scale)?;
        let proj_shape = (bsz * self.num_attention_heads, seq_len, self.head_dim);
        let query_states = self
            .shape(&query_states, seq_len, bsz)?
            .reshape(proj_shape)?
            .to_dtype(DType::F32)?;
        let key_states = self
            .shape(&self.k_proj.forward(xs)?, seq_len, bsz)?
            .reshape(proj_shape)?
            .to_dtype(DType::F32)?;
        let value_states = self
            .shape(&self.v_proj.forward(xs)?, seq_len, bsz)?
            .reshape(proj_shape)?
            .to_dtype(DType::F32)?;
        let attn_weights = query_states.matmul(&key_states.transpose(1, 2)?)?;

        let src_len = key_states.dim(1)?;

        let attn_weights = if let Some(causal_attention_mask) = causal_attention_mask {
            attn_weights
                .reshape((bsz, self.num_attention_heads, seq_len, src_len))?
                .broadcast_add(causal_attention_mask)?
                .reshape((bsz * self.num_attention_heads, seq_len, src_len))?
        } else {
            attn_weights
        };

        let attn_weights = candle_nn::ops::softmax(&attn_weights, D::Minus1)?;

        let attn_output = attn_weights.matmul(&value_states)?.to_dtype(in_dtype)?;
        let attn_output = attn_output
            .reshape((bsz, self.num_attention_heads, seq_len, self.head_dim))?
            .transpose(1, 2)?
            .reshape((bsz, seq_len, embed_dim))?;
        self.out_proj.forward(&attn_output)
    }
}

#[derive(Clone, Debug)]
struct ClipMlp {
    ffn_down: QLinear,
    ffn_up: QLinear,
    activation: Activation,
}

impl ClipMlp {
    fn new(vs: quantized_var_builder::VarBuilder, c: &ClipVisionConfig) -> Result<Self> {
        let ffn_down = QLinear::load(c.hidden_size, c.intermediate_size, vs.pp("ffn_down"))?;
        let ffn_up = QLinear::load(c.intermediate_size, c.hidden_size, vs.pp("ffn_up"))?;
        Ok(ClipMlp {
            ffn_down,
            ffn_up,
            activation: c.activation,
        })
    }
}

impl ClipMlp {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let xs = self.ffn_down.forward(xs)?;
        self.ffn_up.forward(&self.activation.forward(&xs)?)
    }
}

#[derive(Clone, Debug)]
struct ClipEncoderLayer {
    self_attn: ClipAttention,
    layer_norm1: candle_nn::LayerNorm,
    mlp: ClipMlp,
    layer_norm2: candle_nn::LayerNorm,
}

impl ClipEncoderLayer {
    fn new(vs: quantized_var_builder::VarBuilder, c: &ClipVisionConfig) -> Result<Self> {
        let self_attn = ClipAttention::new(vs.clone(), c)?;

        let layer_norm1 = layer_norm(c.hidden_size, 1e-5.into(), vs.pp("ln1"))?;
        let mlp = ClipMlp::new(vs.clone(), c)?;
        let layer_norm2 = layer_norm(c.hidden_size, 1e-5.into(), vs.pp("ln2"))?;

        Ok(ClipEncoderLayer {
            self_attn,
            layer_norm1,
            mlp,
            layer_norm2,
        })
    }

    fn forward(&self, xs: &Tensor, causal_attention_mask: Option<&Tensor>) -> Result<Tensor> {
        let residual = xs;
        let xs = self.layer_norm1.forward(xs)?;
        let xs = self.self_attn.forward(&xs, causal_attention_mask)?;
        let xs = (xs + residual)?;

        let residual = &xs;
        let xs = self.layer_norm2.forward(&xs)?;
        let xs = self.mlp.forward(&xs)?;
        xs + residual
    }
}

#[derive(Clone, Debug)]
pub struct ClipEncoder {
    layers: Vec<ClipEncoderLayer>,
}

impl ClipEncoder {
    pub fn new(vs: quantized_var_builder::VarBuilder, c: &ClipVisionConfig) -> Result<Self> {
        let mut layers: Vec<ClipEncoderLayer> = Vec::new();
        let num_hidden_layers = c.num_hidden_layers;
        // gguf 里面 clip.vision.block_count	是 23，这里的值是 23，但一般 llava 里面是 24
        // 见 ClipVisionTower::forward 的说明，因为 clip 的最后一层在 llava 里用不到，所以直接 gguf 里面去掉了
        for index in 0..num_hidden_layers {
            let layer = ClipEncoderLayer::new(vs.pp(index.to_string()), c)?;
            layers.push(layer);
        }
        Ok(ClipEncoder { layers })
    }

    pub fn forward(&self, xs: &Tensor, causal_attention_mask: Option<&Tensor>) -> Result<Tensor> {
        let mut xs = xs.clone();
        for layer in self.layers.iter() {
            xs = layer.forward(&xs, causal_attention_mask)?;
        }
        Ok(xs)
    }
    // required by LLaVA
    pub fn output_hidden_states(
        &self,
        xs: &Tensor,
        causal_attention_mask: Option<&Tensor>,
    ) -> Result<Vec<Tensor>> {
        let mut xs = xs.clone();
        let mut hidden_states = Vec::new();
        for layer in self.layers.iter() {
            xs = layer.forward(&xs, causal_attention_mask)?;
            hidden_states.push(xs.clone());
        }
        Ok(hidden_states)
    }
}

// https://github.com/huggingface/transformers/blob/f6fa0f0bf0796ac66f201f23bdb8585de1609add/src/transformers/models/clip/modeling_clip.py#L743
#[derive(Clone, Debug)]
pub struct ClipVisionTransformer {
    embeddings: ClipVisionEmbeddings,
    encoder: ClipEncoder,
    pre_layer_norm: candle_nn::LayerNorm,
    // final_layer_norm: candle_nn::LayerNorm,
}

impl ClipVisionTransformer {
    pub fn new(vs: quantized_var_builder::VarBuilder, c: &ClipVisionConfig) -> Result<Self> {
        let embeddings = ClipVisionEmbeddings::new(vs.clone(), c)?;
        let pre_layer_norm = layer_norm(c.hidden_size, 1e-5.into(), vs.pp("pre_ln"))?;
        let encoder = ClipEncoder::new(vs.pp("blk"), c)?;
        // gguf 里面没有 post layernorm
        // let final_layer_norm = layer_norm(c.embed_dim, 1e-5.into(), vs.pp("post_ln"))?;
        Ok(Self {
            embeddings,
            encoder,
            pre_layer_norm,
            // final_layer_norm,
        })
    }
    // required by LLaVA
    pub fn output_hidden_states(&self, pixel_values: &Tensor) -> Result<Vec<Tensor>> {
        let hidden_states = pixel_values
            .apply(&self.embeddings)?
            .apply(&self.pre_layer_norm)?;
        let result = self.encoder.output_hidden_states(&hidden_states, None)?;
        // 见 ClipVisionTower::forward 的说明，llava 其实用不到这一层，索性就不处理了
        // let encoder_outputs = result.last().unwrap();
        // let pooled_output = encoder_outputs.i((.., 0, ..))?;
        // result.push(self.final_layer_norm.forward(&pooled_output)?.clone());
        // result.push(pooled_output);
        Ok(result)
    }
}

impl Module for ClipVisionTransformer {
    fn forward(&self, pixel_values: &Tensor) -> Result<Tensor> {
        let hidden_states = pixel_values
            .apply(&self.embeddings)?
            .apply(&self.pre_layer_norm)?;

        let encoder_outputs = self.encoder.forward(&hidden_states, None)?;
        // https://github.com/huggingface/transformers/blob/f6fa0f0bf0796ac66f201f23bdb8585de1609add/src/transformers/models/clip/modeling_clip.py#L787
        // pooled_output = encoder_outputs[:, 0, :]
        let pooled_output = encoder_outputs.i((.., 0, ..))?;
        // self.final_layer_norm.forward(&pooled_output)
        Ok(pooled_output)
    }
}
