use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub abs_host: String,
    #[serde(default)]
    pub abs_token: String,
    #[serde(default)]
    pub abs_library_id: String,
    #[serde(default)]
    pub local_library: Option<String>,
    #[serde(default = "default_template")]
    pub path_template: String,
}

fn default_template() -> String {
    "{Author}/{Title}.m4b".to_string()
}

#[derive(Clone, Default)]
pub struct AppState {
    pub folder_path: Option<PathBuf>,
    pub config: Option<AppConfig>,
    pub project: Option<crate::models::project::Project>,
    pub search_results: Vec<crate::models::metadata::BookMetadata>,
    pub original_cover_bytes: Option<Vec<u8>>,
}

pub fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("lectern");
    fs::create_dir_all(&path).unwrap_or_default();
    path.push("config.json");
    path
}

pub fn load_config() -> Option<AppConfig> {
    let path = get_config_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(path) {
            return serde_json::from_str(&content).ok();
        }
    }
    None
}

pub fn save_config(config: &AppConfig) {
    let path = get_config_path();
    if let Ok(content) = serde_json::to_string_pretty(config) {
        let _ = fs::write(path, content);
    }
}