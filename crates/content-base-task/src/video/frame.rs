use super::VideoTaskType;
use crate::{
    record::{TaskRunOutput, TaskRunRecord},
    ContentTask, ContentTaskType, FileInfo,
};
use async_trait::async_trait;
use content_base_context::ContentBaseCtx;
use content_handler::video::VideoDecoder;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use storage_macro::Storage;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameInfo {
    pub timestamp: i64,
    // image_file is in the format `artifacts/[shard]/[hash]/frames/[timestamp].jpg`
    pub image_file: PathBuf,
}

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
        let fps = 0.1; // TODO: make this configurable
        video_decoder.save_video_frames(output_dir, fps).await?;

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

impl VideoFrameTask {
    pub async fn frame_content(
        &self,
        file_info: &crate::FileInfo,
        ctx: &ContentBaseCtx,
    ) -> anyhow::Result<Vec<FrameInfo>> {
        let task_type: ContentTaskType = self.clone().into();
        // output_path is in the format `artifacts/[shard]/[hash]/frames`
        let output_path = task_type.task_output_path(file_info, ctx).await?;

        let mut frames: Vec<FrameInfo> = Vec::new();
        let dir_entries = self.read_dir(output_path.clone()).await?;
        for entry in dir_entries {
            // if entry.is_file()
            let file_name = match entry.file_name() {
                Some(name) => name,
                None => continue,
            };
            let file_name_str = file_name.to_string_lossy();
            if let Some(timestamp) = file_name_str.strip_suffix(".jpg") {
                let image_file = output_path.clone().join(file_name_str.to_string());
                if let Ok(ts) = timestamp.parse::<i64>() {
                    frames.push(FrameInfo {
                        timestamp: ts,
                        image_file,
                    });
                }
            }
        }
        frames.sort_by_key(|frame| frame.timestamp);

        Ok(frames)
    }
}
