use candle_core::{DType, Device, Result, Tensor};
use image::{DynamicImage, GenericImageView};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HFPreProcessorConfig {
    // pub aspect_ratio_setting: String,
    pub crop_size: HashMap<String, usize>,
    pub do_center_crop: bool,
    pub do_convert_rgb: bool,
    pub do_normalize: bool,
    pub do_rescale: bool,
    pub do_resize: bool,
    pub image_mean: Vec<f32>,
    pub image_std: Vec<f32>,
    pub resample: u32,
    pub rescale_factor: f32,
    pub size: HashMap<String, f32>,
}

//This struct is mainly for LLaVA aplications, hence it's not completely compatible with python transformer CLIPImageProcessor  few several preprocess that LLaVA used, including "openai/clip-vit-large-patch14-336" and "openai/clip-vit-large-patch14".
#[derive(Serialize, Deserialize, Debug)]
pub struct ImageProcessor {
    #[serde(default = "default_size")]
    pub size: u32, // this is not the same as python transformer
    #[serde(default = "default_do_resize")]
    pub do_resize: bool,
    //resample: u32 // 3 for PIL bicubic, equivalent to rust  CatmullRom. Hence below we use CatmullRom
    #[serde(default = "default_do_center_crop")]
    pub do_center_crop: bool,
    #[serde(default = "default_crop_size")]
    pub crop_size: u32, // this is not the same as python transformer
    #[serde(default = "default_do_rescale")]
    pub do_rescale: bool,
    #[serde(default = "default_rescale_factor")]
    pub rescale_factor: f32,
    #[serde(default = "default_do_normalize")]
    pub do_normalize: bool,
    #[serde(default = "default_image_mean")]
    pub image_mean: Vec<f32>,
    #[serde(default = "default_image_std")]
    pub image_std: Vec<f32>,
}

fn default_size() -> u32 {
    224
}

fn default_do_resize() -> bool {
    true
}

fn default_do_center_crop() -> bool {
    true
}

fn default_crop_size() -> u32 {
    224
}

fn default_do_rescale() -> bool {
    true
}

fn default_rescale_factor() -> f32 {
    1.0 / 255.0
}

fn default_do_normalize() -> bool {
    true
}

fn default_image_mean() -> Vec<f32> {
    vec![0.48145466, 0.4578275, 0.40821073]
}

fn default_image_std() -> Vec<f32> {
    vec![0.26862954, 0.2613026, 0.2757771]
}

impl ImageProcessor {
    pub fn from_hf_preprocessor_config(hf_preprocessor_config: &HFPreProcessorConfig) -> Self {
        Self {
            size: hf_preprocessor_config.size["shortest_edge"] as u32,
            do_resize: hf_preprocessor_config.do_resize,
            do_center_crop: hf_preprocessor_config.do_center_crop,
            crop_size: hf_preprocessor_config.crop_size["height"] as u32,
            do_rescale: hf_preprocessor_config.do_rescale,
            rescale_factor: hf_preprocessor_config.rescale_factor,
            do_normalize: hf_preprocessor_config.do_normalize,
            image_mean: hf_preprocessor_config.image_mean.clone(),
            image_std: hf_preprocessor_config.image_std.clone(),
        }
    }

    ///shortest edge to self.resize, other edge is resized to maintain aspect ratio
    pub fn resize(&self, image: &DynamicImage) -> DynamicImage {
        let (width, height) = image.dimensions();
        let size = self.size;
        if width == size && height == size {
            image.clone()
        } else {
            let (new_width, new_height) = if width < height {
                (
                    size,
                    (((size * height) as f32) / width as f32).ceil() as u32,
                )
            } else {
                (
                    (((size * width) as f32) / height as f32).ceil() as u32,
                    size,
                )
            };
            image.resize(
                new_width,
                new_height,
                image::imageops::FilterType::CatmullRom,
            )
        }
    }

    pub fn center_crop(&self, image: &DynamicImage) -> DynamicImage {
        let (width, height) = image.dimensions();
        let crop_size = self.crop_size;
        let (left, top) = calculate_middle((width, height), (crop_size, crop_size));
        image.crop_imm(left, top, crop_size, crop_size)
    }

    pub fn to_tensor(&self, image: &DynamicImage) -> Result<Tensor> {
        let img = image.to_rgb8().into_raw();
        let (width, height) = image.dimensions();
        Tensor::from_vec(img, (height as usize, width as usize, 3), &Device::Cpu)?
            .to_dtype(DType::F32) // only for internal compute
    }

    pub fn rescale(&self, tensor: &Tensor) -> Result<Tensor> {
        let rescale_factor = self.rescale_factor as f64;
        tensor.affine(rescale_factor, 0.0)
    }

    pub fn normalize(&self, tensor: &Tensor) -> Result<Tensor> {
        let image_mean = self.image_mean.clone();
        let image_std = self.image_std.clone();
        let mean = Tensor::from_vec(image_mean, (3,), &Device::Cpu)?;
        let std = Tensor::from_vec(image_std, (3,), &Device::Cpu)?;
        tensor.broadcast_sub(&mean)?.broadcast_div(&std)
    }

    pub fn to_channel_dimension_format(&self, tensor: &Tensor) -> Result<Tensor> {
        tensor.permute((2, 0, 1))
    }

    pub fn preprocess(&self, image: &DynamicImage) -> Result<Tensor> {
        let image = if self.do_resize {
            self.resize(image)
        } else {
            image.clone()
        };
        let image = if self.do_center_crop {
            self.center_crop(&image)
        } else {
            image
        };
        let tensor = self.to_tensor(&image)?;
        let tensor = if self.do_rescale {
            self.rescale(&tensor)?
        } else {
            tensor
        };
        let tensor = if self.do_normalize {
            self.normalize(&tensor)?
        } else {
            tensor
        };
        self.to_channel_dimension_format(&tensor)
    }
}

pub fn calculate_middle(image_size: (u32, u32), center_size: (u32, u32)) -> (u32, u32) {
    let (width, height) = image_size;
    let (center_width, center_height) = center_size;
    let left = if width <= center_width {
        0
    } else {
        ((width as f32 - center_width as f32) / 2.0).ceil() as u32
    };
    let top = if height <= center_height {
        0
    } else {
        ((height as f32 - center_height as f32) / 2.0).ceil() as u32
    };
    (left, top)
}
