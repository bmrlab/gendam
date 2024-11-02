use super::{
    clip::{ClipVisionConfig, ClipVisionTransformer},
    config::LLAVA_PHI3_CONFIG,
    linear::QLinear,
    quantized_llama::{self, LlamaTextConfig},
    sequential::{seq, QSequential, QSequentialLayer},
};
use candle_core::{quantized::gguf_file, Device, IndexOp, Module, Tensor};

use candle_transformers::quantized_var_builder;
use std::path::Path;
use tokenizers::Tokenizer;

#[derive(Debug, Clone)]
pub struct LLaVAPhi3Config {
    pub eos_token_id: u32,
    pub bos_token_id: u32,
    pub image_token_id: u32,
    pub llama_text_config: LlamaTextConfig,
    pub clip_vision_config: ClipVisionConfig,
}

#[derive(Debug)]
struct MMProjector {
    pub modules: QSequential,
}

impl MMProjector {
    pub fn new(
        vb: quantized_var_builder::VarBuilder,
        config: &LLaVAPhi3Config,
    ) -> candle_core::Result<Self> {
        let text_hidden_size: usize = config.llama_text_config.hidden_size; // 3072
        let mm_hidden_size: usize = config.clip_vision_config.hidden_size; // 1024
        let modules = {
            let layer = QLinear::load(mm_hidden_size, text_hidden_size, vb.pp("0"))?;
            let mut modules = seq().add(QSequentialLayer::QLinear(layer));
            let mlp_depth = 2;
            for i in 1..mlp_depth {
                let layer = QLinear::load(
                    text_hidden_size,
                    text_hidden_size,
                    vb.pp(format!("{}", i * 2)),
                )?;
                modules = modules
                    .add(QSequentialLayer::Activation(candle_nn::Activation::Gelu))
                    .add(QSequentialLayer::QLinear(layer));
            }
            modules
        };
        Ok(Self { modules })
    }

    #[tracing::instrument(level = "info", name = "MMProjector", skip_all)]
    pub fn forward(&self, x: &Tensor) -> candle_core::Result<Tensor> {
        self.modules.forward(x)
    }
}

#[derive(Debug)]
struct ClipVisionTower {
    model: ClipVisionTransformer,
    #[allow(dead_code)]
    pub config: ClipVisionConfig,
}

impl ClipVisionTower {
    pub fn new(
        vb: quantized_var_builder::VarBuilder,
        clip_vision_config: &ClipVisionConfig,
    ) -> candle_core::Result<Self> {
        // let clip_vision_config = ClipVisionConfig::clip_vit_large_patch14_336();
        let vision_model = ClipVisionTransformer::new(vb, &clip_vision_config)?;
        Ok(Self {
            model: vision_model,
            config: clip_vision_config.clone(),
        })
    }

    #[tracing::instrument(level = "info", name = "ClipVisionTower", skip_all)]
    pub fn forward(&self, x: &Tensor) -> candle_core::Result<Tensor> {
        let result = self.model.output_hidden_states(x)?;
        // llava 模型都默认取的 clip 倒数第二层作为 image 特征
        // 为了方便，模型文件里会把最后一层也加上，这样加载模型的代码不用改，
        // gguf 里专门把它去掉了，所以直接取最后一层就行了
        // let select_layer = -2;
        // let index = result.len() as isize + select_layer;
        // let result = result[index as usize].clone();
        let result = result
            .last()
            .ok_or(candle_core::Error::Msg("No hidden states".to_string()))?
            .clone();
        Ok(result.i((.., 1..))?)
    }
}

#[derive(Debug)]
pub struct QLLaVAPhi3 {
    pub llama: quantized_llama::ModelWeights,
    clip_vision_tower: ClipVisionTower,
    mm_projector: MMProjector,
    pub tokenizer: Tokenizer,
    pub config: LLaVAPhi3Config,
}

impl QLLaVAPhi3 {
    pub fn load(
        device: &Device,
        gguf_model_path: impl AsRef<Path>,
        mmproj_gguf_model_path: impl AsRef<Path>,
        tokenizer_path: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let config = LLAVA_PHI3_CONFIG.clone();
        let llama = {
            let mut file = std::fs::File::open(&gguf_model_path)?;
            let gguf_content =
                gguf_file::Content::read(&mut file).map_err(|e| e.with_path(gguf_model_path))?;
            quantized_llama::ModelWeights::from_gguf(gguf_content, &mut file, &device)?
        };

        let (clip_vision_tower, mm_projector) = {
            let vb = quantized_var_builder::VarBuilder::from_gguf(mmproj_gguf_model_path, &device)?;
            let mm_projector = MMProjector::new(vb.pp("mm"), &config)?;
            let clip_vision_tower = ClipVisionTower::new(vb.pp("v"), &config.clip_vision_config)?;
            (clip_vision_tower, mm_projector)
        };

        let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(anyhow::Error::msg)?;
        Ok(Self {
            llama,
            clip_vision_tower,
            mm_projector,
            tokenizer,
            config,
        })
    }

    /// image 564 564
    /// x shape:                        Tensor[dims 1, 3, 336, 336; bf16]
    /// clip_vision_tower result shape: Tensor[dims 1, 576, 1024; bf16]
    /// mm_projector result shape:      Tensor[dims 1, 576, 3072; bf16]
    fn encode_images(&self, x: &Tensor) -> candle_core::Result<Tensor> {
        let image_features = self.clip_vision_tower.forward(x)?;
        // let image_features = self.clip_vision_tower.forward(&x.to_dtype(DType::F32)?)?;
        let image_features = self.mm_projector.forward(&image_features)?;
        // let image_features = image_features.to_dtype(DType::BF16)?;
        Ok(image_features)
    }

    #[tracing::instrument(level = "info", skip_all)]
    pub fn prepare_inputs_labels_for_multimodal(
        &self,
        device: &Device,
        input_ids: &Tensor,
        images_tensors: &[Tensor],
        _image_sizes: &[(u32, u32)],
    ) -> candle_core::Result<Tensor> {
        let concat_images = Tensor::cat(images_tensors, 0)?;
        let image_features_together = self.encode_images(&concat_images)?;
        let split_sizes = images_tensors
            .iter()
            .map(|x| x.shape().dims()[0])
            .collect::<Vec<usize>>();
        // can be replaced by split
        let mut index_pos = 0;
        let mut image_features = Vec::new();
        for split_size in split_sizes.iter() {
            image_features.push(image_features_together.i(index_pos..index_pos + (*split_size))?);
            index_pos += *split_size;
        }

        // mm_patch_merge_type is "flat"
        let image_features = image_features
            .iter()
            .map(|x| x.flatten(0, 1).unwrap())
            .collect::<Vec<Tensor>>();

        let input_ids_vec = input_ids.squeeze(0)?.to_vec1::<i64>()?;
        let mut image_indices = {
            let mut image_indices = vec![0_i64];
            image_indices.extend(
                input_ids_vec
                    .iter()
                    .enumerate()
                    .filter_map(|(i, x)| {
                        if *x == self.config.image_token_id as i64 {
                            Some(i as i64)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<i64>>(),
            );
            image_indices
        };

        // if image_indices.len() == 1 {
        //     //no image, only [0],
        //     return self.llama.embed(input_ids);
        // }

        let input_ids_noim = input_ids_vec
            .iter()
            .filter_map(|x| {
                if *x != self.config.image_token_id as i64 {
                    Some(*x)
                } else {
                    None
                }
            })
            .collect::<Vec<i64>>();
        let input_ids_noim_len = input_ids_noim.len();
        image_indices.push((input_ids_noim_len) as i64);
        let input_ids_noim = Tensor::from_vec(input_ids_noim, input_ids_noim_len, &device)?;
        // println!("input_ids_noim: {:?}", input_ids_noim);
        let cur_input_embeds = self.llama.embed(&input_ids_noim)?;
        // println!("cur_input_embeds: {:?}", cur_input_embeds);
        // can be replace by split if it is implemented in candle
        let input_embed_no_ims = {
            let mut input_embeds = Vec::new();
            for i in 0..image_indices.len() - 1 {
                let start = (image_indices[i]) as usize;
                let end = image_indices[i + 1] as usize;
                input_embeds.push(cur_input_embeds.i((start..end, ..))?)
            }
            input_embeds
        };
        let mut cur_new_input_embeds = Vec::new();
        for (i, image_feature) in image_features.iter().enumerate() {
            cur_new_input_embeds.push(input_embed_no_ims[i].clone());
            // 如果 encode_images 还没实现, 这里先不放进去, 不然 shape 不对
            cur_new_input_embeds.push(image_feature.clone());
        }
        cur_new_input_embeds.push(input_embed_no_ims[image_features.len()].clone());
        let new_input_embeds = Tensor::cat(&cur_new_input_embeds, 0)?;

        // trancate
        let tokenizer_model_max_length = Some(self.config.llama_text_config.max_length);
        let new_input_embeds = if let Some(tokenizer_model_max_length) = tokenizer_model_max_length
        {
            let (new_input_embeds_length, _) = new_input_embeds.shape().dims2()?;
            if new_input_embeds_length > tokenizer_model_max_length {
                new_input_embeds.i((..tokenizer_model_max_length, ..))?
            } else {
                new_input_embeds
            }
        } else {
            new_input_embeds
        };

        Ok(new_input_embeds.unsqueeze(0)?)
    }

    /// Input prompt: "A photo of <image> next to <image>"
    /// Output: [bos_token_id, ...(tokens for "A photo of"), image_token_id, ...(tokens for " next to "), image_token_id]
    pub fn tokenizer_image_token(&self, prompt: &str) -> candle_core::Result<Tensor> {
        let prompt_chunks = prompt
            .split("<image>")
            .map(|s| {
                self.tokenizer
                    .encode(s, true)
                    .unwrap()
                    .get_ids()
                    .to_vec()
                    .iter()
                    .map(|x| *x as i64)
                    .collect()
            })
            .collect::<Vec<Vec<i64>>>();
        let mut input_ids = Vec::new();
        let mut offset = 0;
        if !prompt_chunks.is_empty()
            && !prompt_chunks[0].is_empty()
            && prompt_chunks[0][0] == self.config.bos_token_id as i64
        {
            offset = 1;
            input_ids.push(prompt_chunks[0][0]);
        }

        for x in insert_separator(
            prompt_chunks,
            duplicate_vec(&[self.config.image_token_id as i64], offset + 1),
        )
        .iter()
        {
            input_ids.extend(x[1..].to_vec())
        }
        // println!("input_ids: {:?}", input_ids);
        let input_len = input_ids.len();
        Tensor::from_vec(input_ids, (1, input_len), &Device::Cpu)
    }
}

fn duplicate_vec<T>(vec: &[T], n: usize) -> Vec<T>
where
    T: Clone,
{
    let mut res = Vec::new();
    for _ in 0..n {
        res.extend(vec.to_owned());
    }
    res
}

fn insert_separator<T>(x: Vec<Vec<T>>, sep: Vec<T>) -> Vec<Vec<T>>
where
    T: Clone,
{
    let sep = vec![sep];
    let sep = duplicate_vec(&sep, x.len());
    let mut res = x
        .iter()
        .zip(sep.iter())
        .flat_map(|(x, y)| vec![x.clone(), y.clone()])
        .collect::<Vec<Vec<T>>>();
    res.pop();
    res
}

pub fn format_prompt(prompt: &str, images_count: usize) -> String {
    let images_tags = (0..images_count)
        .map(|_| "<image>\n")
        .collect::<Vec<&str>>()
        .join("");
    format!(
        "<s><|system|>\n<|end|>\n<|user|>\n{image_tags}{text_msg}<|end|>\n<|assistant|>\n",
        image_tags = images_tags.as_str(),
        text_msg = prompt,
    )
}
