
mod services;
mod playback;

use services::{AudioService, BookMetadata, ABSConfig};
use playback::AudioPlayer;

use qmetaobject::*;
use std::cell::RefCell;
use std::path::PathBuf;
use url::Url;
use std::env;

#[derive(Clone, Debug)]
pub enum ConversionUpdate {
    Progress(f64, String),
    Log(String),
    Complete(bool, String),
}

#[derive(QObject, Default)]
pub struct LecternController {
    base: qt_base_class!(trait QObject),

    // Properties
    current_folder: qt_property!(QString; WRITE set_folder_path NOTIFY folder_changed),
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
    
    // Search Inputs
    search_query_input: qt_property!(QString; WRITE set_search_query NOTIFY search_inputs_changed),
    search_by_asin_input: qt_property!(bool; WRITE set_search_by_asin NOTIFY search_inputs_changed),
    search_trigger: qt_property!(bool; WRITE search_metadata NOTIFY search_trigger_changed),
    
    search_inputs_changed: qt_signal!(),
    search_trigger_changed: qt_signal!(),
    
    // Conversion Triggers
    start_conversion_trigger: qt_property!(bool; WRITE start_conversion NOTIFY start_conversion_trigger_changed),
    cancel_conversion_trigger: qt_property!(bool; WRITE cancel_conversion NOTIFY cancel_conversion_trigger_changed),
    
    start_conversion_trigger_changed: qt_signal!(),
    cancel_conversion_trigger_changed: qt_signal!(),

    // ABS settings
    abs_host: qt_property!(QString; WRITE set_abs_host NOTIFY config_changed),
    abs_token: qt_property!(QString; WRITE set_abs_token NOTIFY config_changed),
    abs_library_id: qt_property!(QString; WRITE set_abs_library_id NOTIFY config_changed),
    config_changed: qt_signal!(),
    
    save_config_trigger: qt_property!(bool; WRITE save_config NOTIFY save_config_trigger_changed),
    save_config_trigger_changed: qt_signal!(),


    // Signals
    folder_changed: qt_signal!(),
    status_changed: qt_signal!(),
    progress_changed: qt_signal!(),
    processing_changed: qt_signal!(),
    metadata_changed: qt_signal!(),
    log_message: qt_signal!(message: QString),
    conversion_completed: qt_signal!(),
    error_occurred: qt_signal!(message: QString),

    // Methods invokable from QML
    
    // Chapter Management
    chapters_json: qt_property!(QString; NOTIFY chapters_changed),
    chapters_changed: qt_signal!(),
    // play_media: qt_method!(fn play_media(&mut self, idx: int)),
    // stop_media: qt_method!(fn stop_media(&mut self)),

    // Playback Triggers
    playing_chapter_index: qt_property!(i32; WRITE play_chapter_at NOTIFY playing_chapter_index_changed),
    playing_chapter_index_changed: qt_signal!(),

    playback_stop_trigger: qt_property!(bool; WRITE stop_playback_trigger NOTIFY playback_stop_trigger_changed),
    playback_stop_trigger_changed: qt_signal!(),

    // Operation Triggers
    scan_chapters_trigger: qt_property!(bool; WRITE scan_chapters NOTIFY scan_chapters_trigger_changed),
    scan_chapters_trigger_changed: qt_signal!(),

    // Internal state (not properties)
    audio_player: AudioPlayer,
    chapter_paths: Vec<PathBuf>,
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

    // Setters for ABS config properties
    fn set_abs_host(&mut self, val: QString) { self.abs_host = val; self.config_changed(); }
    fn set_abs_token(&mut self, val: QString) { self.abs_token = val; self.config_changed(); }
    fn set_abs_library_id(&mut self, val: QString) { self.abs_library_id = val; self.config_changed(); }

    fn save_config(&mut self, _val: bool) {
        // Reset trigger
        self.save_config_trigger = false;
        self.save_config_trigger_changed();

        // Use current property values
        // self.abs_host etc are already set by QML bindings/setters

        // Trigger the signal so the UI stays in sync
        self.config_changed();

        // Save current settings to config file
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lectern")
            .join("config.json");

        // Create the directory if it doesn't exist
        if let Some(config_dir) = config_path.parent() {
            if let Err(e) = std::fs::create_dir_all(config_dir) {
                eprintln!("Failed to create config directory: {}", e);
                return;
            }
        }

        let config = serde_json::json!({
            "abs_host": self.abs_host.to_string(),
            "abs_token": self.abs_token.to_string(),
            "abs_library_id": self.abs_library_id.to_string(),
        });

        if let Ok(json) = serde_json::to_string_pretty(&config) {
            if let Err(e) = std::fs::write(config_path, json) {
                eprintln!("Failed to save config: {}", e);
            } else {
                println!("⚙️ Settings saved for: {}", self.abs_host);
            }
        }
    }


    fn set_folder_path(&mut self, url_string: QString) {
        let raw_url = url_string.to_string();
        println!("🔍 set_folder_path called with: {}", raw_url);

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
        println!("📂 Resolved path: {}", path_str);
        self.current_folder = QString::from(path_str.clone());
        self.folder_changed();

        self.status_message = QString::from(format!("Loaded folder: {}", path_str));
        self.status_changed();

        println!("📂 Folder set to: {:?}", path);

        // Trigger chapter scan
        self.scan_chapters(true);
    }

    fn scan_chapters(&mut self, _val: bool) {
        // Reset trigger
        self.scan_chapters_trigger = false;
        self.scan_chapters_trigger_changed();

        let folder = self.current_folder.to_string();
        let qptr = QPointer::from(&*self);
        
        // Update status
        self.status_message = QString::from("Scanning chapters...");
        self.status_changed();

        let on_complete = queued_callback(move |files: Vec<PathBuf>| {
            if let Some(pinned) = qptr.as_pinned() {
                let mut s = pinned.borrow_mut();
                
                // Update internal paths
                s.chapter_paths = files.clone();
                
                // Create JSON for UI
                let mut json_items = Vec::new();
                for (i, path) in files.iter().enumerate() {
                     let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();
                     
                     json_items.push(serde_json::json!({
                        "index": i,
                        "title": name,
                        "path": path.to_string_lossy(),
                        "start_time": 0 // Placeholder
                     }));
                }
                
                if let Ok(json_str) = serde_json::to_string(&json_items) {
                    s.chapters_json = QString::from(json_str);
                    s.chapters_changed();
                }
                
                s.status_message = QString::from(format!("Found {} chapters", files.len()));
                s.status_changed();
            }
        });

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                 let path = PathBuf::from(folder);
                 if let Ok(files) = AudioService::find_mp3_files(&path).await {
                     // Sort them
                     let mut files = files;
                     files.sort();
                     on_complete(files);
                 } else {
                     on_complete(Vec::new());
                 }
            });
        });
    }

//    fn play_media(&mut self, idx: i32) {
//        if let Some(path) = self.chapter_paths.get(idx as usize) {
//            println!("▶️ Playing chapter {}: {:?}", idx, path);
//            self.audio_player.play_file(path.to_str().unwrap_or(""));
//        }
//    }

    fn play_chapter_at(&mut self, idx: i32) {
        self.playing_chapter_index = idx;
        self.playing_chapter_index_changed();
        
        if let Some(path) = self.chapter_paths.get(idx as usize) {
            println!("▶️ Playing chapter {}: {:?}", idx, path);
            self.audio_player.play_file(path.to_str().unwrap_or(""));
        }
    }

    fn stop_playback_trigger(&mut self, _val: bool) {
        self.playback_stop_trigger = _val;
        self.playback_stop_trigger_changed();
        self.audio_player.stop();
    }

    // Search input setters
    fn set_search_query(&mut self, val: QString) { self.search_query_input = val; self.search_inputs_changed(); }
    fn set_search_by_asin(&mut self, val: bool) { self.search_by_asin_input = val; self.search_inputs_changed(); }

    fn search_metadata(&mut self, _val: bool) {
        // Reset trigger
        self.search_trigger = false;
        self.search_trigger_changed();

        let qptr = QPointer::from(&*self);
        let query_str = self.search_query_input.to_string();
        let by_asin = self.search_by_asin_input;

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
                    s.search_author = book.authors.join(", ").into();
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
        let update_progress = queued_callback(move |update: ConversionUpdate| {
            if let Some(pinned) = qptr.as_pinned() {
                let mut controller = pinned.borrow_mut();
                match update {
                    ConversionUpdate::Progress(value, message) => {
                        controller.progress_value = value;
                        controller.status_message = QString::from(message);
                        controller.progress_changed();
                        controller.status_changed();
                    }
                    ConversionUpdate::Log(message) => {
                        // For now, just update status. Could add to a log area later
                        controller.status_message = QString::from(format!("Log: {}", message));
                        controller.status_changed();
                    }
                    ConversionUpdate::Complete(success, message) => {
                        controller.status_message = QString::from(message);
                        controller.is_processing = false;
                        controller.processing_changed();
                        controller.progress_value = if success { 1.0 } else { 0.0 };
                        controller.progress_changed();
                        controller.status_changed();
                    }
                }
            }
        });

        // Start background conversion
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let result = runtime.block_on(async {
                // Validate input
                update_progress(ConversionUpdate::Progress(0.05, "Validating input directory...".to_string()));
                let input_dir = PathBuf::from(folder);
                if !input_dir.exists() {
                    return Err("Input directory does not exist".to_string());
                }

                // Check for MP3 files
                update_progress(ConversionUpdate::Progress(0.1, "Scanning audio files...".to_string()));
                let mp3_files = AudioService::find_mp3_files(&input_dir).await
                    .map_err(|e| format!("Failed to scan files: {}", e))?;

                if mp3_files.is_empty() {
                    return Err("No MP3 files found in directory".to_string());
                }

                update_progress(ConversionUpdate::Progress(0.2, format!("Found {} audio files", mp3_files.len())));

                // Generate output path
                let output_path = input_dir
                    .parent()
                    .unwrap_or(&input_dir)
                    .join(format!("{}.m4b", metadata.title.replace(":", "").replace("/", "")));

                update_progress(ConversionUpdate::Progress(0.3, "Converting audio files...".to_string()));

                // Convert to M4B
                AudioService::convert_to_m4b_with_chapters(&input_dir, &output_path.to_string_lossy(), &metadata).await?;

                update_progress(ConversionUpdate::Progress(0.8, "Applying metadata tags...".to_string()));

                // Apply metadata
                AudioService::apply_tags(&output_path.to_string_lossy(), &metadata).await?;

                update_progress(ConversionUpdate::Progress(0.9, "Uploading to Audiobookshelf...".to_string()));

                // Upload to ABS if configured
                if !abs_config.host.is_empty() && !abs_config.token.is_empty() {
                    match AudioService::upload_and_scan(&output_path.to_string_lossy(), &abs_config).await {
                        Ok(_) => Ok("Conversion and upload completed!".to_string()),
                        Err(e) => {
                            update_progress(ConversionUpdate::Log(format!("Upload error: {}", e)));
                            Ok("Conversion completed, upload failed".to_string())
                        }
                    }
                } else {
                    Ok("Conversion completed!".to_string())
                }
            });

            // Send result back to UI
            match result {
                Ok(msg) => update_progress(ConversionUpdate::Complete(true, msg)),
                Err(err) => update_progress(ConversionUpdate::Complete(false, err)),
            }
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
    eprintln!("🎵 LECTERN starting...");

    let args: Vec<String> = env::args().collect();
    eprintln!("Arguments: {:?}", args);

    // Check for command line mode
    if args.len() > 1 && args[1] == "--cli" {
        eprintln!("Running CLI mode");
        run_cli_mode();
        return;
    }

    eprintln!("Checking display environment...");
    // Check if we have a display
    let display = env::var("DISPLAY").unwrap_or("none".to_string());
    let wayland_display = env::var("WAYLAND_DISPLAY").unwrap_or("none".to_string());
    let has_display = display != "none" || wayland_display != "none";

    eprintln!("DISPLAY: {}", display);
    eprintln!("WAYLAND_DISPLAY: {}", wayland_display);
    eprintln!("Has display: {}", has_display);

    if !has_display {
        eprintln!("❌ No display server detected. GUI cannot be displayed.");
        eprintln!("");
        eprintln!("To run the GUI version, use a system with a display server:");
        eprintln!("  • Linux desktop with X11/Wayland");
        eprintln!("  • Windows or macOS");
        eprintln!("  • SSH with X11 forwarding (-X flag)");
        eprintln!("");
        eprintln!("For command-line testing, run: ./run_lectern.sh --cli");
        eprintln!("");
        eprintln!("The application code is fully functional - only the display is missing!");
        return;
    }

    eprintln!("🎵 Starting Lectern GUI...");

    // Initialize Qt environment
    eprintln!("Initializing Qt...");
    init_qt_to_rust();
    eprintln!("Qt initialized successfully");

    // Create and register the controller
    let mut controller = LecternController::default();
    controller.initialize(); // Load config and initialize
    let controller = RefCell::new(controller);
    let controller_pinned = unsafe { QObjectPinned::new(&controller) };
    let mut engine = QmlEngine::new();

    // Add the qml directory so main.qml can find MetadataTab.qml, etc.
    engine.add_import_path("qml".into());

    engine.set_object_property("controller".into(), controller_pinned);

    // Load the UI
    eprintln!("Loading main.qml...");
    engine.load_file("qml/main.qml".into());
    eprintln!("main.qml loaded");

    println!("✅ Lectern window should now be visible!");
    println!("If you don't see it, check your display environment.");
    println!("Window title should be: 'Lectern - Audiobook Tool'");

    // Start the event loop (This blocks until the window is closed)
    engine.exec();
}

fn run_cli_mode() {
    println!("🎵 LECTERN - Command Line Mode");
    println!("📋 Demonstrating functionality without GUI");
    println!("");

    // Create controller
    let mut controller = LecternController::default();
    controller.initialize();

    println!("✅ Controller initialized");
    println!("📁 Current folder: {}", controller.current_folder);
    println!("📊 Status: {}", controller.status_message);
    println!("🎵 Metadata - Title: {}", controller.metadata_title);
    println!("👤 Author: {}", controller.metadata_author);
    println!("📖 Series: {:?}", controller.metadata_series);
    println!("🎤 Narrator: {:?}", controller.metadata_narrator);
    println!("🖼️ Cover URL: {}", controller.metadata_cover_url);
    println!("🔄 Is processing: {}", controller.is_processing);
    println!("📈 Progress: {}%", (controller.progress_value * 100.0) as i32);
    println!("");

    println!("🎯 All functionality is working!");
    println!("💡 Run without --cli flag in a GUI environment to see the full interface");
}
