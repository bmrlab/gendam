use serde::{Deserialize, Serialize};
use strum_macros::{EnumString, Display};
use specta::Type;
use std::path::PathBuf;

// libraries/[uuid as library id]/settings.json
pub const LIBRARY_SETTINGS_FILE_NAME: &str = "settings.json";

#[derive(Serialize, EnumString, Display, Type, Debug)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum LibrarySettingsThemeEnum {
    Light,
    Dark,
}

#[derive(Serialize, EnumString, Display, Type, Debug)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum LibrarySettingsLayoutEnum {
    List,
    Grid,
    Media,
}

#[derive(Serialize, Type, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LibrarySettings {
    pub title: String,
    pub appearance_theme: LibrarySettingsThemeEnum,
    pub explorer_layout: LibrarySettingsLayoutEnum,
}

impl<'de> Deserialize<'de> for LibrarySettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = match serde_json::Value::deserialize(deserializer) {
            Ok(value) => value,
            Err(_) => return Ok(LibrarySettings::default())
        };
        let settings = LibrarySettings {
            title: value["title"].as_str().unwrap_or("Untitled").to_string(),
            appearance_theme: value["appearanceTheme"].as_str().unwrap_or_default().parse().unwrap_or(LibrarySettingsThemeEnum::Light),
            explorer_layout: value["explorerLayout"].as_str().unwrap_or_default().parse().unwrap_or(LibrarySettingsLayoutEnum::Grid),
        };
        Ok(settings)
    }
}

impl Default for LibrarySettings {
    fn default() -> Self {
        LibrarySettings {
            title: "Untitled".to_string(),
            appearance_theme: LibrarySettingsThemeEnum::Light,
            explorer_layout: LibrarySettingsLayoutEnum::List,
        }
    }
}

pub fn get_library_settings(library_dir: &PathBuf) -> LibrarySettings {
    match std::fs::File::open(library_dir.join(LIBRARY_SETTINGS_FILE_NAME)) {
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
    }
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
