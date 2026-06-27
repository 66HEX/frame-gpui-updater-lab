use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use frame_core::types::DEFAULT_MAX_CONCURRENCY;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::settings::PresetDefinition;

const APP_SETTINGS_VERSION: u32 = 1;
const SETTINGS_FILE_NAME: &str = "settings.json";
const LEGACY_APP_SETTINGS_FILE_NAME: &str = "app-settings.dat";
const LEGACY_PRESETS_FILE_NAME: &str = "presets.dat";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppSettings {
    pub max_concurrency: usize,
    pub custom_presets: Vec<PresetDefinition>,
}

impl AppSettings {
    #[must_use]
    pub fn from_runtime(max_concurrency: usize, presets: &[PresetDefinition]) -> Self {
        Self {
            max_concurrency: valid_max_concurrency(max_concurrency),
            custom_presets: normalize_custom_presets(
                presets
                    .iter()
                    .filter(|preset| !preset.built_in)
                    .cloned()
                    .collect(),
            ),
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            max_concurrency: DEFAULT_MAX_CONCURRENCY,
            custom_presets: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppPersistence {
    settings_path: PathBuf,
}

impl AppPersistence {
    pub fn platform() -> Result<Self, AppPersistenceError> {
        let project_dirs = ProjectDirs::from("", "", "Frame")
            .ok_or(AppPersistenceError::ConfigDirectoryUnavailable)?;
        Ok(Self::from_settings_path(
            project_dirs.config_dir().join(SETTINGS_FILE_NAME),
        ))
    }

    #[must_use]
    pub fn from_settings_path(path: impl Into<PathBuf>) -> Self {
        Self {
            settings_path: path.into(),
        }
    }

    #[must_use]
    pub fn settings_path(&self) -> &Path {
        &self.settings_path
    }

    pub fn load(&self) -> Result<AppSettings, AppPersistenceError> {
        let bytes = match fs::read(&self.settings_path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return self.load_legacy();
            }
            Err(error) => return Err(AppPersistenceError::Io(error)),
        };

        let persisted: PersistedAppSettings = serde_json::from_slice(&bytes)?;
        Ok(persisted.into_app_settings())
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), AppPersistenceError> {
        let persisted = PersistedAppSettings::from_app_settings(settings);
        let json = serde_json::to_vec_pretty(&persisted)?;

        if let Some(parent) = self.settings_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let temp_path = temp_path_for(&self.settings_path);

        fs::write(&temp_path, json)?;
        replace_file(&temp_path, &self.settings_path)?;

        Ok(())
    }

    fn load_legacy(&self) -> Result<AppSettings, AppPersistenceError> {
        let mut settings = AppSettings::default();

        match fs::read(
            self.settings_path
                .with_file_name(LEGACY_APP_SETTINGS_FILE_NAME),
        ) {
            Ok(bytes) => {
                let legacy: LegacyAppSettings = serde_json::from_slice(&bytes)?;
                if let Some(max_concurrency) = legacy.max_concurrency {
                    settings.max_concurrency = valid_max_concurrency(max_concurrency);
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(AppPersistenceError::Io(error)),
        }

        match fs::read(self.settings_path.with_file_name(LEGACY_PRESETS_FILE_NAME)) {
            Ok(bytes) => {
                let legacy: LegacyPresetStore = serde_json::from_slice(&bytes)?;
                settings.custom_presets = normalize_custom_presets(legacy.presets);
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(AppPersistenceError::Io(error)),
        }

        Ok(settings)
    }
}

#[derive(Debug, Error)]
pub enum AppPersistenceError {
    #[error("config directory is unavailable")]
    ConfigDirectoryUnavailable,
    #[error("failed to read or write app settings: {0}")]
    Io(#[from] io::Error),
    #[error("failed to parse app settings: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
struct PersistedAppSettings {
    version: u32,
    max_concurrency: usize,
    custom_presets: Vec<PresetDefinition>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct LegacyAppSettings {
    max_concurrency: Option<usize>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct LegacyPresetStore {
    presets: Vec<PresetDefinition>,
}

impl PersistedAppSettings {
    fn from_app_settings(settings: &AppSettings) -> Self {
        Self {
            version: APP_SETTINGS_VERSION,
            max_concurrency: valid_max_concurrency(settings.max_concurrency),
            custom_presets: normalize_custom_presets(settings.custom_presets.clone()),
        }
    }

    fn into_app_settings(self) -> AppSettings {
        AppSettings {
            max_concurrency: valid_max_concurrency(self.max_concurrency),
            custom_presets: normalize_custom_presets(self.custom_presets),
        }
    }
}

impl Default for PersistedAppSettings {
    fn default() -> Self {
        Self {
            version: APP_SETTINGS_VERSION,
            max_concurrency: DEFAULT_MAX_CONCURRENCY,
            custom_presets: Vec::new(),
        }
    }
}

fn valid_max_concurrency(value: usize) -> usize {
    if value == 0 {
        DEFAULT_MAX_CONCURRENCY
    } else {
        value
    }
}

fn normalize_custom_presets(presets: Vec<PresetDefinition>) -> Vec<PresetDefinition> {
    let mut seen_ids = HashSet::new();

    presets
        .into_iter()
        .filter_map(|mut preset| {
            preset.id = preset.id.trim().to_string();
            preset.name = preset.name.trim().to_string();
            preset.built_in = false;

            if preset.id.is_empty() || preset.name.is_empty() || !seen_ids.insert(preset.id.clone())
            {
                return None;
            }

            Some(preset)
        })
        .collect()
}

fn replace_file(temp_path: &Path, final_path: &Path) -> Result<(), io::Error> {
    match fs::rename(temp_path, final_path) {
        Ok(()) => Ok(()),
        Err(_) if final_path.exists() => {
            fs::remove_file(final_path)?;
            fs::rename(temp_path, final_path)
        }
        Err(error) => Err(error),
    }
}

fn temp_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map_or_else(|| SETTINGS_FILE_NAME.to_string(), ToString::to_string);
    path.with_file_name(format!("{file_name}.tmp"))
}

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;
    use crate::settings::{ConversionConfig, PresetDefinition};

    static TEST_PATH_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn load_returns_defaults_when_settings_file_is_missing() {
        let persistence = AppPersistence::from_settings_path(test_settings_path());

        let settings = persistence
            .load()
            .expect("missing settings should load as defaults");

        assert_eq!(settings, AppSettings::default());
    }

    #[test]
    fn save_round_trips_max_concurrency_and_custom_presets() {
        let persistence = AppPersistence::from_settings_path(test_settings_path());
        let settings = AppSettings {
            max_concurrency: 4,
            custom_presets: vec![PresetDefinition::custom(
                "custom-preset-1".to_string(),
                "Review MP4".to_string(),
                ConversionConfig {
                    video_bitrate: "9000".to_string(),
                    ..ConversionConfig::default()
                },
            )],
        };

        persistence
            .save(&settings)
            .expect("settings should be saved");
        let loaded = persistence.load().expect("settings should be loaded");

        assert_eq!(loaded, settings);
    }

    #[test]
    fn load_replaces_zero_max_concurrency_with_default() {
        let path = test_settings_path();
        let parent = path.parent().expect("test path should have parent");
        fs::create_dir_all(parent).expect("test directory should be created");
        fs::write(
            &path,
            r#"{"version":1,"maxConcurrency":0,"customPresets":[]}"#,
        )
        .expect("settings fixture should be written");

        let settings = AppPersistence::from_settings_path(path)
            .load()
            .expect("settings should load");

        assert_eq!(settings.max_concurrency, DEFAULT_MAX_CONCURRENCY);
    }

    #[test]
    fn load_reads_camel_case_presets_and_fills_missing_config_defaults() {
        let path = test_settings_path();
        let parent = path.parent().expect("test path should have parent");
        fs::create_dir_all(parent).expect("test directory should be created");
        fs::write(
            &path,
            r#"{
                "version": 1,
                "maxConcurrency": 3,
                "customPresets": [{
                    "id": "custom-preset-2",
                    "name": "Legacy",
                    "builtIn": true,
                    "config": {
                        "container": "webm",
                        "metadata": { "mode": "clean" }
                    }
                }]
            }"#,
        )
        .expect("settings fixture should be written");

        let settings = AppPersistence::from_settings_path(path)
            .load()
            .expect("settings should load");

        assert_eq!(settings.max_concurrency, 3);
        assert_eq!(settings.custom_presets[0].config.container, "webm");
        assert_eq!(
            settings.custom_presets[0].config.metadata.mode,
            crate::settings::MetadataMode::Clean
        );
        assert!(!settings.custom_presets[0].built_in);
    }

    #[test]
    fn load_falls_back_to_legacy_tauri_store_files_when_new_settings_are_missing() {
        let path = test_settings_path();
        let parent = path.parent().expect("test path should have parent");
        fs::create_dir_all(parent).expect("test directory should be created");
        fs::write(
            path.with_file_name(LEGACY_APP_SETTINGS_FILE_NAME),
            r#"{"maxConcurrency":5,"autoUpdateCheck":true}"#,
        )
        .expect("legacy app settings fixture should be written");
        fs::write(
            path.with_file_name(LEGACY_PRESETS_FILE_NAME),
            r#"{"presets":[{
                "id":"custom-preset-8",
                "name":"Legacy Review",
                "builtIn":false,
                "config":{"container":"mkv"}
            }]}"#,
        )
        .expect("legacy presets fixture should be written");

        let settings = AppPersistence::from_settings_path(path)
            .load()
            .expect("legacy settings should load");

        assert_eq!(settings.max_concurrency, 5);
        assert_eq!(settings.custom_presets[0].id, "custom-preset-8");
        assert_eq!(settings.custom_presets[0].config.container, "mkv");
    }

    #[test]
    fn from_runtime_persists_only_custom_presets() {
        let settings = AppSettings::from_runtime(
            3,
            &[
                PresetDefinition::built_in(
                    "balanced-mp4",
                    "Balanced MP4",
                    ConversionConfig::default(),
                ),
                PresetDefinition::custom(
                    " custom-preset-1 ".to_string(),
                    " Review MP4 ".to_string(),
                    ConversionConfig::default(),
                ),
            ],
        );

        assert_eq!(settings.custom_presets.len(), 1);
        assert_eq!(settings.custom_presets[0].id, "custom-preset-1");
        assert_eq!(settings.custom_presets[0].name, "Review MP4");
        assert!(!settings.custom_presets[0].built_in);
    }

    fn test_settings_path() -> PathBuf {
        let sequence = TEST_PATH_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_millis();

        std::env::temp_dir()
            .join("frame-app-persistence-tests")
            .join(format!("{}-{millis}-{sequence}", std::process::id()))
            .join(SETTINGS_FILE_NAME)
    }
}
