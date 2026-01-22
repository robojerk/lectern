#![allow(dead_code)]
#![allow(unused_variables)]

mod services;

use services::{AudioService, BookMetadata, ABSConfig};

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

    // ABS settings
    abs_host: qt_property!(QString; NOTIFY config_changed),
    abs_token: qt_property!(QString; NOTIFY config_changed),
    abs_library_id: qt_property!(QString; NOTIFY config_changed),
    config_changed: qt_signal!(),

    // Signals
    folder_changed: qt_signal!(),
    status_changed: qt_signal!(),
    progress_changed: qt_signal!(),
    processing_changed: qt_signal!(),
    metadata_changed: qt_signal!(),
    log_message: qt_signal!(message: QString),
    conversion_completed: qt_signal!(),
    error_occurred: qt_signal!(message: QString),

    // Qt Methods (callable from QML)
    save_config: qt_method!(fn(&mut self, url: QString, token: QString, id: QString)),
    set_folder_path: qt_method!(fn(&mut self, url_string: QString)),
    search_metadata: qt_method!(fn(&mut self, query: QString, by_asin: bool)),
    start_conversion: qt_method!(fn(&mut self)),
    cancel_conversion: qt_method!(fn(&mut self)),
    scan_chapters: qt_method!(fn(&mut self)),
}

impl LecternController {
    /// Initialize the controller
    fn initialize(&mut self) {
        // Load config
        self.load_config();
    }

    fn load_config(&mut self) {
        // Load from config file
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lectern")
            .join("config.json");

        if let Ok(content) = std::fs::read_to_string(config_path) {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                // Load ABS settings if available
                if let Some(host) = config.get("abs_host").and_then(|v| v.as_str()) {
                    self.abs_host = QString::from(host);
                }
                if let Some(token) = config.get("abs_token").and_then(|v| v.as_str()) {
                    self.abs_token = QString::from(token);
                }
                if let Some(library_id) = config.get("abs_library_id").and_then(|v| v.as_str()) {
                    self.abs_library_id = QString::from(library_id);
                }
            }
        }

        self.status_message = QString::from("Ready to process audiobooks");
        self.status_changed();
    }

    fn save_config(&mut self, url: QString, token: QString, id: QString) {
        self.abs_host = url;
        self.abs_token = token;
        self.abs_library_id = id;

        // Trigger the signal so the UI stays in sync
        self.config_changed();

        // Save current settings to config file
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lectern")
            .join("config.json");

        // Create directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let config = serde_json::json!({
            "abs_host": self.abs_host.to_string(),
            "abs_token": self.abs_token.to_string(),
            "abs_library_id": self.abs_library_id.to_string(),
        });

        if let Ok(json) = serde_json::to_string_pretty(&config) {
            if let Err(e) = std::fs::write(&config_path, json) {
                eprintln!("Failed to save config: {}", e);
            } else {
                println!("⚙️ Settings saved for: {}", self.abs_host.to_string());
            }
        }
    }

    fn set_folder_path(&mut self, url_string: QString) {
        let raw_url = url_string.to_string();

        // Parse the string as a URL to handle file:// protocol correctly
        let path = if let Ok(parsed_url) = Url::parse(&raw_url) {
            if parsed_url.scheme() == "file" {
                // to_file_path() handles %20 encoding, Windows vs Unix slashes, etc.
                parsed_url.to_file_path().unwrap_or_else(|_| PathBuf::from(&raw_url))
            } else {
                PathBuf::from(&raw_url)
            }
        } else {
            PathBuf::from(&raw_url)
        };

        // Store the clean, absolute path string
        let path_str = path.to_string_lossy().to_string();
        self.current_folder = QString::from(path_str.clone());
        self.folder_changed();

        self.status_message = QString::from(format!("Loaded folder: {}", path_str));
        self.status_changed();

        println!("📂 Folder set to: {:?}", path);
    }

    fn search_metadata(&mut self, query: QString, _by_asin: bool) {
        let qptr = QPointer::from(&*self);
        let query_str = query.to_string();

        // Show loading state in UI
        self.is_processing = true;
        self.status_message = QString::from(format!("Searching for '{}'...", query_str));
        self.status_changed();
        self.processing_changed();

        // Define the callback to update UI with results
        let on_complete = queued_callback(move |results: Vec<BookMetadata>| {
            if let Some(pinned) = qptr.as_pinned() {
                let mut s = pinned.borrow_mut();
                s.is_processing = false;

                // Grab the first result and update search result properties
                if let Some(book) = results.first() {
                    s.search_title = book.title.clone().into();
                    s.search_author = book.author().into();
                    s.search_cover_url = book.image_url.clone().into();
                    s.status_message = QString::from("Search completed");
                } else {
                    s.status_message = QString::from("No results found");
                }

                s.search_result_changed();
                s.status_changed();
                s.processing_changed();
            }
        });

        // Spawn background search
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Call Audnexus logic in services.rs
                match AudioService::fetch_metadata(&query_str).await {
                    Ok(metadata) => on_complete(vec![metadata]),
                    Err(_) => on_complete(vec![]),
                }
            });
        });
    }

    fn scan_chapters(&mut self) {
        // Placeholder for chapter scanning
        self.status_message = QString::from("Chapter scanning not yet implemented");
        self.status_changed();
        println!("📑 Scanning chapters from: {}", self.current_folder.to_string());
    }

    fn start_conversion(&mut self) {
        // Get current metadata and folder
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
            asin: "".to_string(), // TODO: extract from search
            duration_minutes: None,
            release_date: None,
        };

        let abs_config = ABSConfig {
            host: self.abs_host.to_string(),
            token: self.abs_token.to_string(),
            library_id: self.abs_library_id.to_string(),
        };

        // Set initial loading state on main thread
        self.is_processing = true;
        self.status_message = QString::from("Starting audio conversion...");
        self.progress_value = 0.0;
        self.processing_changed();
        self.status_changed();
        self.progress_changed();

        // Create thread-safe callback for conversion updates
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
                        
                        // Fire completion signal
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

        // Start background conversion
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let result = runtime.block_on(async {
                // Validate input
                let input_dir = PathBuf::from(folder);
                if !input_dir.exists() {
                    return Err("Input directory does not exist".to_string());
                }

                // Generate output path
                let output_path = input_dir
                    .parent()
                    .unwrap_or(&input_dir)
                    .join(format!("{}.m4b", metadata.title.replace(":", "").replace("/", "")));

                // Convert to M4B
                AudioService::convert_to_m4b_with_chapters(&input_dir, &output_path.to_string_lossy(), &metadata).await?;

                // Apply metadata
                AudioService::apply_tags(&output_path.to_string_lossy(), &metadata).await?;

                // Upload to ABS if configured
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

            // Send result back to UI
            update_progress(result);
        });
    }

    fn cancel_conversion(&mut self) {
        // For now, just reset the state (background task cancellation will be implemented later)
        self.is_processing = false;
        self.processing_changed();
        self.status_message = QString::from("Operation cancelled");
        self.status_changed();
    }
}

fn main() {
    println!("🎵 LECTERN starting...");
    println!("Arguments: {:?}", std::env::args().collect::<Vec<_>>());

    // Check display environment
    println!("Checking display environment...");
    if let Ok(display) = std::env::var("DISPLAY") {
        println!("DISPLAY: {}", display);
    }
    if let Ok(wayland) = std::env::var("WAYLAND_DISPLAY") {
        println!("WAYLAND_DISPLAY: {}", wayland);
    }
    
    // Detect if we have a display
    let has_display = std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok();
    println!("Has display: {}", has_display);

    // Initialize Qt environment
    if std::env::var("QT_QPA_PLATFORM").is_err() {
        std::env::set_var("QT_QPA_PLATFORM", "xcb");
    }
    
    println!("🎵 Starting Lectern GUI...");
    
    // Initialize qmetaobject
    println!("Initializing Qt...");
    qmetaobject::qml_register_type::<LecternController>(
        "Lectern",
        1,
        0,
        "LecternController",
    );
    println!("Qt initialized successfully");

    // Create and register the controller
    let mut controller = LecternController::default();
    controller.initialize();
    let controller = RefCell::new(controller);
    let controller_pinned = unsafe { QObjectPinned::new(&controller) };
    
    let mut engine = QmlEngine::new();
    engine.set_object_property("controller".into(), controller_pinned);

    // Load the UI
    println!("Loading main.qml...");
    engine.load_file("qml/main.qml".into());
    println!("main.qml loaded");

    println!("✅ Lectern window should now be visible!");
    println!("If you don't see it, check your display environment.");
    println!("Window title should be: 'Lectern - Audiobook Tool'");

    // Start the event loop (This blocks until the window is closed)
    engine.exec();
}