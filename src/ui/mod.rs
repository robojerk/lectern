pub mod colors;
pub mod views;
pub mod helpers;
pub mod cover_search;
pub mod state;
pub mod handlers;

use handlers::*;



pub use views::ViewMode;
use cover_search::CoverResult;


use crate::models::{Chapter, BookMetadata};
use std::time::Instant;
use std::sync::Arc;
use tokio::sync::Mutex;
use state::{SearchState, MetadataState, CoverState, ChapterState, ChapterRegion, FileState, MetadataProvider};

// Store process handle separately since Child is not Clone
#[derive(Debug)]
pub struct ChapterPlaybackProcess {
    pub process: tokio::process::Child,
}

impl Drop for ChapterPlaybackProcess {
    fn drop(&mut self) {
        // Ensure process is killed when dropped
        // Use start_kill() for immediate termination (non-blocking)
        let _ = self.process.start_kill();
    }
}

#[derive(Debug, Clone)]
pub struct ChapterPlaybackState {
    pub chapter_index: usize,
    pub start_time: Instant,
    pub elapsed_ms: u64,
    pub is_playing: bool,
    pub process_id: Option<u32>, // Store process ID for reference
    pub was_manually_stopped: bool, // Track if user manually stopped playback
}

use iced::widget::{column, container, text_editor};
use iced::{Application, Command, Element, Length, Theme, Subscription};
use iced::{window, event};


#[derive(Debug, Clone)]
pub enum Message {
    SearchQueryChanged(String),
    SearchAuthorChanged(String),
    SearchByAsinToggled(bool),
    PerformSearch,
    NextPage,
    PreviousPage,
    SearchCompleted(Result<Vec<BookMetadata>, String>),
    SelectBook(usize), // Index into results
    SwitchToSearch,
    SwitchToMetadata,
    SwitchToCover,
    SwitchToChapters,
    SwitchToConvert,
    TitleChanged(String),
    SubtitleChanged(String),
    AuthorChanged(String),
    SeriesChanged(String),
    SeriesNumberChanged(String),
    NarratorChanged(String),
    DescriptionAction(text_editor::Action),
    IsbnChanged(String),
    AsinChanged(String),
    PublisherChanged(String),
    PublishYearChanged(String),
    GenreChanged(String),
    TagsChanged(String),
    LanguageChanged(String),
    ExplicitToggled(bool),
    AbridgedToggled(bool),
    // File selection
    BrowseFiles,
    BrowseFolder,
    FileSelected(Option<String>), // Path to selected file/directory
    FileDropped(Vec<String>), // Paths from drag & drop
    FileParsed(Result<BookMetadata, String>), // Result of parsing the file
    // Cover management
    BrowseCoverImage,
    CoverImageSelected(Option<String>), // Path to selected cover image
    SearchCover,
    CoverSearchCompleted(Result<Vec<CoverResult>, String>),
    SelectCover(usize), // Index into cover search results
    CoverUrlChanged(String), // Manual URL entry
    DownloadCoverImage(String), // URL to download
    CoverImageDownloaded(Result<(String, Vec<u8>, iced::widget::image::Handle), String>), // URL, raw data, handle
    SearchCoverImageDownloaded(Result<(String, Vec<u8>, iced::widget::image::Handle), String>), // URL, raw data, handle
    SearchCoverImagesDownloaded(Vec<(String, Result<iced::widget::image::Handle, String>)>), // Batch download results (background thread)
    // Chapter management
    ChapterTitleChanged(usize, String), // Index, new title
    ChapterTimeChanged(usize, String), // Index, new time string (HH:MM:SS)
    ChapterTimeAdjusted(usize, i64), // Index, adjustment in seconds
    ChapterLockToggled(usize), // Index
    ChapterDelete(usize), // Index
    ChapterInsertBelow(usize), // Index - insert new chapter after this one
    ChapterRemoveAll,
    ChapterShiftTimes(i64), // Shift all unlocked chapters by seconds
    ChapterLookup, // Lookup chapters from provider
    ChapterLookupCompleted(Result<Vec<Chapter>, String>),
    ChapterExtractFromFile, // Extract chapters from file using ffprobe
    ChapterExtractCompleted(Result<Vec<Chapter>, String>),
    ChapterShiftAll(i64), // Shift all chapters by offset (milliseconds, can be negative)
    ChapterValidate, // Validate chapters (check for gaps, overlaps, etc.)
    ChapterShiftWithRipple(usize, u64), // Shift individual chapter with ripple effect (index, new_start_ms)
    ChapterPlay(usize), // Play chapter at index (preview from start_time)
    ChapterPlaybackStarted(usize, Option<u32>, Arc<Mutex<ChapterPlaybackProcess>>), // Playback started (index, process_id, process_handle)
    ChapterPlaybackError(String), // Error starting playback
    ChapterPlaybackTick, // Timer tick for playback progress
    ChapterStopPlayback, // Stop current playback
    ChapterPlaybackProcessExited, // Process finished naturally
    ChaptersShowSecondsToggled(bool),
    ChaptersGlobalLockToggled,
    ChapterAsinChanged(String), // Manual ASIN entry for chapter lookup
    ChapterToggleAsinInput, // Toggle ASIN input area
    ChapterRegionChanged(ChapterRegion),
    ChapterRemoveAudibleToggled(bool),
    MapChaptersFromFiles, // Map chapters from audio files (one file = one chapter)
    ChapterSetTimeFromPlayback(usize), // Set chapter start time to current playback elapsed time
    // Settings
    SwitchToSettings,
    LocalLibraryPathChanged(String),
    BrowseLocalLibraryPath,
    LocalLibraryPathSelected(Option<String>),
    MediaManagementTemplateChanged(String),
    AudiobookshelfHostChanged(String),
    AudiobookshelfTokenChanged(String),
    AudiobookshelfLibraryIdChanged(String),
    // Provider selection
    MetadataProviderChanged(MetadataProvider), // Provider name for metadata search
    // Convert messages
    StartConversion,
    BrowseOutputPath,
    OutputPathSelected(Option<String>),
    ConversionCompleted(Result<(String, u64, u64), String>),
    ConversionNormalizeVolumeToggled(bool),
    ConversionBitrateChanged(String),
    ConversionCodecChanged(String),
    ConversionChannelsChanged(String),
}

pub struct Lectern {
    // State modules organized by feature
    pub search: SearchState,
    pub metadata: MetadataState,
    pub cover: CoverState,
    pub chapters: ChapterState,
    // Chapter playback state (not in ChapterState because types aren't Clone)
    pub chapter_playback_state: Option<ChapterPlaybackState>,
    pub chapter_playback_process: Option<Arc<Mutex<ChapterPlaybackProcess>>>,
    pub file: FileState,
    
    // Current view mode
    pub view_mode: ViewMode,
    
    // Settings state
    pub local_library_path: Option<String>,
    pub media_management_template: String, // e.g., "{Author}/{Series}/{Title}.m4b"
    pub audiobookshelf_host: String,
    pub audiobookshelf_token: String,
    pub audiobookshelf_library_id: String,
    
    // Convert state
    pub output_path: Option<String>,
    pub is_converting: bool,
    pub conversion_error: Option<String>,
    pub source_size: u64,
    pub output_size: u64,
    pub conversion_normalize_volume: bool,
    pub conversion_bitrate: String, // "auto", "64k", "96k", "128k", "192k"
    pub conversion_codec: String, // "aac", "copy", "opus"
    pub conversion_channels: String, // "auto", "1", "2"
}

impl Default for Lectern {
    fn default() -> Self {
        Self {
            // State modules
            search: SearchState::default(),
            metadata: MetadataState::default(),
            cover: CoverState::default(),
            chapters: ChapterState::default(),
            // Chapter playback state
            chapter_playback_state: None,
            chapter_playback_process: None,
            file: FileState::default(),
            
            // Current view mode
            view_mode: ViewMode::Metadata,
            
            // Settings state
            local_library_path: None,
            media_management_template: "{Author}/{Title}.m4b".to_string(),
            audiobookshelf_host: String::new(),
            audiobookshelf_token: String::new(),
            audiobookshelf_library_id: String::new(),
            
            // Convert state
            output_path: None,
            is_converting: false,
            conversion_error: None,
            source_size: 0,
            output_size: 0,
            conversion_normalize_volume: false,
            conversion_bitrate: "auto".to_string(),
            conversion_codec: "aac".to_string(),
            conversion_channels: "auto".to_string(),
            
            // Note: metadata_provider is now in metadata.metadata_provider
        }
    }
}

impl Drop for Lectern {
    fn drop(&mut self) {
        // Clean up any running processes when the app is dropped
        if let Some(process_handle) = self.chapter_playback_process.take() {
            // Try to kill the process synchronously to ensure cleanup
            let process_clone = process_handle.clone();
            // Use a blocking approach to ensure the process is killed
            if let Ok(rt) = tokio::runtime::Runtime::new() {
                let _ = rt.block_on(async {
                    let mut proc = process_clone.lock().await;
                    let _ = proc.process.kill().await;
                });
            }
            // Also try start_kill as a fallback (non-blocking)
            // Note: try_lock() doesn't exist on Arc<Mutex<>>, so we just drop
            drop(process_handle);
        }
        // Also try to kill via process ID as a fallback
        if let Some(ref state) = self.chapter_playback_state {
            if let Some(pid) = state.process_id {
                // Try TERM first, then KILL if needed
                let _ = std::process::Command::new("kill")
                    .arg("-TERM")
                    .arg(&pid.to_string())
                    .output();
                // Give it a moment
                std::thread::sleep(std::time::Duration::from_millis(100));
                // Force kill if still running
                let _ = std::process::Command::new("kill")
                    .arg("-KILL")
                    .arg(&pid.to_string())
                    .output();
            }
        }
    }
}

impl Application for Lectern {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        "Lectern - Audiobook Tool".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        // Try each handler in order - first one that returns Some() wins
        if let Some(cmd) = handle_search(self, message.clone()) {
            return cmd;
        }
        if let Some(cmd) = handle_metadata(self, message.clone()) {
            return cmd;
        }
        if let Some(cmd) = handle_cover(self, message.clone()) {
            return cmd;
        }
        if let Some(cmd) = handle_chapters(self, message.clone()) {
            return cmd;
        }
        if let Some(cmd) = handle_file(self, message.clone()) {
            return cmd;
        }
        if let Some(cmd) = handle_settings(self, message.clone()) {
            return cmd;
        }
        if let Some(cmd) = handle_convert(self, message.clone()) {
            return cmd;
        }
        if let Some(cmd) = handle_navigation(self, message.clone()) {
            return cmd;
        }
        
        // Message not handled (shouldn't happen if all messages are covered)
        eprintln!("[WARNING] Unhandled message: {:?}", message);
        Command::none()
    }

    fn view(&self) -> Element<'_, Message> {
        use views::LecternView;
        
        let header = self.view_header();
        
        // Call view functions from their modules
        let content = match self.view_mode {
            ViewMode::Search => views::search::view_search(self),
            ViewMode::Metadata => views::metadata::view_metadata(self),
            ViewMode::Cover => views::cover::view_cover(self),
            ViewMode::Chapters => views::chapters::view_chapters(self),
            ViewMode::Convert => views::convert::view_convert(self),
            ViewMode::Settings => views::settings::view_settings(self),
        };
        
        container(
            column![
                header,
                content,
            ]
            .spacing(10)
            .padding(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        // Subscribe to file drop events
        // NOTE: File drops work on X11 but are NOT implemented on Wayland in Iced 0.12/winit
        // This is a known limitation: https://github.com/rust-windowing/winit/issues/1881
        // Winit has an open PR (#2429) to add Wayland support, but it's not merged yet.
        // Once winit adds Wayland drag-and-drop support, Iced will inherit it automatically.
        event::listen_with(|event, _status| {
            match event {
                event::Event::Window(_window_id, window::Event::FileDropped(paths)) => {
                    println!("[DEBUG] FileDropped event received: {:?}", paths);
                    let converted_paths: Vec<String> = paths.into_iter()
                        .map(|p| {
                            let path_str = p.to_string_lossy().to_string();
                            println!("[DEBUG] Converting path: {:?} -> '{}'", p, path_str);
                            path_str
                        })
                        .collect();
                    println!("[DEBUG] Converted {} paths: {:?}", converted_paths.len(), converted_paths);
                    Some(Message::FileDropped(converted_paths))
                }
                event::Event::Window(_window_id, window::Event::FileHovered(path)) => {
                    println!("[DEBUG] FileHovered event received: {:?}", path);
                    None // Just log for debugging
                }
                event::Event::Window(_window_id, window::Event::FilesHoveredLeft) => {
                    println!("[DEBUG] FilesHoveredLeft event received");
                    None // Just log for debugging
                }
                _ => None
            }
        })
    }
}
