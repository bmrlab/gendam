use super::VideoTaskType;
use crate::{
    record::{TaskRunOutput, TaskRunRecord},
    ContentTask, ContentTaskType, FileInfo,
};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use content_handler::video::VideoDecoder;
use serde_json::{json, Value};
use std::path::PathBuf;
use storage_macro::Storage;

#[derive(Clone, Storage, Debug, Default)]
pub struct VideoFrameTask;

#[async_trait]
impl ContentTask for VideoFrameTask {
    async fn task_output(&self, _task_run_record: &TaskRunRecord) -> anyhow::Result<TaskRunOutput> {
        Ok(TaskRunOutput::Folder(PathBuf::from("frames")))
    }

    async fn inner_run(
        &self,
        file_info: &FileInfo,
        ctx: &ContentBaseCtx,
        task_run_record: &mut TaskRunRecord,
    ) -> anyhow::Result<()> {
        let output = self.task_output(task_run_record).await?;
        let output_dir = output.to_path_buf(&file_info.file_identifier, ctx).await?;

        let video_decoder = VideoDecoder::new(&file_info.file_path)?;
        video_decoder.save_video_frames(output_dir).await?;

        Ok(())
    }

    fn task_parameters(&self, _: &ContentBaseCtx) -> Value {
        json!({
            "method": "ffmpeg",
        })
    }

    fn task_dependencies(&self) -> Vec<ContentTaskType> {
        vec![] as Vec<ContentTaskType>
    }
}

impl Into<ContentTaskType> for VideoFrameTask {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Video(VideoTaskType::Frame(self.clone()))
    }
}
