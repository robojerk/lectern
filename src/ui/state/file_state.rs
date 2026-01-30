#[derive(Debug, Clone)]
pub struct FileState {
    pub selected_file_path: Option<String>,
    pub audio_file_paths: Vec<String>, // List of audio files when directory is selected
    pub is_parsing_file: bool,
    pub file_parse_error: Option<String>,
    /// Metadata or chapter files found in the audiobook folder (name, full path)
    pub found_metadata_chapter_files: Vec<(String, String)>,
}

impl Default for FileState {
    fn default() -> Self {
        Self {
            selected_file_path: None,
            audio_file_paths: Vec::new(),
            is_parsing_file: false,
            file_parse_error: None,
            found_metadata_chapter_files: Vec::new(),
        }
    }
}
