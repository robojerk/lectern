pub mod colors;
pub mod views;
pub mod helpers;
pub mod cover_search;

use colors as colors_module;
pub use colors_module::*;

pub use views::ViewMode;
use cover_search::{CoverResult, download_image, download_images_parallel_threaded, search_cover_art};
use helpers::{parse_audiobook_file, get_audio_files_from_directory, parse_time_string, 
              get_audio_file_duration, extract_chapters_from_file, generate_chapters_from_files,
              validate_chapters, shift_chapter_with_ripple, play_chapter_headless, find_audio_file_for_chapter};

use crate::services::{AudioService, BookMetadata};
use crate::models::Chapter;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ChapterPlaybackState {
    pub chapter_index: usize,
    pub start_time: Instant,
    pub elapsed_ms: u64,
    pub is_playing: bool,
}

use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Application, Command, Element, Length, Settings, Theme, Subscription};
use iced::window;
use iced::event;
use std::path::Path;
use std::collections::HashMap;

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
    DescriptionChanged(String),
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
    CoverImageDownloaded(Result<(String, Vec<u8>), String>), // URL, image data
    SearchCoverImageDownloaded(Result<(String, Vec<u8>), String>), // URL, image data for search results (single)
    SearchCoverImagesDownloaded(Vec<(String, Result<Vec<u8>, String>)>), // Batch download results (background thread)
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
    ChapterPlaybackTick, // Timer tick for playback progress
    ChapterStopPlayback, // Stop current playback
    ChaptersShowSecondsToggled(bool),
    ChaptersGlobalLockToggled,
    ChapterAsinChanged(String), // Manual ASIN entry for chapter lookup
    MapChaptersFromFiles, // Map chapters from audio files (one file = one chapter)
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
    MetadataProviderChanged(String), // Provider name for metadata search
    // Convert messages
    StartConversion,
    BrowseOutputPath,
    OutputPathSelected(Option<String>),
    ConversionCompleted(Result<String, String>),
}

pub struct Lectern {
    // Search state
    pub search_query: String,
    pub search_author: String,
    pub search_by_asin: bool,
    pub is_searching: bool,
    pub search_results: Vec<BookMetadata>,
    pub search_error: Option<String>,
    pub search_current_page: usize,
    pub search_results_per_page: usize,
    pub search_result_covers: HashMap<String, Vec<u8>>, // Cache for search result cover images (URL -> image data)
    pub search_result_downloading: std::sync::Arc<std::sync::Mutex<Vec<String>>>, // URLs currently being downloaded
    
    // Selected book (editing mode)
    pub selected_book: Option<BookMetadata>,
    
    // Editing fields (for metadata tab)
    pub editing_title: String,
    pub editing_subtitle: String,
    pub editing_author: String,
    pub editing_series: String,
    pub editing_series_number: String,
    pub editing_narrator: String,
    pub editing_description: String,
    pub editing_isbn: String,
    pub editing_publisher: String,
    pub editing_publish_year: String,
    pub editing_genre: String,
    pub editing_tags: String,
    pub editing_language: String,
    pub editing_explicit: bool,
    pub editing_abridged: bool,
    
    // File selection state
    pub selected_file_path: Option<String>,
    pub audio_file_paths: Vec<String>, // List of audio files when directory is selected
    pub is_parsing_file: bool,
    pub file_parse_error: Option<String>,
    
    // Cover state
    pub cover_image_path: Option<String>, // Local file path or URL
    pub cover_image_data: Option<Vec<u8>>, // Downloaded image data for URLs
    pub cover_image_url_cached: Option<String>, // URL that corresponds to cached image data
    pub is_searching_cover: bool,
    pub cover_search_results: Vec<CoverResult>,
    pub cover_search_error: Option<String>,
    pub is_downloading_cover: bool,
    
    // Chapter state
    pub chapters: Vec<Chapter>,
    pub chapters_show_seconds: bool,
    pub chapters_global_locked: bool,
    pub is_looking_up_chapters: bool,
    pub chapter_lookup_error: Option<String>,
    pub chapter_playback_state: Option<ChapterPlaybackState>, // Current playback state
    pub chapter_asin_input: String, // Manual ASIN input for chapter lookup
    
    // Current view mode
    pub view_mode: ViewMode,
    
    // Settings state
    pub local_library_path: Option<String>,
    pub media_management_template: String, // e.g., "{Author}/{Series}/{Title}.m4b"
    pub audiobookshelf_host: String,
    pub audiobookshelf_token: String,
    pub audiobookshelf_library_id: String,
    
    // Metadata provider selection
    pub metadata_provider: String, // "auto", "audnexus", "open_library", "google_books"
    
    // Convert state
    pub output_path: Option<String>,
    pub is_converting: bool,
    pub conversion_error: Option<String>,
}

impl Default for Lectern {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            search_author: String::new(),
            search_by_asin: false,
            is_searching: false,
            search_results: Vec::new(),
            search_error: None,
            search_current_page: 0,
            search_results_per_page: 10,
            search_result_covers: std::collections::HashMap::new(),
            search_result_downloading: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            selected_book: None,
            editing_title: String::new(),
            editing_subtitle: String::new(),
            editing_author: String::new(),
            editing_series: String::new(),
            editing_series_number: String::new(),
            editing_narrator: String::new(),
            editing_description: String::new(),
            editing_isbn: String::new(),
            editing_publisher: String::new(),
            editing_publish_year: String::new(),
            editing_genre: String::new(),
            editing_tags: String::new(),
            editing_language: String::new(),
            editing_explicit: false,
            editing_abridged: false,
            selected_file_path: None,
            audio_file_paths: Vec::new(),
            is_parsing_file: false,
            file_parse_error: None,
            cover_image_path: None,
            cover_image_data: None,
            cover_image_url_cached: None,
            is_searching_cover: false,
            cover_search_results: Vec::new(),
            cover_search_error: None,
            is_downloading_cover: false,
            chapters: Vec::new(),
            chapters_show_seconds: false,
            chapters_global_locked: false,
            is_looking_up_chapters: false,
            chapter_lookup_error: None,
            chapter_playback_state: None,
            chapter_asin_input: String::new(),
            view_mode: ViewMode::Metadata,
            local_library_path: None,
            media_management_template: "{Author}/{Title}.m4b".to_string(),
            audiobookshelf_host: String::new(),
            audiobookshelf_token: String::new(),
            audiobookshelf_library_id: String::new(),
            metadata_provider: "auto".to_string(),
            output_path: None,
            is_converting: false,
            conversion_error: None,
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
        match message {
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
            }
            Message::SearchAuthorChanged(author) => {
                self.search_author = author;
            }
            Message::NextPage => {
                let total_pages = (self.search_results.len() + self.search_results_per_page - 1) / self.search_results_per_page;
                if self.search_current_page < total_pages.saturating_sub(1) {
                    self.search_current_page += 1;
                    // Download covers for new page
                    let start_idx = self.search_current_page * self.search_results_per_page;
                    let end_idx = (start_idx + self.search_results_per_page).min(self.search_results.len());
                    let page_results = &self.search_results[start_idx..end_idx];
                    
                    let urls_to_download: Vec<String> = page_results.iter()
                        .filter_map(|book| book.cover_url.as_ref())
                        .filter(|url| (url.starts_with("http://") || url.starts_with("https://")) 
                            && !self.search_result_covers.contains_key(*url))
                        .cloned()
                        .collect();
                    
                    if !urls_to_download.is_empty() {
                        println!("[DEBUG] Starting background download of {} cover images for page {}", urls_to_download.len(), self.search_current_page + 1);
                        let urls_clone = urls_to_download.clone();
                        return Command::perform(
                            async move {
                                let handle = download_images_parallel_threaded(urls_clone);
                                tokio::task::spawn_blocking(move || {
                                    handle.join().unwrap_or_else(|_| vec![])
                                }).await.unwrap_or_else(|_| vec![])
                            },
                            Message::SearchCoverImagesDownloaded,
                        );
                    }
                }
            }
            Message::PreviousPage => {
                if self.search_current_page > 0 {
                    self.search_current_page -= 1;
                    // Download covers for new page
                    let start_idx = self.search_current_page * self.search_results_per_page;
                    let end_idx = (start_idx + self.search_results_per_page).min(self.search_results.len());
                    let page_results = &self.search_results[start_idx..end_idx];
                    
                    let urls_to_download: Vec<String> = page_results.iter()
                        .filter_map(|book| book.cover_url.as_ref())
                        .filter(|url| (url.starts_with("http://") || url.starts_with("https://")) 
                            && !self.search_result_covers.contains_key(*url))
                        .cloned()
                        .collect();
                    
                    if !urls_to_download.is_empty() {
                        println!("[DEBUG] Starting background download of {} cover images for page {}", urls_to_download.len(), self.search_current_page + 1);
                        let urls_clone = urls_to_download.clone();
                        return Command::perform(
                            async move {
                                let handle = download_images_parallel_threaded(urls_clone);
                                tokio::task::spawn_blocking(move || {
                                    handle.join().unwrap_or_else(|_| vec![])
                                }).await.unwrap_or_else(|_| vec![])
                            },
                            Message::SearchCoverImagesDownloaded,
                        );
                    }
                }
            }
            Message::SearchByAsinToggled(enabled) => {
                self.search_by_asin = enabled;
            }
            Message::PerformSearch => {
                if self.search_query.trim().is_empty() {
                    return Command::none();
                }
                
                self.is_searching = true;
                self.search_error = None;
                self.search_results.clear();
                
                let query = self.search_query.clone();
                let author = self.search_author.clone();
                let by_asin = self.search_by_asin;
                let provider = self.metadata_provider.clone();
                self.search_current_page = 0; // Reset to first page on new search
                
                // Combine query and author if both provided
                let search_query = if !author.trim().is_empty() && !query.trim().is_empty() {
                    format!("{} {}", query, author)
                } else if !author.trim().is_empty() {
                    author
                } else {
                    query
                };
                
                // Spawn async search task
                // Create a Tokio runtime for reqwest since Iced's default executor doesn't provide one
                println!("[DEBUG] Starting search for: '{}' (Author: '{}', ASIN: {}, Provider: {})", 
                    self.search_query, self.search_author, by_asin, provider);
                return Command::perform(
                    async move {
                        // Create a new Tokio runtime for this task
                        match tokio::runtime::Runtime::new() {
                            Ok(rt) => {
                                println!("[DEBUG] Tokio runtime created, calling search_metadata...");
                                let provider_opt = if provider == "auto" {
                                    None
                                } else {
                                    Some(provider.as_str())
                                };
                                let result = rt.block_on(AudioService::search_metadata(&search_query, by_asin, provider_opt));
                                match &result {
                                    Ok(books) => println!("[DEBUG] Search returned {} results", books.len()),
                                    Err(e) => println!("[DEBUG] Search error: {}", e),
                                }
                                result
                            },
                            Err(e) => {
                                println!("[DEBUG] Failed to create Tokio runtime: {}", e);
                                Err(format!("Failed to create Tokio runtime: {}", e))
                            },
                        }
                    },
                    Message::SearchCompleted,
                );
            }
            Message::SearchCompleted(Ok(results)) => {
                self.is_searching = false;
                self.search_results = results.clone();
                self.view_mode = ViewMode::Search;
                self.search_current_page = 0; // Reset to first page
                
                // Only download covers for the current page (first 10 results)
                let start_idx = self.search_current_page * self.search_results_per_page;
                let end_idx = (start_idx + self.search_results_per_page).min(results.len());
                let page_results = &results[start_idx..end_idx];
                
                // Collect cover URLs for current page only
                let urls_to_download: Vec<String> = page_results.iter()
                    .filter_map(|book| book.cover_url.as_ref())
                    .filter(|url| (url.starts_with("http://") || url.starts_with("https://")) 
                        && !self.search_result_covers.contains_key(*url))
                    .cloned()
                    .collect();
                
                // Download covers for current page using async Command (non-blocking)
                if !urls_to_download.is_empty() {
                    println!("[DEBUG] Starting async download of {} cover images for page {}", urls_to_download.len(), self.search_current_page + 1);
                    let urls_clone = urls_to_download.clone();
                    let downloading = self.search_result_downloading.clone();
                    
                    // Mark URLs as downloading
                    {
                        let mut downloading_list = downloading.lock().unwrap();
                        downloading_list.extend(urls_clone.iter().cloned());
                    }
                    
                    return Command::perform(
                        async move {
                            // Spawn blocking thread for image downloads (non-blocking for UI)
                            let handle = std::thread::spawn(move || {
                                let join_handle = download_images_parallel_threaded(urls_clone.clone());
                                join_handle.join().unwrap_or_else(|_| vec![])
                            });
                            handle.join().unwrap_or_else(|_| vec![])
                        },
                        Message::SearchCoverImagesDownloaded,
                    );
                }
            }
            Message::SearchCompleted(Err(e)) => {
                self.is_searching = false;
                self.search_error = Some(format!("Search failed: {}", e));
                println!("[ERROR] Search failed: {}", e);
            }
            Message::SelectBook(index) => {
                if let Some(book) = self.search_results.get(index).cloned() {
                    println!("[DEBUG] SelectBook - Populating fields from book: '{}' by '{}'", book.title, book.author);
                    println!("[DEBUG] SelectBook - Book fields: subtitle={:?}, series={:?}, series_number={:?}, narrator={:?}, description={:?}, isbn={:?}, publisher={:?}, publish_year={:?}, genre={:?}, language={:?}, explicit={:?}, abridged={:?}",
                        book.subtitle, book.series, book.series_number, book.narrator, 
                        book.description.as_ref().map(|d| if d.len() > 50 { format!("{}...", &d[..50]) } else { d.clone() }),
                        book.isbn, book.publisher, book.publish_year, book.genre, book.language, book.explicit, book.abridged);
                    
                    self.selected_book = Some(book.clone());
                    self.editing_title = book.title;
                    self.editing_subtitle = book.subtitle.unwrap_or_default();
                    self.editing_author = book.author;
                    self.editing_series = book.series.unwrap_or_default();
                    self.editing_series_number = book.series_number.unwrap_or_default();
                    self.editing_narrator = book.narrator.unwrap_or_default();
                    self.editing_description = book.description.unwrap_or_default();
                    self.editing_isbn = book.isbn.unwrap_or_default();
                    self.editing_publisher = book.publisher.unwrap_or_default();
                    self.editing_publish_year = book.publish_year.unwrap_or_default();
                    self.editing_genre = book.genre.unwrap_or_default();
                    self.editing_tags = book.tags.unwrap_or_default();
                    self.editing_language = book.language.unwrap_or_default();
                    self.editing_explicit = book.explicit.unwrap_or(false);
                    self.editing_abridged = book.abridged.unwrap_or(false);
                    
                    println!("[DEBUG] SelectBook - Populated editing fields: subtitle='{}', series='{}', narrator='{}', isbn='{}', publisher='{}', year='{}', genre='{}', language='{}'",
                        self.editing_subtitle, self.editing_series, self.editing_narrator, 
                        self.editing_isbn, self.editing_publisher, self.editing_publish_year, 
                        self.editing_genre, self.editing_language);
                    // Initialize cover image path
                    self.cover_image_path = book.cover_url.clone();
                    // If cover is a URL, check if we already have it cached
                    if let Some(ref cover_url) = book.cover_url {
                        if cover_url.starts_with("http://") || cover_url.starts_with("https://") {
                            // Check if we already have this URL cached
                            if self.cover_image_url_cached.as_ref() != Some(cover_url) {
                                // URL changed or not cached - download it
                                self.cover_image_data = None; // Clear old cached data
                                self.cover_image_url_cached = None;
                                self.is_downloading_cover = true;
                                let url_clone = cover_url.clone();
                                self.view_mode = ViewMode::Metadata;
                                self.search_results.clear(); // Close search view
                                return Command::perform(
                                    async move {
                                        // Create Tokio runtime for image download
                                        match tokio::runtime::Runtime::new() {
                                            Ok(rt) => {
                                                rt.block_on(download_image(&url_clone))
                                            },
                                            Err(e) => Err(format!("Failed to create Tokio runtime: {}", e)),
                                        }
                                    },
                                    Message::CoverImageDownloaded,
                                );
                            } else {
                                // Already cached - no need to download
                                println!("[DEBUG] Cover image already cached for URL: {}", cover_url);
                            }
                        } else {
                            // Not a URL - clear cache
                            self.cover_image_data = None;
                            self.cover_image_url_cached = None;
                        }
                    } else {
                        // No cover URL - clear cache
                        self.cover_image_data = None;
                        self.cover_image_url_cached = None;
                    }
                    self.view_mode = ViewMode::Metadata;
                    self.search_results.clear(); // Close search view
                }
            }
            Message::SwitchToSearch => {
                self.view_mode = ViewMode::Search;
            }
            Message::SwitchToMetadata => {
                self.view_mode = ViewMode::Metadata;
            }
            Message::SwitchToCover => {
                self.view_mode = ViewMode::Cover;
                // Check if we need to download the cover image
                if let Some(ref cover_path) = self.cover_image_path {
                    if cover_path.starts_with("http://") || cover_path.starts_with("https://") {
                        // Check if we already have this URL cached
                        if self.cover_image_url_cached.as_ref() != Some(cover_path) {
                            // URL changed or not cached - download it
                            if !self.is_downloading_cover {
                                self.cover_image_data = None; // Clear old cached data
                                self.cover_image_url_cached = None;
                                self.is_downloading_cover = true;
                                let url_clone = cover_path.clone();
                                return Command::perform(
                                    async move {
                                        // Create Tokio runtime for image download
                                        match tokio::runtime::Runtime::new() {
                                            Ok(rt) => {
                                                rt.block_on(download_image(&url_clone))
                                            },
                                            Err(e) => Err(format!("Failed to create Tokio runtime: {}", e)),
                                        }
                                    },
                                    Message::CoverImageDownloaded,
                                );
                            }
                        } else {
                            // Already cached - no need to download
                            println!("[DEBUG] Cover image already cached, using cached version");
                        }
                    }
                }
            }
            Message::SwitchToChapters => {
                self.view_mode = ViewMode::Chapters;
            }
            Message::SwitchToConvert => {
                self.view_mode = ViewMode::Convert;
            }
            Message::SwitchToSettings => {
                self.view_mode = ViewMode::Settings;
            }
            Message::LocalLibraryPathChanged(path) => {
                self.local_library_path = if path.trim().is_empty() {
                    None
                } else {
                    Some(path)
                };
            }
            Message::BrowseLocalLibraryPath => {
                return Command::perform(async move {
                    let (tx, rx) = futures::channel::oneshot::channel();
                    std::thread::spawn(move || {
                        let dialog = rfd::FileDialog::new();
                        let result = dialog.pick_folder()
                            .map(|p| p.to_string_lossy().to_string());
                        let _ = tx.send(result);
                    });
                    rx.await.unwrap_or(None)
                }, |path| {
                    Message::LocalLibraryPathSelected(path)
                });
            }
            Message::LocalLibraryPathSelected(Some(path)) => {
                self.local_library_path = Some(path);
            }
            Message::LocalLibraryPathSelected(None) => {
                // User cancelled
            }
            Message::MediaManagementTemplateChanged(template) => {
                self.media_management_template = template;
            }
            Message::AudiobookshelfHostChanged(host) => {
                self.audiobookshelf_host = host;
            }
            Message::AudiobookshelfTokenChanged(token) => {
                self.audiobookshelf_token = token;
            }
            Message::AudiobookshelfLibraryIdChanged(library_id) => {
                self.audiobookshelf_library_id = library_id;
            }
            Message::MetadataProviderChanged(provider) => {
                println!("[DEBUG] MetadataProviderChanged to: '{}'", provider);
                self.metadata_provider = provider;
                println!("[DEBUG] metadata_provider is now: '{}'", self.metadata_provider);
            }
            Message::StartConversion => {
                // Determine output path
                let output_path = if let Some(ref lib_path) = self.local_library_path {
                    // Use template to generate path
                    let template = &self.media_management_template;
                    // For now, use a simple placeholder - full implementation would parse template
                    Some(format!("{}/{}", lib_path, 
                        self.selected_book.as_ref()
                            .map(|b| format!("{}.m4b", b.title.replace("/", "-")))
                            .unwrap_or_else(|| "output.m4b".to_string())))
                } else {
                    // Need to ask user for path
                    self.output_path.clone()
                };
                
                if let Some(path) = output_path {
                    self.is_converting = true;
                    self.conversion_error = None;
                    
                    let selected_book = self.selected_book.clone();
                    let cover_path = self.cover_image_path.clone();
                    let chapters = self.chapters.clone();
                    let audio_files = self.audio_file_paths.clone();
                    
                    return Command::perform(
                        async move {
                            // TODO: Implement actual M4B conversion
                            // This would use FFmpeg to:
                            // 1. Combine audio files
                            // 2. Embed metadata
                            // 3. Embed cover image
                            // 4. Embed chapters
                            
                            // For now, just simulate
                            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                            
                            if selected_book.is_some() {
                                Ok(format!("Conversion would create: {}", path))
                            } else {
                                Err("No book selected".to_string())
                            }
                        },
                        Message::ConversionCompleted,
                    );
                } else {
                    // Need to browse for output path
                    return Command::perform(async move {
                        let (tx, rx) = futures::channel::oneshot::channel();
                        std::thread::spawn(move || {
                            let dialog = rfd::FileDialog::new()
                                .add_filter("M4B Files", &["m4b"])
                                .set_file_name("output.m4b");
                            let result = dialog.save_file()
                                .map(|p| p.to_string_lossy().to_string());
                            let _ = tx.send(result);
                        });
                        rx.await.unwrap_or(None)
                    }, |path| {
                        Message::OutputPathSelected(path)
                    });
                }
            }
            Message::BrowseOutputPath => {
                let default_filename = self.selected_book.as_ref()
                    .map(|b| format!("{}.m4b", b.title.replace("/", "-")))
                    .unwrap_or_else(|| "output.m4b".to_string());
                return Command::perform(async move {
                    let (tx, rx) = futures::channel::oneshot::channel();
                    let filename = default_filename.clone();
                    std::thread::spawn(move || {
                        let dialog = rfd::FileDialog::new()
                            .add_filter("M4B Files", &["m4b"])
                            .set_file_name(&filename);
                        let result = dialog.save_file()
                            .map(|p| p.to_string_lossy().to_string());
                        let _ = tx.send(result);
                    });
                    rx.await.unwrap_or(None)
                }, |path| {
                    Message::OutputPathSelected(path)
                });
            }
            Message::OutputPathSelected(Some(path)) => {
                self.output_path = Some(path);
                // Don't auto-start - let user click the button
            }
            Message::OutputPathSelected(None) => {
                // User cancelled
            }
            Message::ConversionCompleted(Ok(path)) => {
                self.is_converting = false;
                self.conversion_error = None;
                println!("[DEBUG] Conversion completed: {}", path);
                // TODO: Show success message, optionally upload to Audiobookshelf
            }
            Message::ConversionCompleted(Err(e)) => {
                self.is_converting = false;
                self.conversion_error = Some(e);
            }
            Message::TitleChanged(title) => {
                self.editing_title = title;
                if let Some(ref mut book) = self.selected_book {
                    book.title = self.editing_title.clone();
                }
            }
            Message::AuthorChanged(author) => {
                self.editing_author = author;
                if let Some(ref mut book) = self.selected_book {
                    book.author = self.editing_author.clone();
                }
            }
            Message::SeriesChanged(series) => {
                self.editing_series = series;
                if let Some(ref mut book) = self.selected_book {
                    book.series = Some(self.editing_series.clone());
                }
            }
            Message::NarratorChanged(narrator) => {
                self.editing_narrator = narrator;
                if let Some(ref mut book) = self.selected_book {
                    book.narrator = Some(self.editing_narrator.clone());
                }
            }
            Message::DescriptionChanged(desc) => {
                self.editing_description = desc;
                if let Some(ref mut book) = self.selected_book {
                    book.description = Some(self.editing_description.clone());
                }
            }
            Message::SubtitleChanged(subtitle) => {
                self.editing_subtitle = subtitle;
                if let Some(ref mut book) = self.selected_book {
                    book.subtitle = Some(self.editing_subtitle.clone());
                }
            }
            Message::SeriesNumberChanged(num) => {
                self.editing_series_number = num;
                if let Some(ref mut book) = self.selected_book {
                    book.series_number = Some(self.editing_series_number.clone());
                }
            }
            Message::IsbnChanged(isbn) => {
                self.editing_isbn = isbn;
                if let Some(ref mut book) = self.selected_book {
                    book.isbn = Some(self.editing_isbn.clone());
                }
            }
            Message::AsinChanged(asin) => {
                if let Some(ref mut book) = self.selected_book {
                    book.asin = if asin.trim().is_empty() {
                        None
                    } else {
                        Some(asin)
                    };
                }
            }
            Message::PublisherChanged(pub_name) => {
                self.editing_publisher = pub_name;
                if let Some(ref mut book) = self.selected_book {
                    book.publisher = Some(self.editing_publisher.clone());
                }
            }
            Message::PublishYearChanged(year) => {
                self.editing_publish_year = year;
                if let Some(ref mut book) = self.selected_book {
                    book.publish_year = Some(self.editing_publish_year.clone());
                }
            }
            Message::GenreChanged(genre) => {
                self.editing_genre = genre;
                if let Some(ref mut book) = self.selected_book {
                    book.genre = Some(self.editing_genre.clone());
                }
            }
            Message::TagsChanged(tags) => {
                self.editing_tags = tags;
                if let Some(ref mut book) = self.selected_book {
                    book.tags = Some(self.editing_tags.clone());
                }
            }
            Message::LanguageChanged(lang) => {
                self.editing_language = lang;
                if let Some(ref mut book) = self.selected_book {
                    book.language = Some(self.editing_language.clone());
                }
            }
            Message::ExplicitToggled(value) => {
                self.editing_explicit = value;
                if let Some(ref mut book) = self.selected_book {
                    book.explicit = Some(value);
                }
            }
            Message::AbridgedToggled(value) => {
                self.editing_abridged = value;
                if let Some(ref mut book) = self.selected_book {
                    book.abridged = Some(value);
                }
            }
            Message::BrowseFiles => {
                // Open file picker for files
                return Command::perform(async move {
                    let (tx, rx) = futures::channel::oneshot::channel();
                    std::thread::spawn(move || {
                        let dialog = rfd::FileDialog::new()
                            .add_filter("Audiobook Files", &["m4b", "m4a"])
                            .add_filter("Audio Files", &["mp3", "aac", "wav", "flac", "m4b", "m4a"])
                            .add_filter("All Files", &["*"]);
                        
                        let result = dialog.pick_file()
                            .map(|p| p.to_string_lossy().to_string());
                        let _ = tx.send(result);
                    });
                    rx.await.unwrap_or(None)
                }, |path| {
                    if let Some(p) = path {
                        Message::FileSelected(Some(p))
                    } else {
                        Message::FileSelected(None)
                    }
                });
            }
            Message::BrowseFolder => {
                // Open folder picker
                return Command::perform(async move {
                    let (tx, rx) = futures::channel::oneshot::channel();
                    std::thread::spawn(move || {
                        let dialog = rfd::FileDialog::new();
                        let result = dialog.pick_folder()
                            .map(|p| p.to_string_lossy().to_string());
                        let _ = tx.send(result);
                    });
                    rx.await.unwrap_or(None)
                }, |path| {
                    if let Some(p) = path {
                        Message::FileSelected(Some(p))
                    } else {
                        Message::FileSelected(None)
                    }
                });
            }
            Message::FileSelected(Some(path)) => {
                self.selected_file_path = Some(path.clone());
                self.is_parsing_file = true;
                self.file_parse_error = None;
                
                // Parse file (synchronous operation)
                let path_clone = path.clone();
                return Command::perform(
                    async move {
                        // File parsing is synchronous, so we just call it directly
                        parse_audiobook_file(&path_clone)
                    },
                    Message::FileParsed,
                );
            }
            Message::FileDropped(paths) => {
                println!("[DEBUG] FileDropped handler - received {} paths: {:?}", paths.len(), paths);
                
                // Filter out invalid paths (like "/" or empty strings)
                // Also, if we have multiple paths and the first is "/", it might be a path split issue
                // Try to reconstruct the full path if it looks like it was split
                let valid_paths: Vec<String> = if paths.len() > 1 && paths.first().map(|p| p.as_str()) == Some("/") {
                    // Path might have been split into components - try to reconstruct
                    // Join all components, but add "/" between them if needed
                    let mut reconstructed = String::new();
                    for (i, component) in paths.iter().enumerate() {
                        if i == 0 {
                            reconstructed.push_str(component);
                        } else {
                            // Add separator if previous component doesn't end with /
                            if !reconstructed.ends_with('/') {
                                reconstructed.push('/');
                            }
                            reconstructed.push_str(component);
                        }
                    }
                    println!("[DEBUG] Attempting to reconstruct path from {} components: '{}'", paths.len(), reconstructed);
                    if Path::new(&reconstructed).exists() {
                        vec![reconstructed]
                    } else {
                        // Try alternative: join with no separator (in case components already have separators)
                        let alt_reconstructed = paths.join("");
                        println!("[DEBUG] Trying alternative reconstruction: '{}'", alt_reconstructed);
                        if Path::new(&alt_reconstructed).exists() {
                            vec![alt_reconstructed]
                        } else {
                            // Fall back to filtering - find the longest valid path
                            let mut filtered: Vec<String> = paths.iter()
                                .filter(|p| {
                                    let path = Path::new(p);
                                    !p.is_empty() && p.as_str() != "/" && path.exists()
                                })
                                .cloned()
                                .collect();
                            // Sort by length (longest first) to prefer full paths over components
                            filtered.sort_by(|a, b| b.len().cmp(&a.len()));
                            filtered
                        }
                    }
                } else {
                    // Normal filtering - find longest valid path
                    let mut filtered: Vec<String> = paths.iter()
                        .filter(|p| {
                            let path = Path::new(p);
                            !p.is_empty() && p.as_str() != "/" && path.exists()
                        })
                        .cloned()
                        .collect();
                    // Sort by length (longest first) to prefer full paths
                    filtered.sort_by(|a, b| b.len().cmp(&a.len()));
                    filtered
                };
                
                println!("[DEBUG] Filtered to {} valid paths: {:?}", valid_paths.len(), valid_paths);
                
                // Handle dropped files or folders - take the first valid one
                if let Some(path) = valid_paths.first() {
                    println!("[DEBUG] Processing dropped path: '{}'", path);
                    self.selected_file_path = Some(path.clone());
                    self.is_parsing_file = true;
                    self.file_parse_error = None;
                    
                    let path_clone = path.clone();
                    // Check if it's a directory or file
                    let path_obj = Path::new(&path_clone);
                    if path_obj.is_dir() {
                        println!("[DEBUG] Path is a directory, scanning for audio files...");
                        // For directories, collect audio files first
                        let audio_files = get_audio_files_from_directory(&path_clone);
                        println!("[DEBUG] Found {} audio files in directory", audio_files.len());
                        if !audio_files.is_empty() {
                            // Store all audio file paths for chapter mapping
                            self.audio_file_paths = audio_files.clone();
                            // Parse the directory itself (not individual files) to get directory metadata
                            println!("[DEBUG] Parsing directory metadata for: '{}'", path_clone);
                            return Command::perform(
                                async move {
                                    let result = parse_audiobook_file(&path_clone);
                                    match &result {
                                        Ok(meta) => println!("[DEBUG] Directory parsed successfully: '{}' by '{}' ({} files)", 
                                            meta.title, meta.author, audio_files.len()),
                                        Err(e) => println!("[ERROR] Directory parse error: {}", e),
                                    }
                                    result
                                },
                                Message::FileParsed,
                            );
                        } else {
                            self.is_parsing_file = false;
                            let error_msg = format!("No audio files found in directory: {}", path_clone);
                            println!("[ERROR] {}", error_msg);
                            self.file_parse_error = Some(error_msg);
                        }
                    } else {
                        // For single files, parse directly
                        println!("[DEBUG] Path is a file, parsing directly...");
                        return Command::perform(
                            async move {
                                let result = parse_audiobook_file(&path_clone);
                                match &result {
                                    Ok(meta) => println!("[DEBUG] File parsed successfully: '{}' by '{}'", meta.title, meta.author),
                                    Err(e) => println!("[DEBUG] File parse error: {}", e),
                                }
                                result
                            },
                            Message::FileParsed,
                        );
                    }
                } else {
                    let error_msg = format!("No valid paths in dropped files. Received {} paths, but none were valid.", paths.len());
                    println!("[ERROR] {}", error_msg);
                    self.file_parse_error = Some(error_msg);
                }
            }
            Message::FileSelected(None) => {
                // User cancelled file selection
            }
            Message::FileParsed(Ok(metadata)) => {
                println!("[DEBUG] FileParsed(Ok) - Successfully parsed file/directory");
                self.is_parsing_file = false;
                self.selected_book = Some(metadata.clone());
                // Populate editing fields
                self.editing_title = metadata.title.clone();
                self.editing_subtitle = metadata.subtitle.unwrap_or_default();
                self.editing_author = metadata.author.clone();
                self.editing_series = metadata.series.unwrap_or_default();
                self.editing_series_number = metadata.series_number.unwrap_or_default();
                self.editing_narrator = metadata.narrator.unwrap_or_default();
                self.editing_description = metadata.description.unwrap_or_default();
                self.editing_isbn = metadata.isbn.unwrap_or_default();
                self.editing_publisher = metadata.publisher.unwrap_or_default();
                self.editing_publish_year = metadata.publish_year.unwrap_or_default();
                self.editing_genre = metadata.genre.unwrap_or_default();
                self.editing_tags = metadata.tags.unwrap_or_default();
                self.editing_language = metadata.language.unwrap_or_default();
                self.editing_explicit = metadata.explicit.unwrap_or(false);
                self.editing_abridged = metadata.abridged.unwrap_or(false);
                // Initialize cover image path and handle caching
                self.cover_image_path = metadata.cover_url.clone();
                // If cover is a URL, check if we already have it cached
                if let Some(ref cover_url) = metadata.cover_url {
                    if cover_url.starts_with("http://") || cover_url.starts_with("https://") {
                        // Check if we already have this URL cached
                        if self.cover_image_url_cached.as_ref() != Some(cover_url) {
                            // URL changed or not cached - download it
                            self.cover_image_data = None; // Clear old cached data
                            self.cover_image_url_cached = None;
                            self.is_downloading_cover = true;
                            let url_clone = cover_url.clone();
                            return Command::perform(
                                async move {
                                    // Create Tokio runtime for image download
                                    match tokio::runtime::Runtime::new() {
                                        Ok(rt) => {
                                            rt.block_on(download_image(&url_clone))
                                        },
                                        Err(e) => Err(format!("Failed to create Tokio runtime: {}", e)),
                                    }
                                },
                                Message::CoverImageDownloaded,
                            );
                        } else {
                            // Already cached - no need to download
                            println!("[DEBUG] Cover image already cached for URL: {}", cover_url);
                        }
                    } else {
                        // Not a URL - clear cache
                        self.cover_image_data = None;
                        self.cover_image_url_cached = None;
                    }
                } else {
                    // No cover URL - clear cache
                    self.cover_image_data = None;
                    self.cover_image_url_cached = None;
                }
                
                // Store audio file paths if directory was selected
                if let Some(ref file_path) = self.selected_file_path {
                    if Path::new(file_path).is_dir() {
                        // Re-scan to ensure we have all files (in case parse_audiobook_file didn't populate them)
                        let audio_files = get_audio_files_from_directory(file_path);
                        self.audio_file_paths = audio_files;
                        println!("[DEBUG] Stored {} audio file paths for chapter mapping", self.audio_file_paths.len());
                    } else {
                        self.audio_file_paths.clear();
                    }
                }
                
                println!("[DEBUG] FileParsed - Switching to Metadata view");
                self.view_mode = ViewMode::Metadata;
            }
            Message::FileParsed(Err(e)) => {
                println!("[DEBUG] FileParsed(Err) - Error: {}", e);
                self.is_parsing_file = false;
                self.file_parse_error = Some(e.clone());
                println!("[ERROR] Failed to parse file/directory: {}", e);
            }
            Message::BrowseCoverImage => {
                // Open image file picker
                return Command::perform(async move {
                    let (tx, rx) = futures::channel::oneshot::channel();
                    std::thread::spawn(move || {
                        let dialog = rfd::FileDialog::new()
                            .add_filter("Image Files", &["jpg", "jpeg", "png", "gif", "webp", "bmp"])
                            .add_filter("All Files", &["*"]);
                        let result = dialog.pick_file()
                            .map(|p| p.to_string_lossy().to_string());
                        let _ = tx.send(result);
                    });
                    rx.await.unwrap_or(None)
                }, |path| {
                    Message::CoverImageSelected(path)
                });
            }
            Message::CoverImageSelected(Some(path)) => {
                self.cover_image_path = Some(path);
                if let Some(ref mut book) = self.selected_book {
                    book.cover_url = self.cover_image_path.clone();
                }
            }
            Message::CoverImageSelected(None) => {
                // User cancelled
            }
            Message::SearchCover => {
                // Search for cover using current book metadata
                self.is_searching_cover = true;
                self.cover_search_error = None;
                self.cover_search_results.clear();
                
                let title = self.editing_title.clone();
                let author = self.editing_author.clone();
                let isbn = if self.editing_isbn.is_empty() {
                    None
                } else {
                    Some(self.editing_isbn.clone())
                };
                let asin = self.selected_book.as_ref()
                    .and_then(|b| b.asin.clone());
                
                println!("[DEBUG] Searching for cover art - Title: '{}', Author: '{}', ASIN: {:?}", title, author, asin);
                
                return Command::perform(
                    async move {
                        // Create Tokio runtime for HTTP requests
                        match tokio::runtime::Runtime::new() {
                            Ok(rt) => {
                                let result = rt.block_on(search_cover_art(&title, &author, isbn.as_deref(), asin.as_deref()));
                                match &result {
                                    Ok(covers) => println!("[DEBUG] Cover search found {} results", covers.len()),
                                    Err(e) => println!("[DEBUG] Cover search error: {}", e),
                                }
                                result
                            },
                            Err(e) => Err(format!("Failed to create Tokio runtime: {}", e)),
                        }
                    },
                    Message::CoverSearchCompleted,
                );
            }
            Message::CoverSearchCompleted(Ok(results)) => {
                self.is_searching_cover = false;
                self.cover_search_results = results;
                println!("[DEBUG] Cover search completed: {} results displayed", self.cover_search_results.len());
            }
            Message::CoverSearchCompleted(Err(e)) => {
                self.is_searching_cover = false;
                println!("[DEBUG] Cover search error: {}", e);
                self.cover_search_error = Some(e);
            }
            Message::SelectCover(index) => {
                if let Some(cover) = self.cover_search_results.get(index) {
                    self.cover_image_path = Some(cover.url.clone());
                    // If it's a URL, check if we already have it cached
                    if cover.url.starts_with("http://") || cover.url.starts_with("https://") {
                        // Check if we already have this URL cached
                        if self.cover_image_url_cached.as_ref() != Some(&cover.url) {
                            // URL changed or not cached - download it
                            self.cover_image_data = None; // Clear old cached data
                            self.cover_image_url_cached = None;
                            self.is_downloading_cover = true;
                            let url_clone = cover.url.clone();
                            if let Some(ref mut book) = self.selected_book {
                                book.cover_url = Some(cover.url.clone());
                            }
                            return Command::perform(
                                async move {
                                    // Create Tokio runtime for image download
                                    match tokio::runtime::Runtime::new() {
                                        Ok(rt) => {
                                            rt.block_on(download_image(&url_clone))
                                        },
                                        Err(e) => Err(format!("Failed to create Tokio runtime: {}", e)),
                                    }
                                },
                                Message::CoverImageDownloaded,
                            );
                        } else {
                            // Already cached - no need to download
                            println!("[DEBUG] Cover image already cached for URL: {}", cover.url);
                            if let Some(ref mut book) = self.selected_book {
                                book.cover_url = Some(cover.url.clone());
                            }
                        }
                    } else {
                        // Not a URL - clear cache
                        self.cover_image_data = None;
                        self.cover_image_url_cached = None;
                        if let Some(ref mut book) = self.selected_book {
                            book.cover_url = Some(cover.url.clone());
                        }
                    }
                }
            }
            Message::CoverUrlChanged(url) => {
                let trimmed_url = url.trim();
                println!("[DEBUG] Cover URL changed: '{}'", trimmed_url);
                self.cover_image_path = if trimmed_url.is_empty() {
                    None
                } else {
                    Some(trimmed_url.to_string())
                };
                // If it's a URL, check if we already have it cached
                if let Some(ref url_path) = self.cover_image_path {
                    if url_path.starts_with("http://") || url_path.starts_with("https://") {
                        // Check if we already have this URL cached
                        if self.cover_image_url_cached.as_ref() != Some(url_path) {
                            // URL changed or not cached - download it
                            self.cover_image_data = None; // Clear old cached data
                            self.cover_image_url_cached = None;
                            self.is_downloading_cover = true;
                            let url_clone = url_path.clone();
                            return Command::perform(
                                async move {
                                    // Create Tokio runtime for image download
                                    match tokio::runtime::Runtime::new() {
                                        Ok(rt) => {
                                            rt.block_on(download_image(&url_clone))
                                        },
                                        Err(e) => Err(format!("Failed to create Tokio runtime: {}", e)),
                                    }
                                },
                                Message::CoverImageDownloaded,
                            );
                        } else {
                            // Already cached - no need to download
                            println!("[DEBUG] Cover image already cached for URL: {}", url_path);
                        }
                    } else {
                        // Not a URL - clear cache
                        self.cover_image_data = None;
                        self.cover_image_url_cached = None;
                    }
                } else {
                    // No URL - clear cache
                    self.cover_image_data = None;
                    self.cover_image_url_cached = None;
                }
                if let Some(ref mut book) = self.selected_book {
                    book.cover_url = self.cover_image_path.clone();
                }
                println!("[DEBUG] Cover image path set to: {:?}", self.cover_image_path);
            }
            Message::DownloadCoverImage(url) => {
                self.is_downloading_cover = true;
                return Command::perform(
                    async move {
                        // Create Tokio runtime for image download
                        match tokio::runtime::Runtime::new() {
                            Ok(rt) => {
                                rt.block_on(download_image(&url))
                            },
                            Err(e) => Err(format!("Failed to create Tokio runtime: {}", e)),
                        }
                    },
                    Message::CoverImageDownloaded,
                );
            }
            Message::CoverImageDownloaded(Ok((url, image_data))) => {
                self.is_downloading_cover = false;
                self.cover_image_data = Some(image_data);
                self.cover_image_url_cached = Some(url.clone());
                println!("[DEBUG] Successfully downloaded and cached cover image from: {}", url);
            }
            Message::CoverImageDownloaded(Err(e)) => {
                self.is_downloading_cover = false;
                println!("[DEBUG] Failed to download cover image: {}", e);
                self.cover_search_error = Some(format!("Failed to download image: {}", e));
            }
            Message::SearchCoverImagesDownloaded(results) => {
                // Remove from downloading list
                {
                    let mut downloading_list = self.search_result_downloading.lock().unwrap();
                    downloading_list.clear();
                }
                
                // Cache all successfully downloaded images
                let mut success_count = 0;
                let mut fail_count = 0;
                for (url, result) in results {
                    match result {
                        Ok(image_data) => {
                            self.search_result_covers.insert(url.clone(), image_data);
                            success_count += 1;
                            println!("[DEBUG] Successfully downloaded and cached search result cover from: {}", url);
                        },
                        Err(e) => {
                            fail_count += 1;
                            println!("[DEBUG] Failed to download search result cover from {}: {}", url, e);
                        }
                    }
                }
                println!("[DEBUG] Batch download complete: {} succeeded, {} failed", success_count, fail_count);
            }
            Message::SearchCoverImageDownloaded(Ok((url, image_data))) => {
                // Legacy handler for single downloads (kept for compatibility)
                self.search_result_covers.insert(url.clone(), image_data);
                println!("[DEBUG] Successfully downloaded and cached search result cover from: {}", url);
            }
            Message::SearchCoverImageDownloaded(Err(e)) => {
                println!("[DEBUG] Failed to download search result cover image: {}", e);
            }
            // Chapter management
            Message::ChapterTitleChanged(index, new_title) => {
                if let Some(chapter) = self.chapters.get_mut(index) {
                    chapter.title = new_title;
                }
            }
            Message::ChapterTimeChanged(index, time_str) => {
                if let Some(chapter) = self.chapters.get_mut(index) {
                    if let Ok(seconds) = parse_time_string(&time_str) {
                        chapter.start_time = (seconds * 1000) as u64;
                    }
                }
            }
            Message::ChapterTimeAdjusted(index, adjustment_seconds) => {
                if let Some(chapter) = self.chapters.get_mut(index) {
                    if !chapter.is_locked {
                        let current_seconds = (chapter.start_time / 1000) as i64;
                        let new_seconds = (current_seconds + adjustment_seconds).max(0);
                        chapter.start_time = (new_seconds * 1000) as u64;
                    }
                }
            }
            Message::ChapterLockToggled(index) => {
                if let Some(chapter) = self.chapters.get_mut(index) {
                    chapter.is_locked = !chapter.is_locked;
                }
            }
            Message::ChapterDelete(index) => {
                if index < self.chapters.len() {
                    self.chapters.remove(index);
                }
            }
            Message::ChapterInsertBelow(index) => {
                let new_start_time = if index < self.chapters.len() {
                    let current = &self.chapters[index];
                    current.start_time + current.duration
                } else if let Some(last) = self.chapters.last() {
                    last.start_time + last.duration
                } else {
                    0
                };
                let new_chapter = Chapter::new(
                    format!("Chapter {}", self.chapters.len() + 1),
                    new_start_time,
                    0, // Duration will be calculated
                );
                if index + 1 < self.chapters.len() {
                    self.chapters.insert(index + 1, new_chapter);
                } else {
                    self.chapters.push(new_chapter);
                }
            }
            Message::ChapterRemoveAll => {
                self.chapters.clear();
            }
            Message::ChapterShiftTimes(seconds) => {
                for chapter in &mut self.chapters {
                    if !chapter.is_locked {
                        let current_seconds = (chapter.start_time / 1000) as i64;
                        let new_seconds = (current_seconds + seconds).max(0);
                        chapter.start_time = (new_seconds * 1000) as u64;
                    }
                }
            }
            Message::ChapterLookup => {
                self.is_looking_up_chapters = true;
                self.chapter_lookup_error = None;
                
                // Get ASIN from manual input, selected book, or editing fields
                let asin = if !self.chapter_asin_input.trim().is_empty() {
                    Some(self.chapter_asin_input.trim().to_string())
                } else {
                    self.selected_book.as_ref()
                        .and_then(|b| b.asin.clone())
                };
                
                if let Some(asin_val) = asin {
                    println!("[DEBUG] Looking up chapters for ASIN: {}", asin_val);
                    return Command::perform(
                        async move {
                            match tokio::runtime::Runtime::new() {
                                Ok(rt) => {
                                    rt.block_on(AudioService::fetch_chapters_by_asin(&asin_val))
                                },
                                Err(e) => Err(format!("Failed to create Tokio runtime: {}", e)),
                            }
                        },
                        Message::ChapterLookupCompleted,
                    );
                } else {
                    self.is_looking_up_chapters = false;
                    self.chapter_lookup_error = Some("No ASIN available. Please enter an ASIN in the field above or search for a book first.".to_string());
                }
            }
            Message::ChapterAsinChanged(asin) => {
                self.chapter_asin_input = asin;
            }
            Message::MapChaptersFromFiles => {
                if self.audio_file_paths.is_empty() {
                    self.chapter_lookup_error = Some("No audio files found. Please select a directory with audio files first.".to_string());
                } else {
                    // Use helper function to generate chapters with real durations
                    match generate_chapters_from_files(&self.audio_file_paths) {
                        Ok(chapters) => {
                            self.chapters = chapters;
                            println!("[DEBUG] Mapped {} chapters from {} audio files", self.chapters.len(), self.audio_file_paths.len());
                        },
                        Err(e) => {
                            self.chapter_lookup_error = Some(format!("Failed to generate chapters: {}", e));
                            println!("[ERROR] Failed to generate chapters: {}", e);
                        }
                    }
                }
            }
            Message::ChapterExtractFromFile => {
                // Extract chapters from the selected file
                if let Some(ref file_path) = self.selected_file_path {
                    self.is_looking_up_chapters = true;
                    self.chapter_lookup_error = None;
                    let path_clone = file_path.clone();
                    
                    return Command::perform(
                        async move {
                            // Extract chapters in background thread to avoid blocking UI
                            tokio::task::spawn_blocking(move || {
                                extract_chapters_from_file(&path_clone)
                            }).await.unwrap_or_else(|_| Err("Task join error".to_string()))
                        },
                        Message::ChapterExtractCompleted,
                    );
                } else {
                    self.chapter_lookup_error = Some("No file selected. Please select an audio file first.".to_string());
                }
            }
            Message::ChapterExtractCompleted(Ok(chapters)) => {
                self.is_looking_up_chapters = false;
                if !chapters.is_empty() {
                    self.chapters = chapters;
                    println!("[DEBUG] Extracted {} chapters from file", self.chapters.len());
                } else {
                    self.chapter_lookup_error = Some("No chapters found in file".to_string());
                }
            }
            Message::ChapterExtractCompleted(Err(e)) => {
                self.is_looking_up_chapters = false;
                self.chapter_lookup_error = Some(format!("Failed to extract chapters: {}", e));
                println!("[ERROR] Failed to extract chapters: {}", e);
            }
            Message::ChapterShiftAll(offset_ms) => {
                // Shift all unlocked chapters by offset
                let chapters_count = self.chapters.len();
                for chapter in &mut self.chapters {
                    if !chapter.is_locked {
                        // Apply offset, but don't allow negative start times
                        if offset_ms < 0 && chapter.start_time < (-offset_ms) as u64 {
                            chapter.start_time = 0;
                        } else {
                            chapter.start_time = (chapter.start_time as i64 + offset_ms).max(0) as u64;
                        }
                    }
                }
                println!("[DEBUG] Shifted {} unlocked chapters by {} ms", chapters_count, offset_ms);
            }
            Message::ChapterValidate => {
                // Validate chapters and show errors/warnings
                let total_duration = self.selected_file_path.as_ref()
                    .and_then(|path| {
                        // Try to get total duration from first file or selected file
                        if Path::new(path).is_file() {
                            get_audio_file_duration(path).ok()
                        } else if !self.audio_file_paths.is_empty() {
                            // Sum all file durations
                            let mut total = 0u64;
                            for file in &self.audio_file_paths {
                                if let Ok(dur) = get_audio_file_duration(file) {
                                    total += dur;
                                }
                            }
                            if total > 0 { Some(total) } else { None }
                        } else {
                            None
                        }
                    });
                
                let errors = validate_chapters(&self.chapters, total_duration);
                if errors.is_empty() {
                    self.chapter_lookup_error = None;
                    println!("[DEBUG] Chapter validation passed");
                } else {
                    let error_msg = format!("Validation issues: {}", errors.join("; "));
                    self.chapter_lookup_error = Some(error_msg.clone());
                    println!("[WARNING] Chapter validation: {}", error_msg);
                }
            }
            Message::ChapterShiftWithRipple(index, new_start_ms) => {
                // Shift individual chapter with ripple effect
                match shift_chapter_with_ripple(&mut self.chapters, index, new_start_ms) {
                    Ok(()) => {
                        println!("[DEBUG] Shifted chapter {} to {} ms with ripple effect", index + 1, new_start_ms);
                    },
                    Err(e) => {
                        self.chapter_lookup_error = Some(e.clone());
                        println!("[ERROR] Failed to shift chapter: {}", e);
                    }
                }
            }
            Message::ChapterPlay(index) => {
                // Stop any existing playback
                if let Some(ref mut state) = self.chapter_playback_state {
                    if state.is_playing {
                        // Stop will be handled by ChapterStopPlayback
                        return Command::none();
                    }
                }
                
                // Play chapter from its start time
                if index >= self.chapters.len() {
                    self.chapter_lookup_error = Some("Invalid chapter index".to_string());
                    return Command::none();
                }
                
                let chapter = &self.chapters[index];
                // Play for 30 seconds as a preview, or full chapter duration if less
                let preview_duration_ms = Some(chapter.duration.min(30000));
                
                // Find the audio file to play and get the correct start time
                if let Some((file_path, start_time_ms)) = find_audio_file_for_chapter(
                    self.selected_file_path.as_ref(),
                    &self.audio_file_paths,
                    index,
                    chapter.start_time,
                ) {
                    // Start playback in background
                    match play_chapter_headless(&file_path, start_time_ms, preview_duration_ms) {
                        Ok(_child) => {
                            // Set playback state
                            self.chapter_playback_state = Some(ChapterPlaybackState {
                                chapter_index: index,
                                start_time: Instant::now(),
                                elapsed_ms: 0,
                                is_playing: true,
                            });
                            
                            println!("[DEBUG] Started playing chapter {} from {} at {} ms", 
                                index + 1, file_path, start_time_ms);
                            
                            // Start timer to update elapsed time every 100ms
                            // Use std::thread::sleep in a blocking thread (Iced doesn't provide Tokio runtime)
                            return Command::perform(
                                async move {
                                    std::thread::sleep(Duration::from_millis(100));
                                },
                                |_| Message::ChapterPlaybackTick,
                            );
                        },
                        Err(e) => {
                            self.chapter_lookup_error = Some(format!("Failed to play chapter: {}", e));
                            println!("[ERROR] Failed to play chapter: {}", e);
                        }
                    }
                } else {
                    self.chapter_lookup_error = Some("No audio file found to play".to_string());
                    println!("[ERROR] No audio file found for chapter {}", index + 1);
                }
            }
            Message::ChapterPlaybackTick => {
                // Update elapsed time
                if let Some(ref mut state) = self.chapter_playback_state {
                    if state.is_playing {
                        state.elapsed_ms = state.start_time.elapsed().as_millis() as u64;
                        
                        // Check if playback should have finished (30 second preview)
                        let chapter = &self.chapters[state.chapter_index];
                        let max_duration = chapter.duration.min(30000);
                        if state.elapsed_ms >= max_duration {
                            state.is_playing = false;
                            self.chapter_playback_state = None;
                            return Command::none();
                        }
                        
                        // Schedule next tick using blocking sleep (Iced doesn't provide Tokio runtime)
                        return Command::perform(
                            async move {
                                std::thread::sleep(Duration::from_millis(100));
                            },
                            |_| Message::ChapterPlaybackTick,
                        );
                    }
                }
            }
            Message::ChapterStopPlayback => {
                if let Some(ref mut state) = self.chapter_playback_state {
                    state.is_playing = false;
                }
                self.chapter_playback_state = None;
                println!("[DEBUG] Stopped chapter playback");
            }
            Message::ChapterLookupCompleted(Ok(chapters)) => {
                self.is_looking_up_chapters = false;
                let count = chapters.len();
                if !chapters.is_empty() {
                    self.chapters = chapters;
                    println!("[DEBUG] Loaded {} chapters from provider", count);
                    // Note: User can now use "Shift all" buttons to adjust if needed
                }
            }
            Message::ChapterLookupCompleted(Err(e)) => {
                self.is_looking_up_chapters = false;
                self.chapter_lookup_error = Some(e);
            }
            Message::ChaptersShowSecondsToggled(show) => {
                self.chapters_show_seconds = show;
            }
            Message::ChaptersGlobalLockToggled => {
                let locked = !self.chapters_global_locked;
                self.chapters_global_locked = locked;
                for chapter in &mut self.chapters {
                    chapter.is_locked = locked;
                }
            }
        }
        
        Command::none()
    }


    fn view(&self) -> Element<Message> {
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
