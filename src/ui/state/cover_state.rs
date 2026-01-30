use crate::ui::cover_search::CoverResult;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CoverState {
    pub cover_image_path: Option<String>, // Local file path or URL
    pub cover_image_data: Option<Vec<u8>>, // Downloaded image data for URLs
    pub cover_image_handle: Option<iced::widget::image::Handle>, // Cached handle for Iced
    pub cover_image_url_cached: Option<String>, // URL that corresponds to cached image data
    pub is_searching_cover: bool,
    pub cover_search_results: Vec<CoverResult>,
    /// Cache for cover search result thumbnails (URL -> image handle)
    pub cover_search_result_handles: HashMap<String, iced::widget::image::Handle>,
    pub cover_search_error: Option<String>,
    pub is_downloading_cover: bool,
}

impl Default for CoverState {
    fn default() -> Self {
        Self {
            cover_image_path: None,
            cover_image_data: None,
            cover_image_handle: None,
            cover_image_url_cached: None,
            is_searching_cover: false,
            cover_search_results: Vec::new(),
            cover_search_result_handles: HashMap::new(),
            cover_search_error: None,
            is_downloading_cover: false,
        }
    }
}
