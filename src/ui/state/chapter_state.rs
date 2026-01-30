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
    pub shift_all_input: String, // User-entered shift amount in seconds (e.g. "0", "-5", "2.5") for "Shift all"
    pub book_duration_ms: Option<u64>, // Total book duration (ms) from ffprobe; used to validate chapter starts
    pub is_mapping_from_files: bool, // True while "Map from files" is running
    /// Phase for loading spinner (0..4); only used when is_mapping_from_files or is_looking_up_chapters.
    pub loading_spinner_phase: u8,
    /// Rotation in degrees for canvas spinner (0..360); advances each tick when loading.
    pub loading_spinner_rotation: f32,
    pub shift_held: bool, // True when Shift key is held (for lock range)
    pub last_lock_clicked_index: Option<usize>, // Anchor for Shift+click lock range
    /// Virtual list: (scroll_offset_y, viewport_height, content_height) from scrollable on_scroll.
    pub chapter_list_viewport: Option<(f32, f32, f32)>,
    /// Incremented on CloseBook so in-flight chapter loads are ignored when they complete.
    pub load_generation: u64,
    /// Pending lookup result (e.g. from Audible); user can Apply (replace) or Map titles only.
    pub lookup_result: Option<Vec<crate::models::Chapter>>,
    /// Total duration (ms) from last lookup; used to warn if it differs from book_duration_ms.
    pub lookup_duration_ms: Option<u64>,
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
            shift_all_input: String::new(),
            book_duration_ms: None,
            is_mapping_from_files: false,
            loading_spinner_phase: 0,
            loading_spinner_rotation: 0.0,
            shift_held: false,
            last_lock_clicked_index: None,
            chapter_list_viewport: None,
            load_generation: 0,
            lookup_result: None,
            lookup_duration_ms: None,
        }
    }
}
