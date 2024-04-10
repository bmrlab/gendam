use anyhow::anyhow;
use candle_core::{IndexOp, Tensor};
use candle_transformers::object_detection::{non_maximum_suppression, Bbox, KeyPoint};
use image::GenericImageView;
use ndarray::{s, Array3, Axis, Ix3};
use ort::{CPUExecutionProvider, CoreMLExecutionProvider, GraphOptimizationLevel, Session};
use std::path::Path;

pub(self) mod coco_classes;

pub struct YOLO {
    model: Session,
    confidence_threshold: f32,
    nms_threshold: f32,
}

pub struct YOLODetectionResult {
    class_name: String,
    bounding_box: Bbox<Vec<KeyPoint>>,
}

impl YOLO {
    pub async fn new(resources_dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let model_uri = "yolo/yolov8x.onnx";
        let download = file_downloader::FileDownload::new(file_downloader::FileDownloadConfig {
            resources_dir: resources_dir.as_ref().to_path_buf(),
            ..Default::default()
        });

        let model_path = download.download_if_not_exists(&model_uri).await?;
        let model = Session::builder()?
            .with_execution_providers([
                CPUExecutionProvider::default().build(),
                CoreMLExecutionProvider::default().build(),
            ])?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(16)?
            .commit_from_file(model_path)?;

        Ok(Self {
            model,
            confidence_threshold: 0.5,
            nms_threshold: 0.45,
        })
    }

    pub async fn detect(
        &self,
        image_path: impl AsRef<Path>,
    ) -> anyhow::Result<Vec<YOLODetectionResult>> {
        // refer the pipeline to candle:
        // https://github.com/huggingface/candle/blob/main/candle-examples/examples/yolo-v8/main.rs
        // the only exception is that here we use onnx to inference
        // but preprocessing of input image and postprocessing(NMS) is the same
        let original_image = image::open(image_path)?;
        let (width, height) = {
            let w = original_image.width() as usize;
            let h = original_image.height() as usize;
            if w < h {
                let w = w * 640 / h;
                // Sizes have to be divisible by 32.
                (w / 32 * 32, 640)
            } else {
                let h = h * 640 / w;
                (640, h / 32 * 32)
            }
        };
        let img = original_image.resize_exact(
            width as u32,
            height as u32,
            image::imageops::FilterType::CatmullRom,
        );

        let mut array = Array3::zeros((3, img.height() as usize, img.width() as usize));
        for i in 0..img.width() {
            for j in 0..img.height() {
                let p = img.get_pixel(i, j);

                array[[0, j as usize, i as usize]] = p[0] as f32 / 255.0;
                array[[1, j as usize, i as usize]] = p[1] as f32 / 255.0;
                array[[2, j as usize, i as usize]] = p[2] as f32 / 255.0;
            }
        }
        let array = array.insert_axis(Axis(0));
        let outputs = self.model.run(ort::inputs!["images" => array.view()]?)?;

        let output = outputs
            .get("output0")
            .ok_or(anyhow!("output not found"))?
            .try_extract_tensor::<f32>()?
            .view()
            .to_owned();

        // here output is in shape (batch_size, category_num, anchor_num)
        // we need to run NMS on the results
        let output = output.into_dimensionality::<Ix3>()?;
        let output = output.slice_move(s![0, .., ..]);
        let output_shape = output.shape().to_owned();

        let pred = Tensor::from_iter(output, &candle_core::Device::Cpu)?;
        let pred = pred.reshape(output_shape)?;
        let (pred_size, npreds) = pred.dims2()?;
        let nclasses = pred_size - 4;
        let mut bboxes: Vec<Vec<Bbox<Vec<KeyPoint>>>> = (0..nclasses).map(|_| vec![]).collect();
        // Extract the bounding boxes for which confidence is above the threshold.
        for index in 0..npreds {
            let pred = Vec::<f32>::try_from(pred.i((.., index))?)?;
            let confidence = *pred[4..].iter().max_by(|x, y| x.total_cmp(y)).unwrap();
            if confidence > self.confidence_threshold {
                let mut class_index = 0;
                for i in 0..nclasses {
                    if pred[4 + i] > pred[4 + class_index] {
                        class_index = i
                    }
                }
                if pred[class_index + 4] > 0. {
                    let bbox = Bbox {
                        xmin: pred[0] - pred[2] / 2.,
                        ymin: pred[1] - pred[3] / 2.,
                        xmax: pred[0] + pred[2] / 2.,
                        ymax: pred[1] + pred[3] / 2.,
                        confidence,
                        data: vec![],
                    };
                    bboxes[class_index].push(bbox)
                }
            }
        }

        non_maximum_suppression(&mut bboxes, self.nms_threshold);

        let mut results = vec![];

        bboxes
            .iter()
            .enumerate()
            .for_each(|(class_index, boxes_list)| {
                boxes_list.iter().for_each(|bbox| {
                    results.push(YOLODetectionResult {
                        class_name: coco_classes::NAMES[class_index].to_string(),
                        bounding_box: bbox.clone(),
                    })
                })
            });

        Ok(results)
    }
}

#[test_log::test(tokio::test)]
async fn test_yolo() {
    let yolo = YOLO::new(
        "/Users/zhuo/dev/tezign/bmrlab/tauri-dam-test-playground/apps/desktop/src-tauri/resources",
    )
    .await
    .expect("failed to load model");

    match yolo.detect("/Users/zhuo/Library/Application Support/cc.musedam.local/libraries/cd559e25-c136-4877-825e-84268a53e366/artifacts/5ad/5adec370b6040f26/frames/1000.jpg").await {
        Ok(result) => {
            tracing::info!("detection result:");
            result.iter().for_each(|r| {
                tracing::info!("{}: {}", r.class_name, r.bounding_box.confidence);
            });
        }
        Err(e) => {
            tracing::error!("error: {:?}", e);
        }
    };
}
