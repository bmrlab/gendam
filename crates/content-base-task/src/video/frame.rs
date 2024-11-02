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

/// TODO: 优化，应该是对 frames 进行关键帧拆分以后形成不同的 chunk 而不是固定数量
/// 这里设置的时候要注意，最好是 n^2，因为 llava phi 3 模型会把图片拼成 n * n grids 作为一个图片输入
pub const VIDEO_FRAME_SUMMARY_BATCH_SIZE: usize = 9;

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
        let video_decoder = VideoDecoder::new(&file_info.file_full_path_on_disk)?;
        let tmp_dir = ctx
            .tmp_dir()
            .join(format!("{}/frames", file_info.file_identifier));
        std::fs::create_dir_all(&tmp_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create tmp dir: {e}"))?;
        // TODO: 需要可以配置
        // 但是要注意如果不是每一秒都有 frame 就要处理搜索结果里通过语音定位的 frame
        // 要四舍五入到 frame_interval_seconds 的整数倍
        let frame_interval_seconds = 1;
        video_decoder
            .save_video_frames(&tmp_dir, &output_dir, frame_interval_seconds)
            .await?;
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
        file_identifier: &str,
        ctx: &ContentBaseCtx,
    ) -> anyhow::Result<Vec<FrameInfo>> {
        let task_type: ContentTaskType = self.clone().into();
        // output_path is in the format `artifacts/[shard]/[hash]/frames`
        let output_path = task_type.task_output_path(file_identifier, ctx).await?;

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
