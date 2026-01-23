#![allow(dead_code)]
#![allow(unused_variables)]

mod services;

use services::{AudioService, BookMetadata, ABSConfig, SearchProvider};

use qmetaobject::*;
use std::cell::RefCell;
use std::path::PathBuf;
use url::Url;

#[derive(QObject, Default)]
pub struct LecternController {
    base: qt_base_class!(trait QObject),

    // Properties
    current_folder: qt_property!(QString; NOTIFY folder_changed),
    status_message: qt_property!(QString; NOTIFY status_changed),
    progress_value: qt_property!(f64; NOTIFY progress_changed),
    is_processing: qt_property!(bool; NOTIFY processing_changed),

    // Metadata properties
    metadata_title: qt_property!(QString; NOTIFY metadata_changed),
    metadata_author: qt_property!(QString; NOTIFY metadata_changed),
    metadata_series: qt_property!(QString; NOTIFY metadata_changed),
    metadata_narrator: qt_property!(QString; NOTIFY metadata_changed),
    metadata_cover_url: qt_property!(QString; NOTIFY metadata_changed),

    // Search result properties
    search_title: qt_property!(QString; NOTIFY search_result_changed),
    search_author: qt_property!(QString; NOTIFY search_result_changed),
    search_cover_url: qt_property!(QString; NOTIFY search_result_changed),
    search_result_changed: qt_signal!(),

    // Cover search result properties
    cover_search_urls: qt_property!(QStringList; NOTIFY cover_search_changed),

    // Chapter properties
    chapters: qt_property!(QVariantList; NOTIFY chapters_changed),

    // Search provider
    search_provider: qt_property!(QString; NOTIFY provider_changed),

    // ABS settings
    abs_host: qt_property!(QString; NOTIFY config_changed),
    abs_token: qt_property!(QString; NOTIFY config_changed),
    abs_library_id: qt_property!(QString; NOTIFY config_changed),

    // Local Library settings
    local_library_path: qt_property!(QString; NOTIFY config_changed),
    path_template: qt_property!(QString; NOTIFY config_changed),

    config_changed: qt_signal!(),

    // Signals
    folder_changed: qt_signal!(),
    status_changed: qt_signal!(),
    progress_changed: qt_signal!(),
    processing_changed: qt_signal!(),
    metadata_changed: qt_signal!(),
    provider_changed: qt_signal!(),
    cover_search_changed: qt_signal!(),
    chapters_changed: qt_signal!(),
    log_message: qt_signal!(message: QString),
    conversion_completed: qt_signal!(),
    error_occurred: qt_signal!(message: QString),

    // Qt Methods (callable from QML)
    save_config: qt_method!(fn(&mut self, url: QString, token: QString, id: QString, local_path: QString, template: QString)),
    set_folder_path: qt_method!(fn(&mut self, url_string: QString)),
    search_metadata: qt_method!(fn(&mut self, query: QString, by_asin: bool)),
    search_metadata_with_provider: qt_method!(fn(&mut self, query: QString, provider: QString)),
    search_cover_art: qt_method!(fn(&mut self, query: QString, provider: QString)),
    auto_search_metadata: qt_method!(fn(&mut self, query: QString)),
    start_conversion: qt_method!(fn(&mut self)),
    cancel_conversion: qt_method!(fn(&mut self)),
    scan_chapters: qt_method!(fn(&mut self)),
}

impl LecternController {
    fn initialize(&mut self) {
        self.load_config();

        // Set default path template if not configured
        if self.path_template.to_string().is_empty() {
            self.path_template = QString::from("{Path to Local Library}/{Author}/{Title}.m4b");
        }
    }

    fn load_config(&mut self) {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lectern")
            .join("config.json");

        if let Ok(content) = std::fs::read_to_string(config_path) {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
        if let Some(host) = config.get("abs_host").and_then(|v| v.as_str()) {
            self.abs_host = QString::from(host);
        }
        if let Some(token) = config.get("abs_token").and_then(|v| v.as_str()) {
            self.abs_token = QString::from(token);
        }
        if let Some(library_id) = config.get("abs_library_id").and_then(|v| v.as_str()) {
            self.abs_library_id = QString::from(library_id);
        }
        if let Some(local_path) = config.get("local_library_path").and_then(|v| v.as_str()) {
            self.local_library_path = QString::from(local_path);
        }
        if let Some(template) = config.get("path_template").and_then(|v| v.as_str()) {
            self.path_template = QString::from(template);
        }
            }
        }

        self.status_message = QString::from("Ready to process audiobooks");
        self.status_changed();
    }

    fn save_config(&mut self, url: QString, token: QString, id: QString, local_path: QString, template: QString) {
        self.abs_host = url;
        self.abs_token = token;
        self.abs_library_id = id;
        self.local_library_path = local_path;
        self.path_template = template;
        self.config_changed();

        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lectern")
            .join("config.json");

        if let Some(parent) = config_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let config = serde_json::json!({
            "abs_host": self.abs_host.to_string(),
            "abs_token": self.abs_token.to_string(),
            "abs_library_id": self.abs_library_id.to_string(),
            "local_library_path": self.local_library_path.to_string(),
            "path_template": self.path_template.to_string(),
        });

        if let Ok(json) = serde_json::to_string_pretty(&config) {
            if let Err(e) = std::fs::write(&config_path, json) {
                eprintln!("Failed to save config: {}", e);
            } else {
                println!("⚙️ Settings saved");
            }
        }
    }

    fn set_folder_path(&mut self, url_string: QString) {
        let raw_url = url_string.to_string();

        let path = if let Ok(parsed_url) = Url::parse(&raw_url) {
            if parsed_url.scheme() == "file" {
                parsed_url.to_file_path().unwrap_or_else(|_| PathBuf::from(&raw_url))
            } else {
                PathBuf::from(&raw_url)
            }
        } else {
            PathBuf::from(&raw_url)
        };

        let path_str = path.to_string_lossy().to_string();
        self.current_folder = QString::from(path_str.clone());
        self.folder_changed();

        // Try to automatically search for metadata based on folder name
        if let Some(search_term) = AudioService::parse_folder_name_for_search(&path_str) {
            println!("🔍 Auto-searching for metadata: '{}'", search_term);
            self.auto_search_metadata(QString::from(search_term));
        } else {
            self.status_message = QString::from(format!("Loaded folder: {}", path_str));
            self.status_changed();
        }

        println!("📂 Folder set to: {:?}", path);
    }

    fn search_metadata(&mut self, query: QString, _by_asin: bool) {
        // Use the default provider (try all providers)
        self.search_metadata_with_provider_impl(query, None);
    }

    fn search_metadata_with_provider(&mut self, query: QString, provider: QString) {
        let provider_enum = match provider.to_string().as_str() {
            "audnexus" => Some(SearchProvider::Audnexus),
            "google" => Some(SearchProvider::GoogleBooks),
            "openlibrary" => Some(SearchProvider::OpenLibrary),
            "itunes" => Some(SearchProvider::ITunes),
            "fantlab" => Some(SearchProvider::FantLab),
            _ => None,
        };

        self.search_metadata_with_provider_impl(query, provider_enum);
    }

    fn search_metadata_with_provider_impl(&mut self, query: QString, provider: Option<SearchProvider>) {
        let qptr = QPointer::from(&*self);
        let query_str = query.to_string();
        let provider_name = provider.as_ref()
            .map(|p| AudioService::provider_name(p))
            .unwrap_or("All providers");

        self.is_processing = true;
        self.status_message = QString::from(format!("Searching '{}' with {}...", query_str, provider_name));
        self.status_changed();
        self.processing_changed();

        let on_complete = queued_callback(move |results: Vec<BookMetadata>| {
            if let Some(pinned) = qptr.as_pinned() {
                let mut s = pinned.borrow_mut();
                s.is_processing = false;

                if let Some(book) = results.first() {
                    s.search_title = book.title.clone().into();
                    let author_str = book.authors.join(", ");
                    s.search_author = author_str.into();
                    s.search_cover_url = book.image_url.clone().into();
                    s.status_message = QString::from("✓ Metadata found");
                } else {
                    s.search_title = QString::from("");
                    s.search_author = QString::from("");
                    s.search_cover_url = QString::from("");
                    s.status_message = QString::from("❌ No metadata found");
                }

                s.search_result_changed();
                s.status_changed();
                s.processing_changed();
            }
        });

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let result = if let Some(provider) = provider {
                    AudioService::fetch_metadata_with_provider(&query_str, provider).await
                } else {
                    AudioService::fetch_metadata(&query_str).await
                };

                match result {
                    Ok(metadata) => on_complete(vec![metadata]),
                    Err(_) => on_complete(vec![]),
                }
            });
        });
    }

    fn search_cover_art(&mut self, query: QString, provider: QString) {
        let qptr = QPointer::from(&*self);
        let query_str = query.to_string();

        let provider_enum = match provider.to_string().as_str() {
            "audnexus" => SearchProvider::Audnexus,
            "google" => SearchProvider::GoogleBooks,
            "openlibrary" => SearchProvider::OpenLibrary,
            _ => SearchProvider::GoogleBooks, // default
        };

        self.is_processing = true;
        self.status_message = QString::from(format!("Searching covers with {}...", AudioService::provider_name(&provider_enum)));
        self.status_changed();
        self.processing_changed();

        let on_complete = queued_callback(move |urls: Vec<String>| {
            if let Some(pinned) = qptr.as_pinned() {
                let mut s = pinned.borrow_mut();
                s.is_processing = false;
                s.processing_changed();

                let url_count = urls.len();
                let qstring_list: Vec<QString> = urls.into_iter().map(|url| url.into()).collect();
                s.cover_search_urls = qstring_list.into();
                s.cover_search_changed();

                s.status_message = QString::from(format!("Found {} cover images", url_count));
                s.status_changed();
            }
        });

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                match AudioService::search_cover_art(&query_str, provider_enum).await {
                    Ok(urls) => on_complete(urls),
                    Err(e) => {
                        println!("Cover search failed: {}", e);
                        on_complete(vec![]);
                    }
                }
            });
        });
    }

    fn auto_search_metadata(&mut self, query: QString) {
        let qptr = QPointer::from(&*self);
        let query_str = query.to_string();

        self.is_processing = true;
        self.status_message = QString::from("Auto-searching for metadata...");
        self.status_changed();
        self.processing_changed();

        let on_complete = queued_callback(move |results: Vec<BookMetadata>| {
            if let Some(pinned) = qptr.as_pinned() {
                let mut s = pinned.borrow_mut();
                s.is_processing = false;

                if let Some(book) = results.first() {
                    s.search_title = book.title.clone().into();
                    let author_str = book.authors.join(", ");
                    s.search_author = author_str.into();
                    s.search_cover_url = book.image_url.clone().into();
                    s.search_result_changed();
                    s.status_message = QString::from("✓ Metadata found automatically");
                } else {
                    s.search_title = QString::from("");
                    s.search_author = QString::from("");
                    s.search_cover_url = QString::from("");
                    s.search_result_changed();
                    s.status_message = QString::from("No automatic metadata found - use manual search");
                }

                s.status_changed();
                s.processing_changed();
            }
        });

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                match AudioService::fetch_metadata(&query_str).await {
                    Ok(metadata) => on_complete(vec![metadata]),
                    Err(e) => {
                        println!("Auto-search failed: {}", e);
                        on_complete(vec![]);
                    }
                }
            });
        });
    }

    fn scan_chapters(&mut self) {
        let folder = self.current_folder.to_string();

        if folder.is_empty() {
            self.status_message = QString::from("No folder selected");
            self.status_changed();
            return;
        }

        self.is_processing = true;
        self.status_message = QString::from("Scanning for chapters...");
        self.status_changed();
        self.processing_changed();

        let qptr = QPointer::from(&*self);
        let on_complete = queued_callback(move |chapters: Vec<services::Chapter>| {
            if let Some(pinned) = qptr.as_pinned() {
                let mut s = pinned.borrow_mut();
                s.is_processing = false;
                s.processing_changed();

                // Convert chapters to QVariantList for QML
                let mut qvariant_list = QVariantList::default();
                for chapter in chapters {
                    let mut chapter_map = QVariantMap::default();
                    chapter_map.insert("title".into(), QString::from(chapter.title).into());
                    chapter_map.insert("start_time".into(), chapter.start_time.into());
                    chapter_map.insert("end_time".into(), chapter.end_time.unwrap_or(0.0).into());
                    chapter_map.insert("locked".into(), chapter.locked.into());
                    qvariant_list.push(chapter_map.into());
                }

                s.chapters = qvariant_list;
                s.chapters_changed();

                s.status_message = QString::from(format!("Found {} chapters", s.chapters.len()));
                s.status_changed();
            }
        });

        std::thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let result = runtime.block_on(async {
                match AudioService::auto_detect_chapters(&folder).await {
                    Ok(mut chapters) => {
                        // Calculate timestamps based on file durations
                        if let Err(e) = AudioService::calculate_chapter_timestamps(&folder, &mut chapters).await {
                            println!("Warning: Failed to calculate timestamps: {}", e);
                        }
                        chapters
                    }
                    Err(e) => {
                        println!("Chapter scan failed: {}", e);
                        vec![]
                    }
                }
            });

            on_complete(result);
        });
    }

    fn start_conversion(&mut self) {
        let folder = self.current_folder.to_string();
        let metadata = BookMetadata {
            title: self.metadata_title.to_string(),
            authors: vec![self.metadata_author.to_string()],
            narrator_names: if self.metadata_narrator.to_string().is_empty() {
                None
            } else {
                Some(vec![self.metadata_narrator.to_string()])
            },
            series_name: if self.metadata_series.to_string().is_empty() {
                None
            } else {
                Some(self.metadata_series.to_string())
            },
            image_url: self.metadata_cover_url.to_string(),
            asin: "".to_string(),
            duration_minutes: None,
            release_date: None,
        };

        let abs_config = ABSConfig {
            host: self.abs_host.to_string(),
            token: self.abs_token.to_string(),
            library_id: self.abs_library_id.to_string(),
        };

        self.is_processing = true;
        self.status_message = QString::from("Starting audio conversion...");
        self.progress_value = 0.0;
        self.processing_changed();
        self.status_changed();
        self.progress_changed();

        let qptr = QPointer::from(&*self);
        let update_progress = queued_callback(move |update: Result<String, String>| {
            if let Some(pinned) = qptr.as_pinned() {
                let mut controller = pinned.borrow_mut();
                match update {
                    Ok(message) => {
                        controller.status_message = QString::from(message.as_str());
                        controller.is_processing = false;
                        controller.processing_changed();
                        controller.status_changed();
                        
                        if message.contains("completed") {
                            controller.conversion_completed();
                        }
                    }
                    Err(error) => {
                        controller.status_message = QString::from(error.as_str());
                        controller.error_occurred(QString::from("Conversion failed"));
                        controller.status_changed();
                        controller.is_processing = false;
                        controller.processing_changed();
                    }
                }
            }
        });

        let local_library_path = self.local_library_path.to_string();
        let path_template = self.path_template.to_string();

        std::thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let result = runtime.block_on(async {
                let input_dir = PathBuf::from(folder);
                if !input_dir.exists() {
                    return Err("Input directory does not exist".to_string());
                }

                // Determine output path
                let output_path = if !local_library_path.is_empty() && !path_template.is_empty() {
                    // Use Local Library path templating
                    match AudioService::generate_local_library_path(&local_library_path, &path_template, &metadata) {
                        Ok(templated_path) => {
                            println!("Using Local Library path: {}", templated_path);
                            PathBuf::from(templated_path)
                        }
                        Err(e) => {
                            println!("Path template error: {}, falling back to default", e);
                            // Fallback to default path
                            input_dir
                                .parent()
                                .unwrap_or(&input_dir)
                                .join(format!("{}.m4b", metadata.title.replace(":", "").replace("/", "")))
                        }
                    }
                } else {
                    // Default behavior: save next to input folder
                    input_dir
                        .parent()
                        .unwrap_or(&input_dir)
                        .join(format!("{}.m4b", metadata.title.replace(":", "").replace("/", "")))
                };

                // Ensure output directory exists
                if let Some(parent) = output_path.parent() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        return Err(format!("Failed to create output directory: {}", e));
                    }
                }

                AudioService::convert_to_m4b_with_chapters(&input_dir, &output_path.to_string_lossy(), &metadata).await?;
                AudioService::apply_tags(&output_path.to_string_lossy(), &metadata).await?;

                if !abs_config.host.is_empty() && !abs_config.token.is_empty() {
                    match AudioService::upload_and_scan(&output_path.to_string_lossy(), &abs_config).await {
                        Ok(_) => Ok("✓ Conversion and upload completed!".to_string()),
                        Err(e) => {
                            println!("Upload error: {}", e);
                            Ok("✓ Conversion completed, upload failed".to_string())
                        }
                    }
                } else {
                    Ok("✓ Conversion completed!".to_string())
                }
            });

            update_progress(result);
        });
    }

    fn cancel_conversion(&mut self) {
        self.is_processing = false;
        self.processing_changed();
        self.status_message = QString::from("Operation cancelled");
        self.status_changed();
    }
}

fn main() {
    println!("🎵 Starting Lectern...");

    // Set Qt platform if not set
    if std::env::var("QT_QPA_PLATFORM").is_err() {
        std::env::set_var("QT_QPA_PLATFORM", "xcb");
    }

    // Create controller
    let mut controller = LecternController::default();
    controller.initialize();
    let controller = RefCell::new(controller);
    let controller_pinned = unsafe { QObjectPinned::new(&controller) };

    // Create QML engine
    let mut engine = QmlEngine::new();
    engine.set_object_property("controller".into(), controller_pinned);

    // Load QML - try absolute path first
    let qml_path = std::path::Path::new("qml/main.qml");
    if qml_path.exists() {
        println!("Loading QML from: {:?}", qml_path.canonicalize().unwrap_or(qml_path.to_path_buf()));
        engine.load_file("qml/main.qml".into());
    } else {
        eprintln!("❌ ERROR: qml/main.qml not found!");
        eprintln!("Current directory: {:?}", std::env::current_dir());
        std::process::exit(1);
    }

    println!("Starting Qt event loop...");
    engine.exec();
    println!("Qt event loop ended");
}