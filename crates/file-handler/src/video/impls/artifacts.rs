use std::{collections::HashMap, path::PathBuf};

use anyhow::bail;
use uuid::Uuid;

use crate::{
    artifacts::{ArtifactsResult, ArtifactsSettings},
    video::{VideoHandler, VideoTaskType, ARTIFACTS_SETTINGS_FILE_NAME},
};

impl VideoHandler {
    fn get_artifacts_settings(&self) -> ArtifactsSettings {
        match std::fs::read_to_string(self.artifacts_dir().join(ARTIFACTS_SETTINGS_FILE_NAME)) {
            std::result::Result::Ok(json_content) => {
                if let std::result::Result::Ok(settings) =
                    serde_json::from_str::<ArtifactsSettings>(&json_content)
                {
                    settings
                } else {
                    ArtifactsSettings::default()
                }
            }
            Err(_) => ArtifactsSettings::default(),
        }
    }

    fn set_artifacts_settings(&self, artifacts_settings: ArtifactsSettings) -> anyhow::Result<()> {
        std::fs::write(
            self.artifacts_dir.join(ARTIFACTS_SETTINGS_FILE_NAME),
            serde_json::to_string(&artifacts_settings)?,
        )?;

        Ok(())
    }

    fn get_task_parent_path_list(&self, task_type: &VideoTaskType) -> Vec<ArtifactsResult> {
        task_type
            .get_parent_task()
            .iter()
            .map(|v| self.get_output_info_in_settings(v).unwrap_or_default())
            .collect::<Vec<_>>()
    }

    pub fn get_output_info_in_settings(
        &self,
        task_type: &VideoTaskType,
    ) -> anyhow::Result<ArtifactsResult> {
        let settings = self.get_artifacts_settings();

        let current_model_name = settings
            .models
            .get(&task_type.to_string())
            .map(|v| v.clone());

        let input_path = self
            .get_task_parent_path_list(&task_type)
            .iter()
            .map(|v| v.dir.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(":");

        match current_model_name {
            Some(current_model_name) => {
                let key = format!("{}:{}", &current_model_name, input_path);

                if let Some(Some(output_path)) = settings
                    .results
                    .get(&task_type.to_string())
                    .map(|v| v.get(&key))
                {
                    return Ok(output_path.to_owned());
                }

                bail!("output path not found in settings")
            }
            _ => bail!("output path not found in settings"),
        }
    }

    pub fn set_default_output_path(&self, task_type: &VideoTaskType) -> anyhow::Result<()> {
        let mut settings = self.get_artifacts_settings();

        let current_model_name = match task_type {
            VideoTaskType::FrameContentEmbedding => {
                self.multi_modal_embedding().ok().map(|v| v.1.to_string())
            }
            VideoTaskType::FrameCaption => self.image_caption().ok().map(|v| v.1.to_string()),
            VideoTaskType::FrameCaptionEmbedding | VideoTaskType::TranscriptEmbedding => {
                self.text_embedding().ok().map(|v| v.1.to_string())
            }
            VideoTaskType::Transcript => self.audio_transcript().ok().map(|v| v.1.to_string()),
            _ => Some("".into()),
        }
        .ok_or(anyhow::anyhow!("model not found"))?;

        settings
            .models
            .insert(task_type.to_string(), current_model_name.clone());

        let output_dir = PathBuf::from(match task_type {
            // 针对 frames 做一下特殊处理，因为前端需要读取里面的内容，不方便通过接口获取
            VideoTaskType::Frame => "frames".into(),
            _ => format!("{}-{}", &task_type.to_string(), Uuid::new_v4()),
        });

        let input_path = self
            .get_task_parent_path_list(task_type)
            .iter()
            .map(|v| v.dir.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(":");

        let key = format!("{}:{}", &current_model_name, input_path);

        let output_dir = match settings.results.get_mut(&task_type.to_string()) {
            Some(model_results) => match model_results.get_mut(&key) {
                Some(result) => result.dir.to_owned(),
                _ => {
                    model_results.insert(
                        key.to_string(),
                        ArtifactsResult {
                            dir: output_dir.clone(),
                            files: None,
                        },
                    );
                    output_dir
                }
            },
            _ => {
                let mut model_results = HashMap::new();
                model_results.insert(
                    key.to_string(),
                    ArtifactsResult {
                        dir: output_dir.clone(),
                        files: None,
                    },
                );
                settings
                    .results
                    .insert(task_type.to_string(), model_results);
                output_dir
            }
        };

        let output_dir = self.artifacts_dir.join(PathBuf::from(output_dir));
        if !output_dir.exists() {
            std::fs::create_dir_all(&output_dir)?;
        }

        self.set_artifacts_settings(settings)?;

        Ok(())
    }

    pub fn set_artifacts_result(&self, task_type: &VideoTaskType) -> anyhow::Result<()> {
        let output_info = self.get_output_info_in_settings(task_type)?;

        let artifacts_result = std::fs::read_dir(self.artifacts_dir.join(&output_info.dir))?
            .into_iter()
            .filter_map(|v| v.ok().map(|t| t.path()))
            .filter_map(|v| {
                v.file_name()
                    .map(|t| PathBuf::from(PathBuf::from(t.to_os_string())))
            })
            .collect::<Vec<_>>();

        let mut settings = self.get_artifacts_settings();

        let current_model_name = settings
            .models
            .get(&task_type.to_string())
            .map(|v| v.clone());

        let input_path = self
            .get_task_parent_path_list(&task_type)
            .iter()
            .map(|v| v.dir.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(":");

        match current_model_name {
            Some(current_model_name) => {
                let key = format!("{}:{}", &current_model_name, input_path);
                if let Some(Some(output_path)) = settings
                    .results
                    .get_mut(&task_type.to_string())
                    .map(|v| v.get_mut(&key))
                {
                    output_path.files = Some(artifacts_result);
                } else {
                    bail!("output path not found in settings")
                }
            }
            _ => bail!("output path not found in settings"),
        }

        self.set_artifacts_settings(settings)?;

        Ok(())
    }

    pub fn check_artifacts(&self, task_type: &VideoTaskType) -> bool {
        match self.get_output_info_in_settings(task_type) {
            std::result::Result::Ok(ArtifactsResult {
                dir,
                files: Some(files),
            }) => {
                for file_name in files {
                    if !self.artifacts_dir.join(&dir).join(&file_name).exists() {
                        return false;
                    }
                }

                true
            }
            _ => false,
        }
    }

    pub(crate) fn _delete_artifacts_by_task(
        &self,
        task_type: &VideoTaskType,
    ) -> anyhow::Result<()> {
        let mut settings = self.get_artifacts_settings();
        let output_path_map = settings.results.remove(&task_type.to_string());
        if let Some(output_path_map) = output_path_map {
            for output_path in output_path_map.values() {
                std::fs::remove_dir_all(self.artifacts_dir.join(output_path.dir.to_owned()))?;
            }
        }

        self.set_artifacts_settings(settings)?;

        let child_task_type_list = task_type.get_child_task();
        for task_type in child_task_type_list {
            self._delete_artifacts_by_task(&task_type)?;
        }

        Ok(())
    }
}
