use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BookMetadata {
    pub title: String,
    pub authors: Vec<String>,
    pub narrator_names: Option<Vec<String>>,
    pub series_name: Option<String>,
    pub image_url: String,
    pub asin: String,
    pub duration_minutes: Option<u64>,
    pub release_date: Option<String>,
}
