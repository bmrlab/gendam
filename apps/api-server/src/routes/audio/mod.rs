use crate::routes::audio::reader::AudioReader;
use crate::{Ctx, R};
use rspc::Router;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tracing::log::debug;

pub mod reader;

pub fn get_routes() -> Router<Ctx> {
    let router = R.router().procedure(
        "find_by_hash",
        R.query(|ctx, hash: String| async move {
            let artifacts_dir = ctx.library.artifacts_dir.clone();
            let path = artifacts_dir.join(hash).join("transcript.txt");
            serde_json::to_value::<Vec<AudioResp>>(get_all_audio_format(path)).unwrap_or_default()
        }),
    );
    router
}

#[derive(EnumIter, Debug, Deserialize, Serialize, Clone)]
enum AudioType {
    Txt,
    Srt,
    Json,
    Vtt,
    Csv,
    Ale,
    Docx,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AudioResp {
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
