mod services;
mod models;

use iced::widget::{
    button, checkbox, column, container, row, scrollable, text, text_input, Column, Space, image,
};
use iced::widget::image::Handle;
use iced::{Alignment, Application, Command, Element, Length, Settings, Theme, Subscription};
use iced::window;
use iced::event::{self, Event};
use services::{AudioService, BookMetadata};
use models::Chapter;
use std::path::Path;

pub fn main() -> iced::Result {
    Lectern::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(900.0, 700.0),
            ..Default::default()
        },
        ..Default::default()
    })
}

#[derive(Debug, Clone)]
enum Message {
    SearchQueryChanged(String),
    SearchByAsinToggled(bool),
    PerformSearch,
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
    ChaptersShowSecondsToggled(bool),
    ChaptersGlobalLockToggled,
    ChapterAsinChanged(String), // Manual ASIN entry for chapter lookup
    MapChaptersFromFiles, // Map chapters from audio files (one file = one chapter)
}

struct Lectern {
    // Search state
    search_query: String,
    search_by_asin: bool,
    is_searching: bool,
    search_results: Vec<BookMetadata>,
    search_error: Option<String>,
    
    // Selected book (editing mode)
    selected_book: Option<BookMetadata>,
    
    // Editing fields (for metadata tab)
    editing_title: String,
    editing_subtitle: String,
    editing_author: String,
    editing_series: String,
    editing_series_number: String,
    editing_narrator: String,
    editing_description: String,
    editing_isbn: String,
    editing_publisher: String,
    editing_publish_year: String,
    editing_genre: String,
    editing_tags: String,
    editing_language: String,
    editing_explicit: bool,
    editing_abridged: bool,
    
    // File selection state
    selected_file_path: Option<String>,
    audio_file_paths: Vec<String>, // List of audio files when directory is selected
    is_parsing_file: bool,
    file_parse_error: Option<String>,
    
    // Cover state
    cover_image_path: Option<String>, // Local file path or URL
    is_searching_cover: bool,
    cover_search_results: Vec<CoverResult>,
    cover_search_error: Option<String>,
    
    // Chapter state
    chapters: Vec<Chapter>,
    chapters_show_seconds: bool,
    chapters_global_locked: bool,
    is_looking_up_chapters: bool,
    chapter_lookup_error: Option<String>,
    chapter_asin_input: String, // Manual ASIN input for chapter lookup
    
    // Current view mode
    view_mode: ViewMode,
}

#[derive(Debug, Clone)]
struct CoverResult {
    url: String,
    width: u32,
    height: u32,
    source: String, // Provider name
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ViewMode {
    Search,
    Metadata,
    Cover,
    Chapters,
    Convert,
}

impl Default for Lectern {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            search_by_asin: false,
            is_searching: false,
            search_results: Vec::new(),
            search_error: None,
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
            is_searching_cover: false,
            cover_search_results: Vec::new(),
            cover_search_error: None,
            chapters: Vec::new(),
            chapters_show_seconds: false,
            chapters_global_locked: false,
            is_looking_up_chapters: false,
            chapter_lookup_error: None,
            chapter_asin_input: String::new(),
            view_mode: ViewMode::Metadata,
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
                let by_asin = self.search_by_asin;
                
                // Spawn async search task
                // Create a Tokio runtime for reqwest since Iced's default executor doesn't provide one
                println!("[DEBUG] Starting search for: '{}' (ASIN: {})", query, by_asin);
                return Command::perform(
                    async move {
                        // Create a new Tokio runtime for this task
                        match tokio::runtime::Runtime::new() {
                            Ok(rt) => {
                                println!("[DEBUG] Tokio runtime created, calling search_metadata...");
                                let result = rt.block_on(AudioService::search_metadata(&query, by_asin));
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
                self.search_results = results;
                self.view_mode = ViewMode::Search;
            }
            Message::SearchCompleted(Err(e)) => {
                self.is_searching = false;
                self.search_error = Some(format!("Search failed: {}", e));
                println!("[ERROR] Search failed: {}", e);
            }
            Message::SelectBook(index) => {
                if let Some(book) = self.search_results.get(index).cloned() {
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
                    // Initialize cover image path
                    self.cover_image_path = book.cover_url.clone();
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
            }
            Message::SwitchToChapters => {
                self.view_mode = ViewMode::Chapters;
            }
            Message::SwitchToConvert => {
                self.view_mode = ViewMode::Convert;
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
                // Handle dropped files - take the first one
                if let Some(path) = paths.first() {
                    self.selected_file_path = Some(path.clone());
                    self.is_parsing_file = true;
                    self.file_parse_error = None;
                    
                    let path_clone = path.clone();
                    // Parse file (synchronous operation)
                    return Command::perform(
                        async move {
                            parse_audiobook_file(&path_clone)
                        },
                        Message::FileParsed,
                    );
                }
            }
            Message::FileSelected(None) => {
                // User cancelled file selection
            }
            Message::FileParsed(Ok(metadata)) => {
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
                // Initialize cover image path
                self.cover_image_path = metadata.cover_url.clone();
                
                // Store audio file paths if directory was selected
                if let Some(ref file_path) = self.selected_file_path {
                    if Path::new(file_path).is_dir() {
                        self.audio_file_paths = get_audio_files_from_directory(file_path);
                        println!("[DEBUG] Found {} audio files in directory", self.audio_file_paths.len());
                    } else {
                        self.audio_file_paths.clear();
                    }
                }
                
                self.view_mode = ViewMode::Metadata;
            }
            Message::FileParsed(Err(e)) => {
                self.is_parsing_file = false;
                self.file_parse_error = Some(e);
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
                    if let Some(ref mut book) = self.selected_book {
                        book.cover_url = Some(cover.url.clone());
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
                if let Some(ref mut book) = self.selected_book {
                    book.cover_url = self.cover_image_path.clone();
                }
                println!("[DEBUG] Cover image path set to: {:?}", self.cover_image_path);
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
                    // Create one chapter per audio file
                    self.chapters.clear();
                    let mut cumulative_time = 0u64;
                    
                    for (index, file_path) in self.audio_file_paths.iter().enumerate() {
                        let file_name = Path::new(file_path)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or(&format!("Chapter {}", index + 1))
                            .to_string();
                        
                        // Estimate duration (we'll use a placeholder - in real implementation, would get from file)
                        let estimated_duration = 3600000u64; // 1 hour in milliseconds (placeholder)
                        
                        self.chapters.push(Chapter {
                            title: file_name,
                            start_time: cumulative_time,
                            duration: estimated_duration,
                            is_locked: false,
                        });
                        
                        cumulative_time += estimated_duration;
                    }
                    
                    println!("[DEBUG] Mapped {} chapters from {} audio files", self.chapters.len(), self.audio_file_paths.len());
                }
            }
            Message::ChapterLookupCompleted(Ok(chapters)) => {
                self.is_looking_up_chapters = false;
                if !chapters.is_empty() {
                    self.chapters = chapters;
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
        let header = self.view_header();
        
        let content = match self.view_mode {
            ViewMode::Search => self.view_search(),
            ViewMode::Metadata => self.view_metadata(),
            ViewMode::Cover => self.view_cover(),
            ViewMode::Chapters => self.view_chapters(),
            ViewMode::Convert => self.view_convert(),
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
        event::listen_with(|event, _status| {
            if let event::Event::Window(_window_id, window::Event::FileDropped(paths)) = event {
                Some(Message::FileDropped(
                    paths.into_iter()
                        .map(|p| p.to_string_lossy().to_string())
                        .collect()
                ))
            } else {
                None // Ignore other events
            }
        })
    }
}

impl Lectern {
    fn view_header(&self) -> Element<Message> {
        container(
            row![
                text("🎵 Lectern")
                    .size(24)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.7, 0.4, 1.0))),
                Space::with_width(Length::Fill),
                text(if self.selected_book.is_some() {
                    "Editing book"
                } else {
                    "No book selected"
                })
                .size(14)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
            ]
            .align_items(Alignment::Center)
            .spacing(20),
        )
        .padding(15)
        .style(iced::theme::Container::Box)
        .into()
    }
    
    fn view_search(&self) -> Element<Message> {
        // Tab bar for navigation
        let tab_bar = self.view_tab_bar();
        
        // Search bar
        let search_bar = row![
            button("← Back")
                .on_press(if self.selected_book.is_some() {
                    Message::SwitchToMetadata
                } else {
                    Message::SwitchToMetadata
                })
                .padding(10),
            text_input("Search for book metadata...", &self.search_query)
                .on_input(Message::SearchQueryChanged)
                .on_submit(Message::PerformSearch)
                .width(Length::Fill)
                .padding(10),
            button(
                if self.is_searching {
                    "Searching..."
                } else {
                    "Search"
                }
            )
            .on_press(Message::PerformSearch)
            .padding(10),
        ]
        .spacing(10)
        .align_items(Alignment::Center);
        
        // Search results
        let results_content: Element<Message> = if self.is_searching {
            container(text("Searching...").size(18))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Box)
                .into()
        } else if let Some(ref error) = self.search_error {
            container(
                column![
                    text(format!("Error: {}", error)).size(16),
                    text("Check console for details").size(12)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(iced::theme::Container::Box)
            .into()
        } else if self.search_results.is_empty() && !self.search_query.is_empty() {
            container(
                column![
                    text("No results found").size(16),
                    text("Try a different search term or ASIN").size(12)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(iced::theme::Container::Box)
            .into()
        } else if self.search_results.is_empty() {
            container(text("Enter a book title, author, or ASIN to search").size(16))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Box)
                .into()
        } else {
            let mut results_column = Column::new();
            for (index, book) in self.search_results.iter().enumerate() {
                results_column = results_column.push(self.view_search_result(index, book));
            }
            scrollable(
                results_column
                    .spacing(10)
                    .padding(10),
            )
            .into()
        };
        
        container(
            column![
                tab_bar,
                Space::with_height(Length::Fixed(15.0)),
                search_bar,
                results_content,
            ]
            .spacing(15),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
    
    fn view_search_result(&self, index: usize, book: &BookMetadata) -> Element<Message> {
        container(
            row![
                // Cover placeholder
                container(
                    // Cover placeholder (image loading would go here)
                    text("📖")
                        .size(40)
                        .horizontal_alignment(iced::alignment::Horizontal::Center)
                )
                .width(60)
                .height(80)
                .style(iced::theme::Container::Box)
                .center_x()
                .center_y(),
                
                // Book info
                column![
                    text(&book.title)
                        .size(18)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(1.0, 1.0, 1.0))),
                    text(&book.author)
                        .size(14)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7))),
                    if let Some(ref narrator) = book.narrator {
                        text(format!("Narrated by: {}", narrator))
                            .size(12)
                            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6)))
                    } else {
                        text("")
                            .size(12)
                    },
                    if let Some(ref year) = book.publish_year {
                        text(year.clone())
                            .size(12)
                            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6)))
                    } else {
                        text("")
                            .size(12)
                    },
                ]
                .spacing(5)
                .width(Length::Fill),
                
                // "Use This" button
                button("Use This")
                    .on_press(Message::SelectBook(index))
                    .padding(10),
            ]
            .spacing(15)
            .align_items(Alignment::Center)
            .padding(15),
        )
        .style(iced::theme::Container::Box)
        .width(Length::Fill)
        .into()
    }
    
    fn view_metadata(&self) -> Element<Message> {
        // File selection area (shown when no book is selected or at the top)
        let file_selection_area = container(
            column![
                text("📁 Select Audiobook")
                    .size(20)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.7, 0.4, 1.0))),
                text("Drag and drop a folder or M4B file here, or click Browse")
                    .size(14)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                row![
                    button("Browse Files...")
                        .on_press(Message::BrowseFiles)
                        .padding(10),
                    button("Browse Folder...")
                        .on_press(Message::BrowseFolder)
                        .padding(10),
                    if let Some(ref path) = self.selected_file_path {
                        text(format!("Selected: {}", path))
                            .size(12)
                            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5)))
                    } else {
                        text("")
                    },
                ]
                .spacing(10)
                .align_items(Alignment::Center),
                if self.is_parsing_file {
                    text("Parsing file...")
                        .size(14)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7)))
                } else if let Some(ref error) = self.file_parse_error {
                    text(format!("Error: {}", error))
                        .size(14)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(1.0, 0.3, 0.3)))
                } else {
                    text("")
                },
            ]
            .spacing(15)
            .align_items(Alignment::Center)
            .padding(30),
        )
        .style(iced::theme::Container::Box)
        .width(Length::Fill);
        
        if self.selected_book.is_none() {
            return container(
                column![
                    file_selection_area,
                    Space::with_height(Length::Fixed(20.0)),
                    text("Or search for a book to get started")
                        .size(14)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
                    button("Search Books")
                        .on_press(Message::SwitchToSearch)
                        .padding(10),
                ]
                .spacing(20)
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into();
        }
        
        // Tab bar
        let tab_bar = self.view_tab_bar();
        
        // Metadata fields
        let fields = column![
            text("Book Metadata").size(22),
            text_input("Title", &self.editing_title)
                .on_input(Message::TitleChanged)
                .padding(10),
            text_input("Subtitle", &self.editing_subtitle)
                .on_input(Message::SubtitleChanged)
                .padding(10),
            text_input("Author(s)", &self.editing_author)
                .on_input(Message::AuthorChanged)
                .padding(10),
            row![
                text_input("Series", &self.editing_series)
                    .on_input(Message::SeriesChanged)
                    .padding(10)
                    .width(Length::FillPortion(3)),
                text_input("Series #", &self.editing_series_number)
                    .on_input(Message::SeriesNumberChanged)
                    .padding(10)
                    .width(Length::FillPortion(1)),
            ]
            .spacing(10),
            text_input("Narrator(s)", &self.editing_narrator)
                .on_input(Message::NarratorChanged)
                .padding(10),
            row![
                text_input("ISBN", &self.editing_isbn)
                    .on_input(Message::IsbnChanged)
                    .padding(10)
                    .width(Length::FillPortion(1)),
                text_input("ASIN", &self.selected_book.as_ref().map(|b| b.asin.clone().unwrap_or_default()).unwrap_or_default())
                    .padding(10)
                    .width(Length::FillPortion(1)),
            ]
            .spacing(10),
            row![
                text_input("Publisher", &self.editing_publisher)
                    .on_input(Message::PublisherChanged)
                    .padding(10)
                    .width(Length::FillPortion(1)),
                text_input("Publish Year", &self.editing_publish_year)
                    .on_input(Message::PublishYearChanged)
                    .padding(10)
                    .width(Length::FillPortion(1)),
            ]
            .spacing(10),
            row![
                text_input("Genre", &self.editing_genre)
                    .on_input(Message::GenreChanged)
                    .padding(10)
                    .width(Length::FillPortion(1)),
                text_input("Language", &self.editing_language)
                    .on_input(Message::LanguageChanged)
                    .padding(10)
                    .width(Length::FillPortion(1)),
            ]
            .spacing(10),
            text_input("Tags (comma-separated)", &self.editing_tags)
                .on_input(Message::TagsChanged)
                .padding(10),
            row![
                checkbox("Explicit", self.editing_explicit)
                    .on_toggle(Message::ExplicitToggled),
                checkbox("Abridged", self.editing_abridged)
                    .on_toggle(Message::AbridgedToggled),
            ]
            .spacing(20),
            text("Description").size(16),
            text_input("Description", &self.editing_description)
                .on_input(Message::DescriptionChanged)
                .padding(10)
                .size(16),
        ]
        .spacing(15);
        
        // Book is selected - hide file selection area, show tabs and fields
        container(
            column![
                tab_bar,
                scrollable(fields).height(Length::Fill),
            ]
            .spacing(15),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
    
    fn view_cover(&self) -> Element<Message> {
        // Tab bar - always visible when book is selected
        let tab_bar = self.view_tab_bar();
        
        if self.selected_book.is_none() {
            return container(
                column![
                    tab_bar,
                    Space::with_height(Length::Fixed(20.0)),
                    text("No book selected")
                        .size(18)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                    text("Select a book to manage its cover art")
                        .size(14)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
                ]
                .spacing(20)
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        }
        
        let cover_display: Element<Message> = if let Some(ref cover_path) = self.cover_image_path {
            // Try to load and display the image
            let is_url = cover_path.starts_with("http://") || cover_path.starts_with("https://");
            let image_handle = if is_url {
                // For URLs, we'd need to download first - for now show placeholder
                // In a real implementation, you'd download the image and create a Handle from bytes
                None
            } else {
                // For local files, try to load the image
                if Path::new(cover_path).exists() {
                    if let Ok(img_data) = std::fs::read(cover_path) {
                        match ::image::load_from_memory(&img_data) {
                            Ok(img) => {
                                // Convert to RGBA
                                let rgba = img.to_rgba8();
                                let (width, height) = rgba.dimensions();
                                let pixels: Vec<u8> = rgba.into_raw();
                                Some(Handle::from_pixels(width, height, pixels))
                            },
                            Err(_) => None,
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            
            if let Some(handle) = image_handle {
                // Display actual image
                container(
                    column![
                        text("Cover Image")
                            .size(16),
                        image(handle)
                            .width(Length::Fixed(300.0))
                            .height(Length::Fixed(400.0)),
                        text(if is_url { "URL" } else { "Local file" })
                            .size(12)
                            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                )
                .width(Length::Fill)
                .into()
            } else {
                // Fallback: show placeholder with URL info
                let display_text = if is_url {
                    format!("🖼️\nCover Image\n\nURL:\n{}", 
                        if cover_path.len() > 50 {
                            format!("{}...", &cover_path[..50])
                        } else {
                            cover_path.clone()
                        })
                } else {
                    format!("🖼️\nCover Image\n\nFile:\n{}", 
                        if cover_path.len() > 50 {
                            format!("{}...", &cover_path[..50])
                        } else {
                            cover_path.clone()
                        })
                };
                
                container(
                    column![
                        text("Cover Image")
                            .size(16),
                        container(
                            text(&display_text)
                                .size(12)
                                .horizontal_alignment(iced::alignment::Horizontal::Center)
                        )
                        .width(Length::Fixed(300.0))
                        .height(Length::Fixed(400.0))
                        .style(iced::theme::Container::Box)
                        .center_x()
                        .center_y(),
                        text(if is_url { "URL provided (image loading not yet implemented)" } else { "Local file (could not load image)" })
                            .size(12)
                            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                )
                .width(Length::Fill)
                .into()
            }
        } else {
            container(
                column![
                    text("No cover image")
                        .size(16)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                    container(
                        text("📖\nNo Cover")
                            .size(14)
                            .horizontal_alignment(iced::alignment::Horizontal::Center)
                    )
                    .width(Length::Fixed(300.0))
                    .height(Length::Fixed(400.0))
                    .style(iced::theme::Container::Box)
                    .center_x()
                    .center_y(),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .into()
        };
        
        // Cover search section
        let cover_search_section = column![
            text("Search for Cover Art").size(18),
            row![
                button("Search Cover")
                    .on_press(Message::SearchCover)
                    .padding(10),
                if self.is_searching_cover {
                    text("Searching...")
                        .size(14)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7)))
                } else {
                    text("")
                },
            ]
            .spacing(10)
            .align_items(Alignment::Center),
            if let Some(ref error) = self.cover_search_error {
                text(format!("Error: {}", error))
                    .size(14)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(1.0, 0.3, 0.3)))
            } else {
                text("")
            },
        ]
        .spacing(10);
        
        // Cover search results
        let cover_results: Element<Message> = if !self.cover_search_results.is_empty() {
            let mut results_column = Column::new();
            for (index, cover) in self.cover_search_results.iter().enumerate() {
                results_column = results_column.push(
                    container(
                        row![
                            container(
                                text("🖼️")
                                    .size(24)
                                    .horizontal_alignment(iced::alignment::Horizontal::Center)
                            )
                            .width(Length::Fixed(80.0))
                            .height(Length::Fixed(120.0))
                            .style(iced::theme::Container::Box)
                            .center_x()
                            .center_y(),
                            column![
                                text(&cover.source)
                                    .size(14)
                                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7))),
                                text(format!("{}x{}", cover.width, cover.height))
                                    .size(12)
                                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                            ]
                            .spacing(5)
                            .width(Length::Fill),
                            button("Use This")
                                .on_press(Message::SelectCover(index))
                                .padding(10),
                        ]
                        .spacing(15)
                        .align_items(Alignment::Center)
                        .padding(10),
                    )
                    .style(iced::theme::Container::Box)
                    .width(Length::Fill)
                );
            }
            scrollable(
                results_column
                    .spacing(10)
                    .padding(10),
            )
            .height(Length::Fill)
            .into()
        } else {
            Space::with_height(Length::Shrink).into()
        };
        
        container(
            scrollable(
                column![
                    tab_bar,
                    Space::with_height(Length::Fixed(15.0)),
                    row![
                        cover_display,
                        column![
                            text("Cover Options").size(18),
                            button("Browse Image File...")
                                .on_press(Message::BrowseCoverImage)
                                .padding(10),
                            text("Or enter URL:").size(14),
                            text_input("Cover Image URL", 
                                self.cover_image_path.as_deref().unwrap_or(""))
                                .on_input(Message::CoverUrlChanged)
                                .padding(10),
                            Space::with_height(Length::Fixed(20.0)),
                            cover_search_section,
                        ]
                        .spacing(15)
                        .width(Length::FillPortion(1)),
                    ]
                    .spacing(20),
                    if !self.cover_search_results.is_empty() {
                        text(format!("Found {} cover results:", self.cover_search_results.len()))
                            .size(16)
                    } else {
                        text("")
                    },
                    cover_results,
                ]
                .spacing(20)
                .padding(20),
            )
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
    
    fn view_chapters(&self) -> Element<Message> {
        let tab_bar = self.view_tab_bar();
        
        // ASIN input for chapter lookup
        let asin_input_section = column![
            text("Enter ASIN to fetch chapters:").size(14),
            row![
                text_input("ASIN (e.g., B002V02KPU)", &self.chapter_asin_input)
                    .on_input(Message::ChapterAsinChanged)
                    .padding(10)
                    .width(Length::FillPortion(2)),
                text(format!("Current book ASIN: {}", 
                    self.selected_book.as_ref()
                        .and_then(|b| b.asin.as_ref())
                        .map(|a| a.as_str())
                        .unwrap_or("None")))
                    .size(12)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6)))
                    .width(Length::FillPortion(1)),
            ]
            .spacing(10),
        ]
        .spacing(5);
        
        // Top controls
        let mut controls_row = row![
            button("Remove All")
                .on_press(Message::ChapterRemoveAll)
                .padding(10),
            button("Shift Times")
                .on_press(Message::ChapterShiftTimes(0)) // Will need a dialog for amount
                .padding(10),
            button("Lookup")
                .on_press(Message::ChapterLookup)
                .padding(10),
        ];
        
        // Add map files button if audio files are available
        if !self.audio_file_paths.is_empty() {
            let count = self.audio_file_paths.len();
            let btn_label = if count == 1 {
                "Map from 1 File"
            } else {
                // Use a static string for common cases
                match count {
                    2..=9 => "Map from Files", // Will show count in tooltip
                    _ => "Map from Files",
                }
            };
            controls_row = controls_row.push(
                button(btn_label)
                    .on_press(Message::MapChaptersFromFiles)
                    .padding(10)
            );
        }
        
        let top_controls = controls_row
            .push(Space::with_width(Length::Fill))
            .push(checkbox("Show seconds", self.chapters_show_seconds)
                .on_toggle(Message::ChaptersShowSecondsToggled)
                .text_size(14))
            .spacing(10)
            .align_items(Alignment::Center);
        
        // Chapter list header
        let header = row![
            text("#").width(Length::Fixed(50.0)),
            text("START").width(Length::Fixed(150.0)),
            text("TITLE").width(Length::Fill),
            checkbox("", self.chapters_global_locked)
                .on_toggle(|_| Message::ChaptersGlobalLockToggled)
                .width(Length::Fixed(30.0)),
            text("Actions").width(Length::Fixed(200.0)),
        ]
        .spacing(10)
        .padding(10);
        
        // Chapter list
        let chapter_list_content: Element<Message> = if self.chapters.is_empty() {
            column![
                Space::with_height(Length::Fixed(50.0)),
                text("No chapters yet. Use 'Lookup' to fetch chapters from a provider, or add them manually.")
                    .size(14)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
            ]
            .width(Length::Fill)
            .into()
        } else {
            let mut chapter_list = Column::new();
            for (index, chapter) in self.chapters.iter().enumerate() {
                let time_str = format_time(chapter.start_time, self.chapters_show_seconds);
                let chapter_index = index;
                let is_locked = chapter.is_locked;
                
                let time_input = text_input("HH:MM:SS", &time_str)
                    .width(Length::Fixed(120.0))
                    .on_input(move |s| Message::ChapterTimeChanged(chapter_index, s));
                
                let title_input = text_input("Chapter title", &chapter.title)
                    .width(Length::Fill)
                    .on_input(move |s| Message::ChapterTitleChanged(chapter_index, s));
                
                chapter_list = chapter_list.push(
                    row![
                        text(format!("#{}", index + 1))
                            .width(Length::Fixed(50.0))
                            .size(14),
                        row![
                            button("-")
                                .on_press(Message::ChapterTimeAdjusted(chapter_index, -1))
                                .width(Length::Fixed(30.0))
                                .padding(5),
                            time_input,
                            button("+")
                                .on_press(Message::ChapterTimeAdjusted(chapter_index, 1))
                                .width(Length::Fixed(30.0))
                                .padding(5),
                        ]
                        .spacing(5)
                        .width(Length::Fixed(150.0)),
                        title_input,
                        checkbox("", is_locked)
                            .on_toggle(move |_| Message::ChapterLockToggled(chapter_index))
                            .width(Length::Fixed(30.0)),
                        row![
                            button("🔒")
                                .on_press(Message::ChapterLockToggled(chapter_index))
                                .width(Length::Fixed(30.0))
                                .padding(5),
                            button("🗑")
                                .on_press(Message::ChapterDelete(chapter_index))
                                .width(Length::Fixed(30.0))
                                .padding(5),
                            button("+")
                                .on_press(Message::ChapterInsertBelow(chapter_index))
                                .width(Length::Fixed(30.0))
                                .padding(5),
                            button("▶")
                                .on_press(Message::ChapterDelete(chapter_index)) // TODO: Play functionality
                                .width(Length::Fixed(30.0))
                                .padding(5),
                        ]
                        .spacing(5)
                        .width(Length::Fixed(200.0)),
                    ]
                    .spacing(10)
                    .padding(5)
                );
            }
            scrollable(chapter_list.spacing(5)).into()
        };
        
        // Status messages
        let status = if self.is_looking_up_chapters {
            text("Looking up chapters...").size(14)
        } else if let Some(ref error) = self.chapter_lookup_error {
            text(format!("Error: {}", error)).size(14)
        } else {
            text(format!("{} chapters", self.chapters.len())).size(14)
        };
        
        container(
            column![
                tab_bar,
                Space::with_height(Length::Fixed(15.0)),
                asin_input_section,
                Space::with_height(Length::Fixed(10.0)),
                top_controls,
                Space::with_height(Length::Fixed(10.0)),
                header,
                chapter_list_content,
                Space::with_height(Length::Fixed(10.0)),
                status,
            ]
            .spacing(10)
            .padding(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
    
    fn view_convert(&self) -> Element<Message> {
        let tab_bar = self.view_tab_bar();
        
        container(
            column![
                tab_bar,
                Space::with_height(Length::Fixed(20.0)),
                text("Convert tab - Coming soon")
                    .size(18)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
            ]
            .spacing(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
    
    // Helper function to create the tab bar (used in all views)
    fn view_tab_bar(&self) -> Element<Message> {
        row![
            button("📁 Metadata")
                .style(if self.view_mode == ViewMode::Metadata {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                })
                .on_press(Message::SwitchToMetadata),
            button("🖼️ Cover")
                .style(if self.view_mode == ViewMode::Cover {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                })
                .on_press(Message::SwitchToCover),
            button("📑 Chapters")
                .style(if self.view_mode == ViewMode::Chapters {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                })
                .on_press(Message::SwitchToChapters),
            button("🔄 Convert")
                .style(if self.view_mode == ViewMode::Convert {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                })
                .on_press(Message::SwitchToConvert),
            Space::with_width(Length::Fill),
            button("🔍 Search")
                .on_press(Message::SwitchToSearch),
        ]
        .spacing(10)
        .into()
    }
}

// Function to parse an audiobook file or directory
// Note: This is synchronous now since we create the runtime in the Command
// Helper function to format time in milliseconds to HH:MM:SS or HH:MM:SS.mmm
fn format_time(milliseconds: u64, show_seconds: bool) -> String {
    let total_seconds = milliseconds / 1000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    
    if show_seconds {
        let ms = milliseconds % 1000;
        format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, ms)
    } else {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

// Helper function to parse time string (HH:MM:SS or HH:MM:SS.mmm) to seconds
fn parse_time_string(time_str: &str) -> Result<u64, String> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 3 {
        return Err("Invalid time format. Use HH:MM:SS".to_string());
    }
    
    let hours: u64 = parts[0].parse().map_err(|_| "Invalid hours")?;
    let minutes: u64 = parts[1].parse().map_err(|_| "Invalid minutes")?;
    let seconds_part = parts[2];
    
    let (seconds, milliseconds) = if seconds_part.contains('.') {
        let sec_parts: Vec<&str> = seconds_part.split('.').collect();
        let secs: u64 = sec_parts[0].parse().map_err(|_| "Invalid seconds")?;
        let ms: u64 = sec_parts.get(1)
            .unwrap_or(&"0")
            .parse()
            .unwrap_or(0);
        (secs, ms)
    } else {
        (seconds_part.parse().map_err(|_| "Invalid seconds")?, 0)
    };
    
    Ok(hours * 3600 + minutes * 60 + seconds + (milliseconds / 1000))
}

fn parse_audiobook_file(path_str: &str) -> Result<BookMetadata, String> {
    let path = Path::new(path_str);
    
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path_str));
    }
    
    if path.is_file() {
        // Single file (M4B, MP3, etc.)
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        match extension.as_str() {
            "m4b" | "m4a" => {
                // Use existing get_file_metadata if available, or create basic metadata
                Ok(BookMetadata {
                    title: path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                    subtitle: None,
                    author: "Unknown Author".to_string(),
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
                })
            }
            _ => {
                Err(format!("Unsupported file type: {}", extension))
            }
        }
    } else if path.is_dir() {
        // Directory of audio files (MP3, etc.)
        // Check if directory contains audio files
        let mut audio_files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if matches!(ext_lower.as_str(), "mp3" | "aac" | "wav" | "flac" | "m4b" | "m4a") {
                        audio_files.push(entry.path());
                    }
                }
            }
        }
        
        if audio_files.is_empty() {
            return Err("Directory does not contain any audio files".to_string());
        }
        
        // Sort files naturally
        audio_files.sort();
        
        Ok(BookMetadata {
            title: path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string(),
            subtitle: None,
            author: "Unknown Author".to_string(),
            isbn: None,
            asin: None,
            description: Some(format!("Directory with {} audio files", audio_files.len())),
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
        })
    } else {
        Err("Path is neither a file nor a directory".to_string())
    }
}

// Helper function to get audio files from a directory
fn get_audio_files_from_directory(dir_path: &str) -> Vec<String> {
    let mut audio_files = Vec::new();
    let path = Path::new(dir_path);
    
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if matches!(ext_lower.as_str(), "mp3" | "aac" | "wav" | "flac" | "m4b" | "m4a") {
                    if let Some(path_str) = entry.path().to_str() {
                        audio_files.push(path_str.to_string());
                    }
                }
            }
        }
    }
    
    // Sort files naturally (alphabetically)
    audio_files.sort();
    audio_files
}

// Search for cover art from various providers
async fn search_cover_art(
    title: &str,
    author: &str,
    isbn: Option<&str>,
    asin: Option<&str>,
) -> Result<Vec<CoverResult>, String> {
    let mut results = Vec::new();
    
    // Try Open Library first (has good cover art)
    if !title.is_empty() {
        let query = format!("{} {}", title, author);
        if let Ok(covers) = search_open_library_covers(&query).await {
            results.extend(covers);
        }
    }
    
    // Try Google Books
    if !title.is_empty() {
        let query = format!("{} {}", title, author);
        if let Ok(covers) = search_google_books_covers(&query).await {
            results.extend(covers);
        }
    }
    
    // Try using ASIN/ISBN for more specific results
    if let Some(asin_val) = asin {
        if let Ok(covers) = search_audnexus_cover(asin_val).await {
            results.extend(covers);
        }
    }
    
    if results.is_empty() {
        Err("No cover art found".to_string())
    } else {
        Ok(results)
    }
}

async fn search_open_library_covers(query: &str) -> Result<Vec<CoverResult>, String> {
    let client = reqwest::Client::new();
    let url = format!("https://openlibrary.org/search.json?q={}&limit=5", 
                     urlencoding::encode(query));
    
    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("Open Library request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Open Library returned status: {}", response.status()));
    }
    
    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    let mut covers = Vec::new();
    if let Some(docs) = json.get("docs").and_then(|d| d.as_array()) {
        for doc in docs.iter().take(5) {
            if let Some(cover_id) = doc.get("cover_i").and_then(|c| c.as_i64()) {
                let cover_url = format!("https://covers.openlibrary.org/b/id/{}-L.jpg", cover_id);
                covers.push(CoverResult {
                    url: cover_url,
                    width: 500,
                    height: 500,
                    source: "Open Library".to_string(),
                });
            }
        }
    }
    
    Ok(covers)
}

async fn search_google_books_covers(query: &str) -> Result<Vec<CoverResult>, String> {
    let client = reqwest::Client::new();
    let url = format!("https://www.googleapis.com/books/v1/volumes?q={}&maxResults=5", 
                     urlencoding::encode(query));
    
    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("Google Books request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Google Books returned status: {}", response.status()));
    }
    
    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    let mut covers = Vec::new();
    if let Some(items) = json.get("items").and_then(|i| i.as_array()) {
        for item in items.iter().take(5) {
            if let Some(volume_info) = item.get("volumeInfo") {
                if let Some(image_links) = volume_info.get("imageLinks") {
                    if let Some(thumbnail) = image_links.get("thumbnail").and_then(|t| t.as_str()) {
                        // Replace thumbnail size with large size
                        let large_url = thumbnail.replace("zoom=1", "zoom=5").replace("&edge=curl", "");
                        covers.push(CoverResult {
                            url: large_url.to_string(),
                            width: 1280,
                            height: 1280,
                            source: "Google Books".to_string(),
                        });
                    }
                }
            }
        }
    }
    
    Ok(covers)
}

async fn search_audnexus_cover(asin: &str) -> Result<Vec<CoverResult>, String> {
    let client = reqwest::Client::new();
    let url = format!("https://api.audnex.us/books/{}", asin);
    
    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("Audnexus request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Audnexus returned status: {}", response.status()));
    }
    
    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    if let Some(image_url) = json.get("image").and_then(|i| i.as_str()) {
        Ok(vec![CoverResult {
            url: image_url.to_string(),
            width: 500,
            height: 500,
            source: "Audnexus".to_string(),
        }])
    } else {
        Ok(Vec::new())
    }
}
