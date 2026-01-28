use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookMetadata {
    pub title: String,
    pub subtitle: Option<String>,
    pub author: String,
    pub isbn: Option<String>,
    pub asin: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub duration: Option<String>,
    pub narrator: Option<String>,
    pub publisher: Option<String>,
    pub publish_year: Option<String>,
    pub series: Option<String>,
    pub series_number: Option<String>,
    pub genre: Option<String>,
    pub tags: Option<String>, // Comma-separated tags
    pub language: Option<String>,
    pub explicit: Option<bool>,
    pub abridged: Option<bool>,
}

impl Default for BookMetadata {
    fn default() -> Self {
        BookMetadata {
            title: String::new(),
            subtitle: None,
            author: String::new(),
            isbn: None,
            asin: None,
            description: None,
            cover_url: None,
            duration: None,
            narrator: None,
            publisher: None,
            publish_year: None,
            series: None,
            series_number: None,
            genre: None,
            tags: None,
            language: None,
            explicit: None,
            abridged: None,
        }
    }
}
