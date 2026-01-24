use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub audiobookshelf: Option<AudiobookshelfSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudiobookshelfSettings {
    pub base_url: String,
    pub api_key: String,
    pub library_id: String,
}
