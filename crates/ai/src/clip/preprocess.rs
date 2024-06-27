use image::RgbImage;
use ndarray::Array3;
use std::path::Path;

const TARGET_IMAGE_SIZE: u32 = 224;

// 这个方法没用了，现在都通过 opendal 先读取到内存里再继续用
pub fn _read_image(image_path: impl AsRef<Path>) -> anyhow::Result<RgbImage> {
    let image = image::open(image_path)?;
    Ok(image.to_rgb8())
}

pub fn preprocess_rgb8_image(image: &RgbImage) -> anyhow::Result<Array3<f32>> {
    // resize image
    let (w, h) = image.dimensions();
    let w = w as f32;
    let h = h as f32;
    let (w, h) = if w < h {
        (
            TARGET_IMAGE_SIZE,
            ((TARGET_IMAGE_SIZE as f32) * h / w) as u32,
        )
    } else {
        (
            ((TARGET_IMAGE_SIZE as f32) * w / h) as u32,
            TARGET_IMAGE_SIZE,
        )
    };

    let mut image = image::imageops::resize(image, w, h, image::imageops::FilterType::CatmullRom);

    // center crop the image
    let left = (w - TARGET_IMAGE_SIZE) / 2;
    let top = (h - TARGET_IMAGE_SIZE) / 2;
    let image = image::imageops::crop(&mut image, left, top, TARGET_IMAGE_SIZE, TARGET_IMAGE_SIZE)
        .to_image();

    // normalize according to CLIP
    let mut array = Array3::zeros((3, TARGET_IMAGE_SIZE as usize, TARGET_IMAGE_SIZE as usize));

    for i in 0..TARGET_IMAGE_SIZE {
        for j in 0..TARGET_IMAGE_SIZE {
            let p = image.get_pixel(j, i);

            array[[0, i as usize, j as usize]] = (p[0] as f32 / 255.0 - 0.48145466) / 0.26862954;
            array[[1, i as usize, j as usize]] = (p[1] as f32 / 255.0 - 0.4578275) / 0.26130258;
            array[[2, i as usize, j as usize]] = (p[2] as f32 / 255.0 - 0.40821073) / 0.27577711;
        }
    }

    Ok(array)
}
