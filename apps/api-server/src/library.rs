use content_library::Library;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;
use storage::S3Config;
use strum_macros::{Display, EnumString};

use crate::ctx::traits::CtxError;
use crate::CtxWithLibrary;

// libraries/[uuid as library id]/settings.json
pub const LIBRARY_SETTINGS_FILE_NAME: &str = "settings.json";

#[derive(Serialize, EnumString, Display, Type, Debug, Clone)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum LibrarySettingsThemeEnum {
    Light,
    Dark,
}

#[derive(EnumString, Display, Serialize, Deserialize, Type, Clone, Debug)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum LibrarySettingsLayoutEnum {
    List,
    Grid,
    Media,
}

#[derive(Type, Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LibrarySettingsExplorer {
    pub layout: LibrarySettingsLayoutEnum,
    pub inspector_size: u32,
    pub inspector_show: bool,
}

impl Default for LibrarySettingsExplorer {
    fn default() -> Self {
        LibrarySettingsExplorer {
            layout: LibrarySettingsLayoutEnum::Grid,
            inspector_size: 240,
            inspector_show: false,
        }
    }
}

#[derive(Type, Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct LibraryModels {
    pub multi_modal_embedding: String,
    pub text_embedding: String,
    pub image_caption: String,
    pub audio_transcript: String,
    pub llm: String,
}

impl Default for LibraryModels {
    fn default() -> Self {
        LibraryModels {
            multi_modal_embedding: "clip-multilingual-v1".to_string(),
            text_embedding: "puff-base-v1".to_string(),
            image_caption: "llava-phi3-mini".to_string(),
            audio_transcript: "whisper-small".to_string(),
            llm: "qwen2-7b-instruct".to_string(),
        }
    }
}

#[derive(Serialize, Type, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LibrarySettings {
    pub title: String,
    pub appearance_theme: LibrarySettingsThemeEnum,
    pub explorer: LibrarySettingsExplorer,
    pub models: LibraryModels,
    pub always_delete_local_file_after_upload: bool,
    pub s3_config: Option<S3Config>,
}

impl<'de> Deserialize<'de> for LibrarySettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = match serde_json::Value::deserialize(deserializer) {
            Ok(value) => value,
            Err(_) => return Ok(LibrarySettings::default()),
        };

        // let models = value["models"].clone();
        // let models = ;

        let settings = LibrarySettings {
            title: value["title"].as_str().unwrap_or("Untitled").to_string(),
            appearance_theme: value["appearanceTheme"]
                .as_str()
                .unwrap_or_default()
                .parse()
                .unwrap_or(LibrarySettingsThemeEnum::Light),
            explorer: serde_json::from_value::<LibrarySettingsExplorer>(
                value["explorer"].to_owned(),
            )
            .unwrap_or_default(),
            models: serde_json::from_value::<LibraryModels>(value["models"].to_owned())
                .unwrap_or_default(),
            always_delete_local_file_after_upload: value["alwaysDeleteLocalFileAfterUpload"]
                .as_bool()
                .unwrap_or(false),
            s3_config: serde_json::from_value::<Option<S3Config>>(value["s3Config"].to_owned())
                .unwrap_or(None),
        };
        Ok(settings)
    }
}

impl Default for LibrarySettings {
    fn default() -> Self {
        LibrarySettings {
            title: "Untitled".to_string(),
            appearance_theme: LibrarySettingsThemeEnum::Light,
            explorer: Default::default(),
            models: Default::default(),
            always_delete_local_file_after_upload: false,
            s3_config: None,
        }
    }
}

pub fn get_library_settings(library_dir: &PathBuf) -> LibrarySettings {
    let p = library_dir.join(LIBRARY_SETTINGS_FILE_NAME);
    if !p.exists() {
        return LibrarySettings::default();
    }
    let settings = match std::fs::File::open(p) {
        Ok(file) => {
            let reader = std::io::BufReader::new(file);
            match serde_json::from_reader(reader) {
                Ok(values) => match serde_json::from_value::<LibrarySettings>(values) {
                    Ok(settings) => settings,
                    Err(e) => {
                        tracing::error!("Failed to parse library's settings.json: {}", e);
                        LibrarySettings::default()
                    }
                },
                Err(e) => {
                    tracing::error!("Failed to read library's settings.json: {}", e);
                    LibrarySettings::default()
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to open library's settings.json, {}", e);
            LibrarySettings::default()
        }
    };

    settings
}

pub fn set_library_settings(library_dir: &PathBuf, settings: LibrarySettings) {
    match std::fs::File::create(library_dir.join(LIBRARY_SETTINGS_FILE_NAME)) {
        Ok(file) => {
            if let Err(e) = serde_json::to_writer(file, &settings) {
                tracing::error!("Failed to write file: {}", e);
            }
        }
        Err(e) => {
            tracing::error!("Failed to create file: {}", e);
        }
    };
}

/// Load library and wait for it to be loaded
/// If another request comes in while a previous load is still in progress, it will fail with an error.
pub async fn load_library_exclusive_and_wait<TCtx>(
    ctx: TCtx,
    library_id: String,
) -> Result<Library, CtxError>
where
    // 传入 `spawn` 的闭包必须是 `'static` 的
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    let (tx, rx) = tokio::sync::oneshot::channel::<Result<Library, CtxError>>();
    tokio::spawn(async move {
        match ctx.load_library(&library_id).await {
            Ok(library) => {
                tracing::info!(library_id = library_id, "Library loaded: {:?}", library);
                // 不要 unwrap, 请求被 cancel 以后 rx 会被 drop, 这里 send 会返回错误
                let _ = tx.send(Ok(library));
            }
            Err(e) => {
                tracing::error!(library_id = library_id, "Failed to load library: {:?}", e);
                let _ = tx.send(Err(e));
                // 不要 unload, 前端遇到 load 失败以后自己调用 unload, 方便控制状态
                // ctx.unload_library().await
            }
        };
    });
    // 放在 thread 里执行，这样在请求被 cancel 的时候还会继续执行，前端通过轮询 status 接口获取结果
    match rx.await {
        Ok(res) => res,
        Err(e) => Err(CtxError::Internal(format!(
            "Failed to receive load library result: {}",
            e
        ))),
    }
}

/// Load library and wait for it to be loaded
/// If another request comes in while a previous load is still in progress, it will fail with an error.
pub async fn unload_library_exclusive_and_wait<TCtx>(ctx: TCtx) -> Result<(), CtxError>
where
    // 传入 `spawn` 的闭包必须是 `'static` 的
    TCtx: CtxWithLibrary + Clone + Send + Sync + 'static,
{
    let (tx, rx) = tokio::sync::oneshot::channel::<Result<(), CtxError>>();
    tokio::spawn(async move {
        match ctx.unload_library().await {
            Ok(_) => {
                tracing::info!("Library unloaded");
                let _ = tx.send(Ok(()));
            }
            Err(e) => {
                tracing::error!("Failed to unload library: {:?}", e);
                let _ = tx.send(Err(e));
            }
        };
    });
    // 放在 thread 里执行，这样在请求被 cancel 的时候还会继续执行，前端通过轮询 status 接口获取结果
    match rx.await {
        Ok(result) => result,
        Err(e) => Err(CtxError::Internal(format!(
            "Failed to receive unload library result: {}",
            e
        ))),
    }
}
