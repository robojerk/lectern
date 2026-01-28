use crate::models::BookMetadata;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    pub author: String,
    pub by_asin: bool,
    pub is_searching: bool,
    pub results: Vec<BookMetadata>,
    pub error: Option<String>,
    pub current_page: usize,
    pub results_per_page: usize,
    pub result_covers: HashMap<String, iced::widget::image::Handle>, // Cache for search result cover images (URL -> image handle)
    pub downloading: Arc<Mutex<Vec<String>>>, // URLs currently being downloaded
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            query: String::new(),
            author: String::new(),
            by_asin: false,
            is_searching: false,
            results: Vec::new(),
            error: None,
            current_page: 0,
            results_per_page: 10,
            result_covers: HashMap::new(),
            downloading: Arc::new(Mutex::new(Vec::new())),
        }
    }
}
