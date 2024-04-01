use crate::routes::audio::{downloader::DownloadHelper, reader::AudioReader};
use crate::CtxWithLibrary;
use file_handler::video::TRANSCRIPT_FILE_NAME;
use rspc::{Router, RouterBuilder};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::{fmt, path::PathBuf};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tracing::log::debug;
use tracing::{error, warn};

pub mod downloader;
pub mod reader;

#[derive(Debug, Deserialize, Serialize, Clone, Type)]
struct ExportInput {
    #[serde(rename = "types")]
    type_group: Vec<AudioType>,
    hash: String,
    path: String,
    /// 保存的文件名，不包含文件后缀
    #[serde(rename = "fileName")]
    #[specta(optional)]
    file_name: Option<String>,
}

pub fn get_routes<TCtx>() -> RouterBuilder<TCtx>
where
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    Router::<TCtx>::new()
        .query("find_by_hash", |t| {
            t(|ctx, hash: String| async move {
                let library = ctx.library()?;
                let artifacts_dir = library.artifacts_dir(&hash);
                let path = artifacts_dir.join(TRANSCRIPT_FILE_NAME);
                Ok(get_all_audio_format(path))
            })
        })
        .mutation("export", |t| {
            t(|ctx, input: ExportInput| async move {
                let library = ctx.library()?;
                let export_result = audio_export(library.artifacts_dir(&input.hash), input)
                    .unwrap_or_else(|err| {
                        error!("Failed to export audio: {err}",);
                        vec![]
                    });
                Ok(export_result)
            })
        })
        .mutation("batch_export", |t| {
            t(|ctx, input: Vec<ExportInput>| async move {
                let library = ctx.library()?;
                let mut error_list = vec![];
                for item in input {
                    let res = audio_export(library.artifacts_dir(&item.hash), item).unwrap_or_else(
                        |err| {
                            error!("Failed to export audio: {err}",);
                            vec![]
                        },
                    );
                    error_list.extend(res);
                }
                Ok(error_list)
            })
        })
}

#[derive(EnumIter, Debug, Deserialize, Serialize, Clone, Type, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
enum AudioType {
    Txt,
    Srt,
    Json,
    Vtt,
    Csv,
    Ale,
    Docx,
}

impl fmt::Display for AudioType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Type)]
struct AudioResp {
    #[serde(rename = "type")]
    audio_type: AudioType,
    content: String,
}

fn get_all_audio_format(path: PathBuf) -> Vec<AudioResp> {
    let reader = AudioReader::new(path);
    AudioType::iter()
        .map(|audio_type| {
            let content = match audio_type {
                AudioType::Txt => reader.read_to_txt().unwrap_or_default(),
                AudioType::Srt => reader.read_to_srt().unwrap_or_default(),
                AudioType::Json => reader.read_to_json().unwrap_or_default(),
                AudioType::Vtt => reader.read_to_vtt().unwrap_or_default(),
                AudioType::Csv => reader.read_to_csv().unwrap_or_default(),
                AudioType::Ale => reader.read_to_ale().unwrap_or_default(),
                AudioType::Docx => reader.read_to_docx().unwrap_or_default(),
            };
            debug!("audio type: {audio_type:?}, content: {content}",);
            AudioResp {
                audio_type,
                content,
            }
        })
        .collect()
}

fn audio_export(artifacts_dir: PathBuf, input: ExportInput) -> anyhow::Result<Vec<AudioType>> {
    let save_dir = PathBuf::from(input.path);
    let types = input.type_group.clone();
    let reader = AudioReader::new(artifacts_dir.join(TRANSCRIPT_FILE_NAME));
    let downloader = DownloadHelper::new(reader, save_dir.clone());

    let mut error_list = vec![];

    types.iter().for_each(|audio_type| {
        let file_name = input
            .file_name
            .clone()
            .map(|file_name| format!("{file_name}.{audio_type}"))
            .unwrap_or(format!("transcript.{audio_type}"));
        let res = match audio_type {
            AudioType::Csv => downloader.download_to_csv(file_name.clone()),
            AudioType::Ale => downloader.download_to_ale(file_name.clone()),
            AudioType::Docx => downloader.download_to_docx(file_name.clone()),
            AudioType::Srt => downloader.download_to_srt(file_name.clone()),
            AudioType::Json => downloader.download_to_json(file_name.clone()),
            AudioType::Vtt => downloader.download_to_vtt(file_name.clone()),
            AudioType::Txt => downloader.download_to_txt(file_name.clone()),
        };
        if let Err(err) = res {
            error!("Failed to download {audio_type:?}: {err}",);
            error_list.push(audio_type.clone());
        }
    });
    if !error_list.is_empty() {
        warn!("Failed to download error list: {error_list:?}",);
        Ok(error_list)
    } else {
        Ok(vec![])
    }
}
