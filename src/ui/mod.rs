pub mod colors;
pub mod icons;
pub mod theme;
pub mod theme_settings;
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
use iced::time;
use iced::{window, event, keyboard};
use iced::keyboard::key;


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
    /// Keyboard tab: focus next/previous on metadata tab (from subscription).
    MetadataFocusNext,
    MetadataFocusPrevious,
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
    /// Close the current book and return to the initial screen (drag-and-drop / file chooser).
    CloseBook,
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
    CoverSearchResultsImagesDownloaded(Vec<(String, Result<iced::widget::image::Handle, String>)>), // Cover tab search result thumbnails
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
    ChapterLookupCompleted(u64, Result<Vec<Chapter>, String>),
    ChapterLookupApply, // Apply looked-up chapters (replace current)
    MapChapterTitlesOnly, // Apply looked-up titles to existing chapters by index, keep timestamps
    ChapterExtractFromFile, // Extract chapters from file using ffprobe
    ChapterExtractCompleted(u64, Result<Vec<Chapter>, String>),
    ChapterShiftAll(i64), // Shift all chapters by offset (milliseconds, can be negative)
    ChapterShiftAmountChanged(String), // User typing in "Shift all" field (seconds, e.g. "-5" or "2.5")
    ChapterShiftAllApply, // Apply shift from shift_all_input
    ChapterValidate, // Validate chapters (check for gaps, overlaps, etc.)
    ChapterShiftWithRipple(usize, u64), // Shift individual chapter with ripple effect (index, new_start_ms)
    ChapterPlay(usize), // Play chapter at index (preview from start_time)
    ChapterPlaybackStarted(usize, Option<u32>, Arc<Mutex<ChapterPlaybackProcess>>), // Playback started (index, process_id, process_handle)
    ChapterPlaybackError(String), // Error starting playback
    ChapterPlaybackTick, // Timer tick for playback progress
    ChapterLoadingTick, // Timer tick for loading spinner (mapping / lookup)
    ChapterStopPlayback, // Stop current playback
    ChapterPlaybackProcessExited, // Process finished naturally
    ChaptersShowSecondsToggled(bool),
    ChaptersGlobalLockToggled,
    ChapterAsinChanged(String), // Manual ASIN entry for chapter lookup
    ChapterToggleAsinInput, // Toggle ASIN input area
    ShiftModifierChanged(bool), // Shift key pressed (true) or released (false)
    ChapterRegionChanged(ChapterRegion),
    ChapterRemoveAudibleToggled(bool),
    MapChaptersFromFiles, // Map chapters from audio files (one file = one chapter)
    MapChaptersFromFilesCompleted(u64, Result<Vec<Chapter>, String>),
    BookDurationComputed(u64, Result<u64, String>), // (load_generation, duration ms from ffprobe)
    ChapterSetTimeFromPlayback(usize), // Set chapter start time to current playback elapsed time
    /// Virtual list: viewport changed (offset_y, viewport_height, content_height) for visible range.
    ChapterListViewportChanged { offset_y: f32, viewport_height: f32, content_height: f32 },
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
    // Theme / appearance
    ThemeIdChanged(crate::ui::theme::ThemeId),
    DarkModeToggled(bool),
    AccentColorChanged(Option<iced::Color>),
    AccentHexInputChanged(String),
    UseThemeDefaultAccentToggled(bool),
    ChapterIconsLoaded((std::collections::HashMap<String, iced::widget::image::Handle>, std::collections::HashMap<String, iced::widget::image::Handle>)),
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

    // Theme state (cache lives here so we don't re-allocate on every view())
    pub theme_id: crate::ui::theme::ThemeId,
    pub dark_mode: bool,
    pub accent_override: Option<iced::Color>,
    /// Current hex input for custom accent (e.g. "#3daee9"); used when accent_override is Some.
    pub accent_hex_input: String,
    /// Cached extended palette for views; updated when theme_id/dark_mode/accent_override change.
    pub cached_palette: Option<iced::theme::palette::Extended>,
    /// Chapter icons: light and dark sets, populated once at startup.
    pub chapter_icons_light: std::collections::HashMap<String, iced::widget::image::Handle>,
    pub chapter_icons_dark: std::collections::HashMap<String, iced::widget::image::Handle>,
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

            // Theme state
            theme_id: crate::ui::theme::ThemeId::default(),
            dark_mode: true,
            accent_override: None,
            accent_hex_input: String::new(),
            cached_palette: {
                let (_, ext) = crate::ui::theme::build_theme(
                    crate::ui::theme::ThemeId::default(),
                    true,
                    None,
                );
                Some(ext)
            },
            chapter_icons_light: std::collections::HashMap::new(),
            chapter_icons_dark: std::collections::HashMap::new(),
        }
    }
}

impl Lectern {
    /// Returns the current extended palette for use in views. Panics if cache is missing (should not happen).
    pub fn palette(&self) -> &iced::theme::palette::Extended {
        self.cached_palette.as_ref().expect("cached_palette set in Default and on theme change")
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
        let mut app = Self::default();
        if let Some((theme_id, dark_mode, accent_override)) = crate::ui::theme_settings::load() {
            app.theme_id = theme_id;
            app.dark_mode = dark_mode;
            app.accent_override = accent_override;
            app.accent_hex_input = accent_override
                .map(crate::ui::theme_settings::color_to_hex_export)
                .unwrap_or_default();
            app.cached_palette = Some(
                crate::ui::theme::build_theme(app.theme_id, app.dark_mode, app.accent_override).1,
            );
        }
        let cmd = Command::perform(
            tokio::task::spawn_blocking(icons::load_chapter_icons_both),
            |r| {
                Message::ChapterIconsLoaded(r.unwrap_or_else(|_| (std::collections::HashMap::new(), std::collections::HashMap::new())))
            },
        );
        (app, cmd)
    }

    fn title(&self) -> String {
        "Lectern - Audiobook Tool".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        // One-time icon load (stored on Lectern so view() never allocates)
        if let Message::ChapterIconsLoaded((light, dark)) = message.clone() {
            self.chapter_icons_light = light;
            self.chapter_icons_dark = dark;
            return Command::none();
        }
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
        if matches!(message, Message::MetadataFocusNext | Message::MetadataFocusPrevious) {
            if self.view_mode == ViewMode::Metadata {
                return if matches!(message, Message::MetadataFocusNext) {
                    iced::widget::focus_next()
                } else {
                    iced::widget::focus_previous()
                };
            }
            return Command::none();
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
        crate::ui::theme::build_theme(self.theme_id, self.dark_mode, self.accent_override).0
    }

    fn subscription(&self) -> Subscription<Message> {
        // Subscribe to file drop events and Tab for metadata focus navigation
        // NOTE: File drops work on X11 but are NOT implemented on Wayland in Iced 0.12/winit
        // This is a known limitation: https://github.com/rust-windowing/winit/issues/1881
        let event_sub = event::listen_with(|event, _status| {
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
                event::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                    if key == keyboard::Key::Named(key::Named::Shift) {
                        Some(Message::ShiftModifierChanged(true))
                    } else if key == keyboard::Key::Named(key::Named::Tab) {
                        Some(if modifiers.shift() {
                            Message::MetadataFocusPrevious
                        } else {
                            Message::MetadataFocusNext
                        })
                    } else {
                        None
                    }
                }
                event::Event::Keyboard(keyboard::Event::KeyReleased { key, .. }) => {
                    if key == keyboard::Key::Named(key::Named::Shift) {
                        Some(Message::ShiftModifierChanged(false))
                    } else {
                        None
                    }
                }
                _ => None
            }
        });
        let loading_sub = if self.chapters.is_mapping_from_files || self.chapters.is_looking_up_chapters {
            time::every(std::time::Duration::from_millis(100)).map(|_| Message::ChapterLoadingTick)
        } else {
            Subscription::none()
        };
        Subscription::batch([event_sub, loading_sub])
    }
}
