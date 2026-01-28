use crate::models::Chapter;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChapterRegion {
    US,
    CA,
    UK,
    AU,
    FR,
    DE,
}

impl ChapterRegion {
    pub const ALL: [ChapterRegion; 6] = [
        ChapterRegion::US,
        ChapterRegion::CA,
        ChapterRegion::UK,
        ChapterRegion::AU,
        ChapterRegion::FR,
        ChapterRegion::DE,
    ];
}

impl fmt::Display for ChapterRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChapterRegion::US => write!(f, "US"),
            ChapterRegion::CA => write!(f, "CA"),
            ChapterRegion::UK => write!(f, "UK"),
            ChapterRegion::AU => write!(f, "AU"),
            ChapterRegion::FR => write!(f, "FR"),
            ChapterRegion::DE => write!(f, "DE"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChapterState {
    pub chapters: Vec<Chapter>,
    pub show_seconds: bool,
    pub global_locked: bool,
    pub is_looking_up_chapters: bool,
    pub lookup_error: Option<String>,
    // Note: playback_state and playback_process are not Clone, so they're stored
    // directly in Lectern struct, not here
    pub asin_input: String, // Manual ASIN input for chapter lookup
    pub show_asin_input: bool, // Toggle for ASIN input area
    pub selected_region: ChapterRegion,
    pub remove_audible_intro_outro: bool,
    pub chapter_time_editing: std::collections::HashMap<usize, String>, // Store raw input while editing
}

impl Default for ChapterState {
    fn default() -> Self {
        Self {
            chapters: Vec::new(),
            show_seconds: false,
            global_locked: false,
            is_looking_up_chapters: false,
            lookup_error: None,
            asin_input: String::new(),
            show_asin_input: false,
            selected_region: ChapterRegion::US,
            remove_audible_intro_outro: false,
            chapter_time_editing: std::collections::HashMap::new(),
        }
    }
}
