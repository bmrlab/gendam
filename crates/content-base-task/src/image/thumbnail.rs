use super::ImageTaskType;
use crate::{ContentTask, ContentTaskType, TaskRunOutput, TaskRunRecord};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use image::ImageReader;
use serde_json::{json, Value};
use std::path::PathBuf;
use storage_macro::Storage;

#[derive(Clone, Debug, Default, Storage)]
pub struct ImageThumbnailTask;

#[async_trait]
impl ContentTask for ImageThumbnailTask {
    async fn task_output(&self, _task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        Ok(TaskRunOutput::File(PathBuf::from("thumbnail.webp")))
    }

    async fn inner_run(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        let output = self.task_output(task_run_record).await?;
        let output_path = output.to_path_buf(&file_info.file_identifier, ctx).await?;

        let image = ImageReader::open(&file_info.file_path)?
            .with_guessed_format()?
            .decode()?;

        tracing::debug!("image size: {:?} * {:?}", image.width(), image.height());

        let (width, height) = (image.width(), image.height());
        let ratio = f64::min(1024 as f64 / width as f64, 1024 as f64 / height as f64);
        let new_width = (width as f64 * ratio) as u32;
        let new_height = (height as f64 * ratio) as u32;

        // Resize the image
        let thumbnail = image.thumbnail(new_width, new_height);

        let tmp_path = ctx.tmp_dir().join(&file_info.file_identifier);
        if !tmp_path.exists() {
            std::fs::create_dir_all(&tmp_path)?;
        }

        let tmp_path = tmp_path.join("thumbnail.webp");

        tracing::debug!("saving thumbnail to {:?}", tmp_path);

        thumbnail.save_with_format(&tmp_path, image::ImageFormat::WebP)?;
        let data = std::fs::read(&tmp_path)?;
        self.write(output_path, data.into()).await?;
        std::fs::remove_file(tmp_path)?;

        Ok(())
    }

    fn task_parameters(&self, _: &ContentBaseCtx) -> Value {
        json!({
            "max_size": "1024",
        })
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![] as Vec<ContentTaskType>
    }
}

impl Into<ContentTaskType> for ImageThumbnailTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Image(ImageTaskType::Thumbnail(self.clone()))
    }
}
